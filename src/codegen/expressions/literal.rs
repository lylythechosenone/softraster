use llvm_sys::core::{LLVMConstReal, LLVMDoubleTypeInContext, LLVMFloatTypeInContext};
use naga::{Handle, Literal, Scalar, Span, Type, TypeInner};

use crate::codegen::{
    error::{Error, Info},
    Generator,
};

impl Generator {
    pub(super) fn build_literal(
        &mut self,
        span: Span,
        literal: Literal,
    ) -> Result<(*mut llvm_sys::LLVMValue, Handle<Type>), Error> {
        match literal {
            Literal::F64(val) => Ok((
                unsafe { LLVMConstReal(LLVMDoubleTypeInContext(self.module.context), val) },
                self.module.src.types.insert(
                    Type {
                        name: None,
                        inner: TypeInner::Scalar(Scalar::F64),
                    },
                    Span::UNDEFINED,
                ),
            )),
            Literal::F32(val) => Ok((
                unsafe { LLVMConstReal(LLVMFloatTypeInContext(self.module.context), val as f64) },
                self.module.src.types.insert(
                    Type {
                        name: None,
                        inner: TypeInner::Scalar(Scalar::F32),
                    },
                    Span::UNDEFINED,
                ),
            )),
            Literal::U32(val) => Ok((
                self.const_u32(val),
                self.module.src.types.insert(
                    Type {
                        name: None,
                        inner: TypeInner::Scalar(Scalar::U32),
                    },
                    Span::UNDEFINED,
                ),
            )),
            Literal::I32(val) => Ok((
                self.const_i32(val),
                self.module.src.types.insert(
                    Type {
                        name: None,
                        inner: TypeInner::Scalar(Scalar::I32),
                    },
                    Span::UNDEFINED,
                ),
            )),
            Literal::I64(val) => Ok((
                self.const_i64(val),
                self.module.src.types.insert(
                    Type {
                        name: None,
                        inner: TypeInner::Scalar(Scalar::I64),
                    },
                    Span::UNDEFINED,
                ),
            )),
            Literal::Bool(val) => Ok((
                self.const_bool(val),
                self.module.src.types.insert(
                    Type {
                        name: None,
                        inner: TypeInner::Scalar(Scalar::BOOL),
                    },
                    Span::UNDEFINED,
                ),
            )),
            other => Err(Error {
                info: Info::IllegalLiteral(other),
                span,
            }),
        }
    }
}
