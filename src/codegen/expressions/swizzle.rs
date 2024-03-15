use std::ffi::c_uint;

use llvm_sys::{
    core::{
        LLVMBuildInsertElement, LLVMBuildShuffleVector, LLVMGetUndef, LLVMInt64TypeInContext,
        LLVMVectorType,
    },
    LLVMValue,
};
use naga::{Expression, Handle, Span, SwizzleComponent, Type, TypeInner, VectorSize};

use crate::codegen::{
    error::{Error, Info},
    Generator, EMPTY_CSTR,
};

use super::location::Location;

impl Generator {
    pub(super) fn build_swizzle<L: Location>(
        &mut self,
        location: &L,
        span: Span,
        size: VectorSize,
        vector: Handle<Expression>,
        pattern: [SwizzleComponent; 4],
    ) -> Result<(*mut LLVMValue, Handle<Type>), Error> {
        let (vector, ty) = self.eval_cached_expr(location, vector)?;
        let TypeInner::Vector {
            scalar,
            size: in_size,
            ..
        } = self.module.src.types[ty].inner
        else {
            return Err(Error {
                info: Info::NonVectorSwizzle(ty),
                span,
            });
        };
        Ok((
            unsafe {
                let u64_ty = LLVMInt64TypeInContext(self.module.context);
                let mut mask = LLVMGetUndef(LLVMVectorType(u64_ty, size as c_uint));

                for (i, &pattern) in pattern.iter().enumerate().take(size as usize) {
                    if pattern as usize > in_size as usize {
                        return Err(Error {
                            info: Info::NoSuchComponent(ty, pattern as usize),
                            span,
                        });
                    }
                    mask = LLVMBuildInsertElement(
                        self.builder.0,
                        mask,
                        self.const_u64(pattern as u64),
                        self.const_u64(i as u64),
                        EMPTY_CSTR,
                    );
                }

                LLVMBuildShuffleVector(
                    self.builder.0,
                    vector,
                    LLVMGetUndef(self.eval_cached_type(span, ty)?),
                    mask,
                    EMPTY_CSTR,
                )
            },
            self.module.src.types.insert(
                Type {
                    name: None,
                    inner: TypeInner::Vector { size, scalar },
                },
                span,
            ),
        ))
    }
}
