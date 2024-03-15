#![allow(
    clippy::cast_lossless,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::missing_errors_doc
)]

use std::{
    collections::HashMap,
    ffi::{c_char, c_uint, CStr},
    fmt::Debug,
};

use llvm_sys::{
    analysis::{LLVMVerifierFailureAction, LLVMVerifyModule},
    core::{
        LLVMAddFunction, LLVMAddGlobal, LLVMAppendBasicBlockInContext, LLVMBuildAlloca,
        LLVMBuildStore, LLVMContextCreate, LLVMCreateBuilderInContext, LLVMDisposeBuilder,
        LLVMDisposeMessage, LLVMFunctionType, LLVMGetParam, LLVMInt64TypeInContext, LLVMIsConstant,
        LLVMModuleCreateWithNameInContext, LLVMPositionBuilderAtEnd, LLVMSetInitializer,
        LLVMSetSourceFileName, LLVMTypeOf, LLVMVectorType, LLVMVoidTypeInContext,
    },
    target::LLVMPreferredAlignmentOfType,
    LLVMBuilder, LLVMType, LLVMValue,
};
use naga::{
    AddressSpace, Expression, Function, FunctionArgument, FunctionResult, GlobalVariable, Handle,
    LocalVariable, Module as SrcModule, Span, Type, TypeInner,
};

use crate::codegen::error::Info;

use self::{error::Error, expressions::location::Location, target::TargetInfo};
use self::{expressions::location::Global, module::Module};

pub mod error;
mod expressions;
pub mod module;
mod statements;
mod target;
mod types;
mod util;

const EMPTY_CSTR: *const c_char = "\0".as_ptr().cast();

struct BuilderWrapper(*mut LLVMBuilder);
unsafe impl Send for BuilderWrapper {}
impl Drop for BuilderWrapper {
    fn drop(&mut self) {
        unsafe { LLVMDisposeBuilder(self.0) }
    }
}

struct FunctionState {
    src_function: Function,
    locals: Vec<(*mut LLVMValue, Handle<Type>)>,
    params: Vec<(*mut LLVMValue, Handle<Type>)>,
}

struct DebugOnDrop<T: Debug>(T);
impl<T: Debug> DebugOnDrop<T> {
    pub fn cancel(self) {
        core::mem::forget(self);
    }
}
impl<T: Debug> Drop for DebugOnDrop<T> {
    fn drop(&mut self) {
        #[cfg(test)]
        println!("{:?}", self.0);
    }
}

pub struct Generator {
    module: Module,
    builder: BuilderWrapper,
    target_info: TargetInfo,
    expr_cache: HashMap<Handle<Expression>, (*mut LLVMValue, Handle<Type>)>,
    type_cache: HashMap<Handle<Type>, *mut LLVMType>,
    pointer_type_cache: HashMap<Handle<Type>, *mut LLVMType>,
    globals: Vec<(*mut LLVMValue, Handle<Type>)>,
    struct_maps: HashMap<Handle<Type>, Vec<u32>>,
}
impl Generator {
    #[must_use]
    pub fn new(src: SrcModule, file_name: &str) -> Self {
        let context = unsafe { LLVMContextCreate() };

        let mut name = file_name.split('.').next().unwrap_or(file_name).to_string();
        name.push('\0');
        let module = unsafe { LLVMModuleCreateWithNameInContext(name.as_ptr().cast(), context) };

        unsafe { LLVMSetSourceFileName(module, file_name.as_ptr().cast(), file_name.len()) }

        let builder = BuilderWrapper(unsafe { LLVMCreateBuilderInContext(context) });

        Self {
            module: Module {
                context,
                module,
                src,
            },
            builder,
            target_info: TargetInfo::native(),
            expr_cache: HashMap::new(),
            type_cache: HashMap::new(),
            pointer_type_cache: HashMap::new(),
            globals: Vec::new(),
            struct_maps: HashMap::new(),
        }
    }

    fn build_store<L: Location>(
        &mut self,
        location: &L,
        span: Span,
        pointer: *mut LLVMValue,
        value: Handle<Expression>,
    ) -> Result<(), Error> {
        let (value, ty) = self.eval_cached_expr(location, value)?;

        match &self.module.src.types[ty].inner {
            &TypeInner::Matrix {
                rows,
                columns,
                scalar,
            } => {
                let llvm_scalar = self.eval_scalar_type(span, scalar)?;
                let stride = unsafe {
                    LLVMPreferredAlignmentOfType(
                        self.target_info.data_layout,
                        LLVMVectorType(llvm_scalar, rows as c_uint),
                    ) / scalar.width as u32
                };

                let args = [
                    value,
                    pointer,
                    self.const_u64(stride as u64),
                    self.const_bool(false),
                    self.const_u32(rows as u32),
                    self.const_u32(columns as u32),
                ];

                self.build_intrinsic_call(
                    "llvm.matrix.column.major.store",
                    &[unsafe { LLVMTypeOf(value) }, unsafe {
                        LLVMInt64TypeInContext(self.module.context)
                    }],
                    &args,
                );
            }
            _ => unsafe {
                LLVMBuildStore(self.builder.0, value, pointer);
            },
        }

        Ok(())
    }

