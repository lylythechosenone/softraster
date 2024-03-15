use std::ffi::c_uint;

use llvm_sys::core::{LLVMBuildInsertElement, LLVMBuildInsertValue, LLVMGetUndef};
use naga::{Expression, Handle, Span, Type, TypeInner};

use crate::codegen::{
    error::{Error, Info},
    Generator, EMPTY_CSTR,
};

use super::location::Location;

impl Generator {
    pub(super) fn build_compose<L: Location>(
        &mut self,
        location: &L,
        span: Span,
        ty: Handle<Type>,
        components: Vec<Handle<Expression>>,
    ) -> Result<(*mut llvm_sys::LLVMValue, Handle<Type>), Error> {
        match &self.module.src.types[ty].inner {
            TypeInner::Vector { .. } => {
                let llvm_ty = self.eval_cached_type(span, ty)?;
                let mut val = unsafe { LLVMGetUndef(llvm_ty) };

                for (i, component) in components.into_iter().enumerate() {
                    let (component, _) = self.eval_cached_expr(location, component)?;
                    val = unsafe {
                        LLVMBuildInsertElement(
                            self.builder.0,
                            val,
                            component,
                            self.const_u64(i as u64),
                            EMPTY_CSTR,
                        )
                    };
                }

                Ok((val, ty))
            }
            TypeInner::Matrix { .. } => {
                let mut val = unsafe { LLVMGetUndef(self.eval_cached_type(span, ty)?) };

                for (i, component) in components.into_iter().enumerate() {
                    let (component, ty) = self.eval_cached_expr(location, component)?;
                    if let TypeInner::Vector { .. } = self.module.src.types[ty].inner {
                        val = unsafe {
                            LLVMBuildInsertValue(
                                self.builder.0,
                                val,
                                component,
                                i as c_uint,
                                EMPTY_CSTR,
                            )
                        };
                    } else {
                        return Err(Error {
                            info: Info::InvalidComponent(ty),
                            span,
                        });
                    }
                }

                Ok((val, ty))
            }
            &TypeInner::Array { .. } => {
                let llvm_ty = self.eval_cached_type(span, ty)?;
                let mut val = unsafe { LLVMGetUndef(llvm_ty) };

                for (i, component) in components.into_iter().enumerate() {
                    let (component, _) = self.eval_cached_expr(location, component)?;
                    val = unsafe {
                        LLVMBuildInsertValue(
                            self.builder.0,
                            val,
                            component,
                            i as c_uint,
                            EMPTY_CSTR,
                        )
                    };
                }

                Ok((val, ty))
            }
            TypeInner::Struct { .. } => {
                let llvm_ty = self.eval_cached_type(span, ty)?;
                let mut val = unsafe { LLVMGetUndef(llvm_ty) };

                for (i, component) in self
                    .struct_maps
                    .get(&ty)
                    .unwrap()
                    .clone()
                    .into_iter()
                    .zip(components.into_iter())
                {
                    let (component, _) = self.eval_cached_expr(location, component)?;
                    val = unsafe {
                        LLVMBuildInsertValue(
                            self.builder.0,
                            val,
                            component,
                            i as c_uint,
                            EMPTY_CSTR,
                        )
                    };
                }

                Ok((val, ty))
            }
            _ => Err(Error {
                info: Info::NotComposable(ty),
                span,
            }),
        }
    }
}
