use std::collections::hash_map::Entry;

use llvm_sys::{
    core::{LLVMBuildExtractElement, LLVMBuildExtractValue, LLVMBuildGEP2},
    LLVMValue,
};
use naga::{Expression, Handle, Literal, Span, Type, TypeInner};

use crate::codegen::{
    error::{Error, Info},
    Generator, EMPTY_CSTR,
};

use super::{index::Index, location::Location};

impl Generator {
    pub(super) fn build_access<L: Location>(
        &mut self,
        location: &L,
        span: Span,
        base: Handle<Expression>,
        index: Index,
    ) -> Result<(*mut LLVMValue, Handle<Type>), Error> {
        let (base, ty) = self.eval_cached_expr(location, base)?;
        let index = match (index, self.struct_maps.entry(ty)) {
            (Index::Constant(index), Entry::Occupied(entry)) => {
                Index::Constant(entry.get()[index] as usize)
            }
            (other, _) => other,
        };

        if let Some((inner, space)) = self.get_pointee(ty) {
            let llvm_index = match index {
                Index::Dynamic(value) => self.eval_cached_expr(location, value)?.0,
                Index::Constant(constant) => {
                    if let TypeInner::Struct { .. } = self.module.src.types[inner].inner {
                        self.const_u32(constant as u32)
                    } else {
                        self.const_u64(constant as u64)
                    }
                }
            };

            let llvm_inner = self.eval_cached_type(span, inner)?;
            let item_type = self.item_type(
                span,
                inner,
                match index {
                    Index::Dynamic(_) => None,
                    Index::Constant(constant) => Some(constant),
                },
            )?;
            Ok((
                unsafe {
                    LLVMBuildGEP2(
                        self.builder.0,
                        llvm_inner,
                        base,
                        [self.const_u64(0), llvm_index].as_ptr().cast_mut(),
                        2,
                        EMPTY_CSTR,
                    )
                },
                self.module.src.types.insert(
                    Type {
                        name: None,
                        inner: TypeInner::Pointer {
                            base: item_type,
                            space,
                        },
                    },
                    self.module.src.types.get_span(inner),
                ),
            ))
        } else {
            let llvm_index = match index {
                Index::Dynamic(value) => self.eval_cached_expr(location, value)?.0,
                Index::Constant(constant) => self.const_u64(constant as u64),
            };

            let item_type = self.item_type(
                span,
                ty,
                match index {
                    Index::Dynamic(_) => None,
                    Index::Constant(constant) => Some(constant),
                },
            )?;
            Ok((
                if let TypeInner::Vector { .. } = self.module.src.types[ty].inner {
                    unsafe { LLVMBuildExtractElement(self.builder.0, base, llvm_index, EMPTY_CSTR) }
                } else {
                    let index = match index {
                        Index::Constant(constant) => constant,
                        Index::Dynamic(expr) => match location.get_exprs(&self.module)[expr] {
                            Expression::Literal(Literal::U32(v)) => v as usize,
                            Expression::ZeroValue(_) => 0,
                            _ => {
                                return Err(Error {
                                    info: Info::NonConstantIndex,
                                    span,
                                })
                            }
                        },
                    };
                    unsafe { LLVMBuildExtractValue(self.builder.0, base, index as u32, EMPTY_CSTR) }
                },
                item_type,
            ))
        }
    }
}
