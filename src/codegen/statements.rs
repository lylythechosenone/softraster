use llvm_sys::core::{LLVMBuildRet, LLVMBuildRetVoid};
use naga::{FunctionResult, Span, Statement};

use crate::codegen::error::Info;

use super::{error::Error, FunctionState, Generator};

impl Generator {
    pub(super) fn emit_statement(
        &mut self,
        span: Span,
        function: &FunctionState,
        stmt: &Statement,
    ) -> Result<(), Error> {
        #[cfg(test)]
        println!("emitting statement {stmt:?}");
        match stmt {
            Statement::Emit(range) => {
                for handle in range.clone() {
                    self.eval_cached_expr(function, handle)?;
                }
            }
            Statement::Block(stmts) => {
                for stmt in stmts {
                    self.emit_statement(span, function, stmt)?;
                }
            }
            Statement::If { .. } => todo!(),
            Statement::Switch { .. } => todo!(),
            Statement::Loop { .. } => todo!(),
            Statement::Break => todo!(),
            Statement::Continue => todo!(),
            &Statement::Return { value } => match (&function.src_function.result, value) {
                (&Some(FunctionResult { ty: result_ty, .. }), Some(value)) => unsafe {
                    let (result, ty) = self.eval_cached_expr(function, value)?;
                    if result_ty != ty {
                        return Err(Error {
                            info: Info::IncorrectReturnType {
                                expected: Some(result_ty),
                                found: Some(ty),
                            },
                            span: function.src_function.expressions.get_span(value),
                        });
                    }
                    LLVMBuildRet(self.builder.0, result);
                },
                (None, None) => unsafe {
                    LLVMBuildRetVoid(self.builder.0);
                },
                (&Some(FunctionResult { ty: result_ty, .. }), None) => {
                    return Err(Error {
                        info: Info::IncorrectReturnType {
                            expected: Some(result_ty),
                            found: None,
                        },
                        span,
                    })
                }
                (None, Some(value)) => {
                    let (_, ty) = self.eval_cached_expr(function, value)?;
                    return Err(Error {
                        info: Info::IncorrectReturnType {
                            expected: None,
                            found: Some(ty),
                        },
                        span,
                    });
                }
            },
            Statement::Kill => todo!(),
            Statement::Barrier(_) => todo!(),
            &Statement::Store { pointer, value } => {
                let (pointer, _) = self.eval_cached_expr(function, pointer)?;
                self.build_store(function, span, pointer, value)?;
            }
            Statement::ImageStore { .. } => todo!(),
            Statement::Atomic { .. } => todo!(),
            Statement::WorkGroupUniformLoad { .. } => todo!(),
            Statement::Call { .. } => todo!(),
            Statement::RayQuery { .. } => todo!(),
        }
        Ok(())
    }
}
