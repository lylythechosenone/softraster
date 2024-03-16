use std::collections::hash_map::Entry;

use llvm_sys::{core::LLVMConstNull, LLVMValue};
use naga::{Expression, Handle, Type};

use crate::codegen::expressions::location::Global;

use self::{index::Index, location::Location};

use super::{error::Error, Generator};

mod access;
mod binary;
mod compose;
mod index;
mod literal;
mod load;
pub(super) mod location;
mod splat;
mod swizzle;

impl Generator {
    fn eval_expr<L: Location>(
        &mut self,
        location: &L,
        expr: Handle<Expression>,
    ) -> Result<(*mut LLVMValue, Handle<Type>), Error> {
        let exprs = location.get_exprs(&self.module);
        let span = exprs.get_span(expr);

        #[cfg(test)]
        println!("evaluating expression {expr:?} ({:?})", &exprs[expr]);

        match &exprs[expr] {
            &Expression::Literal(literal) => self.build_literal(span, literal),
            &Expression::Constant(constant) => {
                // constants cannot be cached, because they would interfere with
                // expr cache.
                self.eval_expr::<Global>(&Global, self.module.src.constants[constant].init)
            }
            &Expression::ZeroValue(ty) => Ok((
                unsafe { LLVMConstNull(self.eval_cached_type(span, ty)?) },
                ty,
            )),
            &Expression::Compose { ty, ref components } => {
                self.build_compose(location, span, ty, components.clone())
            }
            &Expression::Access { base, index } => {
                self.build_access(location, span, base, Index::Dynamic(index))
            }
            &Expression::AccessIndex { base, index } => {
                self.build_access(location, span, base, Index::Constant(index as usize))
            }
            &Expression::Splat { size, value } => self.build_splat(location, span, value, size),
            &Expression::Swizzle {
                size,
                vector,
                pattern,
            } => {
                let (vector, ty) = self.eval_cached_expr(location, vector)?;
                self.build_swizzle(
                    span,
                    size,
                    vector,
                    ty,
                    [
                        pattern[0] as usize,
                        pattern[1] as usize,
                        pattern[2] as usize,
                        pattern[3] as usize,
                    ],
                )
            }
            &Expression::FunctionArgument(index) => Ok(location.get_params()[index as usize]),
            Expression::GlobalVariable(handle) => Ok(self.globals[handle.index()]),
            Expression::LocalVariable(handle) => Ok(location.get_locals()[handle.index()]),
            &Expression::Load { pointer } => self.build_load(location, span, pointer),
            Expression::ImageSample { .. } => todo!(),
            Expression::ImageLoad { .. } => todo!(),
            Expression::ImageQuery { .. } => todo!(),
            Expression::Unary { .. } => todo!(),
            &Expression::Binary { op, left, right } => {
                self.build_binary(location, span, op, left, right)
            }
            Expression::Select { .. } => todo!(),
            Expression::Derivative { .. } => todo!(),
            Expression::Relational { .. } => todo!(),
            Expression::Math { .. } => todo!(),
            Expression::As { .. } => todo!(),
            Expression::CallResult(_) => todo!(),
            Expression::AtomicResult { .. } => todo!(),
            Expression::WorkGroupUniformLoadResult { .. } => todo!(),
            Expression::ArrayLength(_) => todo!(),
            Expression::RayQueryProceedResult => todo!(),
            Expression::RayQueryGetIntersection { .. } => todo!(),
        }
    }

    pub(super) fn eval_cached_expr<L: Location>(
        &mut self,
        location: &L,
        expr: Handle<Expression>,
    ) -> Result<(*mut LLVMValue, Handle<Type>), Error> {
        #[cfg(test)]
        println!(
            "need expression {expr:?} ({:?})",
            location.get_exprs(&self.module)[expr]
        );

        match self.expr_cache.entry(expr) {
            Entry::Occupied(entry) => Ok(*entry.get()),
            Entry::Vacant(_) => {
                let generated = self.eval_expr(location, expr)?;
                Ok(*self.expr_cache.entry(expr).or_insert(generated))
            }
        }
    }
}