    fn generate_globals(&mut self) -> Result<(), Error> {
        for GlobalVariable {
            name,
            ty,
            init,
            space,
            ..
        } in self.module.src.global_variables.clone().into_inner()
        {
            let Some(mut name) = name else {
                unimplemented!("unnamed globals")
            };
            name.push('\0');

            let global = unsafe {
                LLVMAddGlobal(
                    self.module.module,
                    self.eval_cached_type(Span::UNDEFINED, ty)?,
                    name.as_ptr().cast(),
                )
            };

            if let Some(init) = init {
                let (llvm_init, _) = self.eval_cached_expr::<Global>(&Global, init)?;
                if unsafe { LLVMIsConstant(llvm_init) } == 0 {
                    return Err(Error {
                        info: Info::NonConstantGlobal,
                        span: self.module.src.const_expressions.get_span(init),
                    });
                }
                unsafe { LLVMSetInitializer(global, llvm_init) };
            }

            self.globals.push((
                global,
                self.module.src.types.insert(
                    Type {
                        name: None,
                        inner: TypeInner::Pointer { base: ty, space },
                    },
                    self.module.src.types.get_span(ty),
                ),
            ));
        }

        Ok(())
    }

    fn generate_llvm_function(
        &mut self,
        mut function: Function,
    ) -> Result<(*mut LLVMValue, FunctionState), Error> {
        let mut param_types = Vec::with_capacity(function.arguments.len());
        for &FunctionArgument { ty, .. } in &function.arguments {
            param_types.push(self.eval_cached_type(Span::UNDEFINED, ty)?);
        }
        let function_type = unsafe {
            LLVMFunctionType(
                match function.result {
                    Some(FunctionResult { ty, .. }) => {
                        self.eval_cached_type(Span::UNDEFINED, ty)?
                    }
                    None => LLVMVoidTypeInContext(self.module.context),
                },
                param_types.as_ptr().cast_mut(),
                param_types.len() as c_uint,
                0,
            )
        };

        let Some(name) = &mut function.name else {
            unimplemented!("unnamed functions")
        };
        name.push('\0');

        let llvm_function =
            unsafe { LLVMAddFunction(self.module.module, name.as_ptr().cast(), function_type) };
        let basic_block = unsafe {
            LLVMAppendBasicBlockInContext(
                self.module.context,
                llvm_function,
                "entry\0".as_ptr().cast(),
            )
        };
        unsafe { LLVMPositionBuilderAtEnd(self.builder.0, basic_block) }

        let locals = Vec::with_capacity(function.local_variables.len());
        let params = Vec::with_capacity(function.arguments.len());

        Ok((
            llvm_function,
            FunctionState {
                src_function: function,
                locals,
                params,
            },
        ))
    }

    fn generate_locals(&mut self, state: &mut FunctionState) -> Result<(), Error> {
        for (handle, &LocalVariable { ty, init, .. }) in state.src_function.local_variables.iter() {
            let pointer = unsafe {
                LLVMBuildAlloca(
                    self.builder.0,
                    self.eval_cached_pointer_type(self.module.src.types.get_span(ty), ty)?,
                    EMPTY_CSTR,
                )
            };

            if let Some(init) = init {
                self.build_store(
                    state,
                    state.src_function.local_variables.get_span(handle),
                    pointer,
                    init,
                )?;
            }

            state.locals.push((
                pointer,
                self.module.src.types.insert(
                    Type {
                        name: None,
                        inner: TypeInner::Pointer {
                            base: ty,
                            space: AddressSpace::Function,
                        },
                    },
                    self.module.src.types.get_span(ty),
                ),
            ));
        }

        Ok(())
    }

    fn validate(&mut self) {
        let mut error = core::ptr::null_mut();
        let result = unsafe {
            LLVMVerifyModule(
                self.module.module,
                LLVMVerifierFailureAction::LLVMReturnStatusAction,
                &mut error,
            )
        };

        assert!(
            result == 0,
            "Error in generated LLVM module: {}\nModule: {:?}",
            unsafe { CStr::from_ptr(error) }.to_str().unwrap(),
            self.module
        );

        unsafe { LLVMDisposeMessage(error) };
    }

    /// # Panics
    /// This function panics if it generates an invalid LLVM module. This is
    /// most certainly a bug, and should be reported.
    pub fn generate(mut self) -> Result<Module, Error> {
        self.generate_globals()?;

        for function in self.module.src.functions.clone().into_inner() {
            self.expr_cache.clear();

            #[cfg(test)]
            println!(
                "generating function {} with expressions {:#?}",
                function.name.as_deref().unwrap_or("{unnamed}"),
                function.expressions
            );

            let (llvm_function, mut state) = self.generate_llvm_function(function)?;

            self.generate_locals(&mut state)?;

            for (i, &FunctionArgument { ty, .. }) in state.src_function.arguments.iter().enumerate()
            {
                state
                    .params
                    .push((unsafe { LLVMGetParam(llvm_function, i as c_uint) }, ty));
            }

            for (stmt, &span) in state.src_function.body.span_iter() {
                let panic_catcher = DebugOnDrop(unsafe { &*core::ptr::addr_of!(self.module) });
                self.emit_statement(span, &state, stmt)?;
                panic_catcher.cancel();
            }
        }

        self.validate();

        Ok(self.module)
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use naga::front::wgsl;

    use super::*;

    #[test]
    fn codegen() {
        for entry in std::fs::read_dir("tests").unwrap() {
            let entry = entry.unwrap();
            let generator = Generator::new(
                wgsl::parse_str(&std::fs::read_to_string(entry.path()).unwrap()).unwrap(),
                entry.file_name().to_str().unwrap(),
            );

            let module = generator.generate().unwrap();

            module
                .write_string(
                    File::create(format!("target/{}", entry.file_name().to_str().unwrap()))
                        .unwrap(),
                )
                .unwrap();
        }
    }
}
