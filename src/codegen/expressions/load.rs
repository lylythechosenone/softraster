use std::ffi::c_uint;

use llvm_sys::{
    core::{LLVMBuildLoad2, LLVMInt64TypeInContext, LLVMVectorType},
    target::LLVMPreferredAlignmentOfType,
    LLVMValue,
};
use naga::{Expression, Handle, Span, Type, TypeInner};

use crate::codegen::{
    error::{Error, Info},
    Generator, EMPTY_CSTR,
};

use super::location::Location;

impl Generator {
    pub(super) fn build_load<L: Location>(
        &mut self,
        location: &L,
        span: Span,
        pointer: Handle<Expression>,
    ) -> Result<(*mut LLVMValue, Handle<Type>), Error> {
        let (pointer, ty) = self.eval_cached_expr(location, pointer)?;
        let Some((inner_ty, _)) = self.get_pointee(ty) else {
            return Err(Error {
                info: Info::NotPointer(ty),
                span,
            });
        };

        match &self.module.src.types[inner_ty].inner {
            &TypeInner::Matrix {
                rows,
                columns,
                scalar,
            } => {
                let llvm_scalar = self.eval_scalar_type(span, scalar)?;
                let column = unsafe { LLVMVectorType(llvm_scalar, rows as c_uint) };
                let stride = unsafe {
                    LLVMPreferredAlignmentOfType(self.target_info.data_layout, column)
                        / scalar.width as u32
                };
                let args = [
                    pointer,
                    self.const_u64(stride as u64),
                    self.const_bool(false),
                    self.const_u32(rows as c_uint),
                    self.const_u32(columns as c_uint),
                ];
                Ok((
                    self.build_intrinsic_call(
                        "llvm.matrix.column.major.load",
                        &[
                            unsafe {
                                LLVMVectorType(llvm_scalar, columns as c_uint * rows as c_uint)
                            },
                            unsafe { LLVMInt64TypeInContext(self.module.context) },
                        ],
                        &args,
                    ),
                    inner_ty,
                ))
            }
            _ => Ok((
                unsafe {
                    LLVMBuildLoad2(
                        self.builder.0,
                        self.eval_cached_type(span, inner_ty)?,
                        pointer,
                        EMPTY_CSTR,
                    )
                },
                inner_ty,
            )),
        }
    }
}
