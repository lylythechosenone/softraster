use llvm_sys::{
    core::{LLVMBuildInsertElement, LLVMConstVector, LLVMGetUndef, LLVMIsConstant, LLVMVectorType},
    LLVMValue,
};
use naga::{Expression, Handle, Span, Type, TypeInner, VectorSize};

use crate::codegen::{error::Error, Generator, EMPTY_CSTR};

use super::location::Location;

impl Generator {
    pub(super) fn build_splat<L: Location>(
        &mut self,
        location: &L,
        span: Span,
        value: Handle<Expression>,
        size: VectorSize,
    ) -> Result<(*mut LLVMValue, Handle<Type>), Error> {
        let (value, scalar) = self.eval_cached_expr(location, value)?;
        let ty = self.module.src.types.insert(
            Type {
                name: None,
                inner: TypeInner::Vector {
                    size,
                    scalar: match self.module.src.types[scalar].inner {
                        TypeInner::Scalar(scalar) => scalar,
                        _ => panic!("Attempting to splat vector with non-scalar value"),
                    },
                },
            },
            self.module.src.types.get_span(scalar),
        );
        if unsafe { LLVMIsConstant(value) } != 0 {
            let value = match size {
                naga::VectorSize::Bi => unsafe {
                    LLVMConstVector([value; 2].as_ptr().cast_mut(), 2)
                },
                naga::VectorSize::Tri => unsafe {
                    LLVMConstVector([value; 3].as_ptr().cast_mut(), 3)
                },
                naga::VectorSize::Quad => unsafe {
                    LLVMConstVector([value; 4].as_ptr().cast_mut(), 4)
                },
            };
            Ok((value, ty))
        } else {
            let mut vec = unsafe {
                LLVMGetUndef(LLVMVectorType(
                    self.eval_cached_type(span, scalar)?,
                    size as u32,
                ))
            };

            for i in 0..(size as u64) {
                vec = unsafe {
                    LLVMBuildInsertElement(
                        self.builder.0,
                        vec,
                        value,
                        self.const_u64(i),
                        EMPTY_CSTR,
                    )
                };
            }

            Ok((vec, ty))
        }
    }
}
