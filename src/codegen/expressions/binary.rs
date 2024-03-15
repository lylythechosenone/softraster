use std::ffi::c_uint;

use llvm_sys::{
    core::{
        LLVMBuildAShr, LLVMBuildAdd, LLVMBuildAnd, LLVMBuildExtractValue, LLVMBuildFAdd,
        LLVMBuildFCmp, LLVMBuildFDiv, LLVMBuildFMul, LLVMBuildFRem, LLVMBuildFSub, LLVMBuildICmp,
        LLVMBuildInsertValue, LLVMBuildLShr, LLVMBuildMul, LLVMBuildOr, LLVMBuildSDiv,
        LLVMBuildSRem, LLVMBuildShl, LLVMBuildSub, LLVMBuildUDiv, LLVMBuildURem, LLVMBuildXor,
        LLVMGetUndef, LLVMTypeOf, LLVMVectorType,
    },
    LLVMIntPredicate, LLVMRealPredicate, LLVMValue,
};
use naga::{BinaryOperator, Expression, Handle, Scalar, ScalarKind, Span, Type, TypeInner};

use crate::codegen::{
    error::{Error, Info},
    Generator, EMPTY_CSTR,
};

use super::Location;

macro_rules! component_wise_op {
    ($op:expr; $self:expr, $location:expr, ($lhs:expr, $ty:expr), ($rhs:expr, $rhs_ty:expr), $span:expr; $($kind:pat => $function:expr),* $(,)?) => {{
        if $ty != $rhs_ty {
            return Err(Error {
                info: Info::IncompatibleBinary {
                    operator: $op,
                    lhs: $ty,
                    rhs: Some($rhs_ty),
                },
                span: $span,
            });
        }
        match $self.module.src.types[$ty].inner {
            $(
                TypeInner::Scalar(Scalar {
                    kind: $kind,
                    ..
                })
                | TypeInner::Vector {
                    scalar:
                        Scalar {
                            kind: $kind,
                            ..
                        },
                    ..
                } => Ok((
                    #[allow(clippy::redundant_closure_call)]
                    unsafe { $function($self.builder.0, $lhs, $rhs, EMPTY_CSTR) },
                    $ty,
                )),
            )*
            #[allow(clippy::unnested_or_patterns)]
            TypeInner::Matrix {
                columns,
                scalar:
                    scalar @ Scalar {
                        kind: $($kind)|*,
                        ..
                    },
                ..
            } => {
                let mut result_matrix = unsafe { LLVMGetUndef($self.eval_cached_type($span, $ty)?) };

                for column in 0..(columns as c_uint) {
                    let lhs =
                        unsafe { LLVMBuildExtractValue($self.builder.0, $lhs, column, EMPTY_CSTR) };
                    let rhs =
                        unsafe { LLVMBuildExtractValue($self.builder.0, $rhs, column, EMPTY_CSTR) };

                    let result = match scalar.kind {
                        $(
                            $kind => unsafe {
                                #[allow(clippy::redundant_closure_call)]
                                $function($self.builder.0, lhs, rhs, EMPTY_CSTR)
                            },
                        )*
                        _ => unreachable!(),
                    };

                    result_matrix = unsafe {
                        LLVMBuildInsertValue(
                        $self.builder.0,
                            result_matrix,
                            result,
                            column,
                            EMPTY_CSTR,
                        )
                    };
                }

                Ok((result_matrix, $ty))
            }
            TypeInner::Scalar(scalar@Scalar {
                kind: ScalarKind::AbstractFloat | ScalarKind::AbstractInt,
                ..
            })
            | TypeInner::Vector {
                scalar:
                    scalar@Scalar {
                        kind: ScalarKind::AbstractFloat | ScalarKind::AbstractInt,
                        ..
                    },
                ..
            } | TypeInner::Matrix {
                scalar:
                    scalar@Scalar {
                        kind: ScalarKind::AbstractFloat | ScalarKind::AbstractInt,
                        ..
                    },
                ..
            } => Err(Error {
                info: Info::IllegalScalar(scalar),
                span: $span,
            }),
            _ => Err(Error {
                info: Info::IncompatibleBinary {
                    operator: BinaryOperator::Add,
                    lhs: $ty,
                    rhs: None,
                },
                span: $span,
            }),
        }
    }};
    ($op:expr; $self:expr, $location:expr, $left:expr, $right:expr, $span:expr; $($kind:pat => $function:expr),* $(,)?) => {{
        let (lhs, ty) = $self.eval_cached_expr($location, $left)?;
        let (rhs, rhs_ty) = $self.eval_cached_expr($location, $right)?;
        component_wise_op!($op; $self, $location, (lhs, ty), (rhs, rhs_ty), $span; $($kind => $function),*)
    }};
}

impl Generator {
    #[allow(clippy::too_many_arguments)]
    fn build_matrix_mul(
        &mut self,
        span: Span,
        scalar: Scalar,
        lhs: *mut LLVMValue,
        rhs: *mut LLVMValue,
        rows: u32,
        inner: u32,
        cols: u32,
        result_ty: Handle<Type>,
    ) -> Result<(*mut LLVMValue, Handle<Type>), Error> {
        let llvm_result_ty = unsafe {
            LLVMVectorType(
                self.eval_scalar_type(span, scalar)?,
                rows as c_uint * cols as c_uint,
            )
        };

        let args = [
            lhs,
            rhs,
            self.const_u32(rows),
            self.const_u32(inner),
            self.const_u32(cols),
        ];
        let result = self.build_intrinsic_call(
            "llvm.matrix.multiply",
            &[llvm_result_ty, unsafe { LLVMTypeOf(lhs) }, unsafe {
                LLVMTypeOf(rhs)
            }],
            &args,
        );

        Ok((result, result_ty))
    }

    #[allow(clippy::too_many_lines)]
    fn build_mul<L: Location>(
        &mut self,
        location: &L,
        span: Span,
        left: Handle<Expression>,
        right: Handle<Expression>,
    ) -> Result<(*mut LLVMValue, Handle<Type>), Error> {
        let (lhs, lhs_ty) = self.eval_cached_expr(location, left)?;
        let (rhs, rhs_ty) = self.eval_cached_expr(location, right)?;

        match (
            &self.module.src.types[lhs_ty].inner,
            &self.module.src.types[rhs_ty].inner,
        ) {
            (
                &TypeInner::Matrix {
                    rows: outer_rows,
                    columns: inner,
                    scalar,
                },
                &TypeInner::Matrix {
                    rows: rhs_rows,
                    columns: outer_cols,
                    scalar: rhs_scalar,
                },
            ) if rhs_rows == inner && scalar == rhs_scalar => {
                let result_ty = self.module.src.types.insert(
                    Type {
                        name: None,
                        inner: TypeInner::Matrix {
                            columns: outer_cols,
                            rows: outer_rows,
                            scalar,
                        },
                    },
                    span,
                );
                self.build_matrix_mul(
                    span,
                    scalar,
                    lhs,
                    rhs,
                    outer_rows as u32,
                    inner as u32,
                    outer_cols as u32,
                    result_ty,
                )
            }
            (
                &TypeInner::Matrix {
                    rows: outer_rows,
                    columns: inner,
                    scalar,
                },
                &TypeInner::Vector {
                    size: rhs_rows,
                    scalar: rhs_scalar,
                },
            ) if rhs_rows == inner && scalar == rhs_scalar => self.build_matrix_mul(
                span,
                scalar,
                lhs,
                rhs,
                outer_rows as u32,
                inner as u32,
                1,
                rhs_ty,
            ),
            (
                &TypeInner::Vector {
                    size: inner,
                    scalar,
                },
                &TypeInner::Matrix {
                    rows: rhs_rows,
                    columns: outer_cols,
                    scalar: rhs_scalar,
                },
            ) if rhs_rows == inner && scalar == rhs_scalar => {
                let result_ty = self.module.src.types.insert(
                    Type {
                        name: None,
                        inner: TypeInner::Vector {
                            size: outer_cols,
                            scalar,
                        },
                    },
                    self.module.src.types.get_span(lhs_ty),
                );

                self.build_matrix_mul(
                    span,
                    scalar,
                    lhs,
                    rhs,
                    1,
                    inner as u32,
                    outer_cols as u32,
                    result_ty,
                )
            }
            _ => component_wise_op!(
                BinaryOperator::Multiply;
                self,
                location,
                (lhs, lhs_ty),
                (rhs, rhs_ty),
                span;
                ScalarKind::Float => LLVMBuildFMul,
                ScalarKind::Sint | ScalarKind::Uint => LLVMBuildMul,
            ),
        }
    }

    #[allow(clippy::too_many_lines)]
    pub(super) fn build_binary<L: Location>(
        &mut self,
        location: &L,
        span: Span,
        op: BinaryOperator,
        left: Handle<Expression>,
        right: Handle<Expression>,
    ) -> Result<(*mut LLVMValue, Handle<Type>), Error> {
        match op {
            BinaryOperator::Add => component_wise_op!(
                BinaryOperator::Add; self, location, left, right, span;
                ScalarKind::Float => LLVMBuildFAdd,
                ScalarKind::Sint | ScalarKind::Uint => LLVMBuildAdd,
            ),
            BinaryOperator::Subtract => component_wise_op!(
                BinaryOperator::Subtract; self, location, left, right, span;
                ScalarKind::Float => LLVMBuildFSub,
                ScalarKind::Sint | ScalarKind::Uint => LLVMBuildSub,
            ),
            BinaryOperator::Multiply => self.build_mul(location, span, left, right),
            BinaryOperator::Divide => component_wise_op!(
                BinaryOperator::Divide; self, location, left, right, span;
                ScalarKind::Float => LLVMBuildFDiv,
                ScalarKind::Sint => LLVMBuildSDiv,
                ScalarKind::Uint => LLVMBuildUDiv,
            ),
            BinaryOperator::Modulo => component_wise_op!(
                BinaryOperator::Modulo; self, location, left, right, span;
                ScalarKind::Float => LLVMBuildFRem,
                ScalarKind::Sint => LLVMBuildSRem,
                ScalarKind::Uint => LLVMBuildURem,
            ),
            BinaryOperator::Equal => component_wise_op!(
                BinaryOperator::Equal; self, location, left, right, span;
                ScalarKind::Sint | ScalarKind::Uint | ScalarKind::Bool =>
                    |builder, lhs, rhs, name| LLVMBuildICmp(builder, LLVMIntPredicate::LLVMIntEQ, lhs, rhs, name),
                ScalarKind::Float =>
                    |builder, lhs, rhs, name| LLVMBuildFCmp(builder, LLVMRealPredicate::LLVMRealOEQ, lhs, rhs, name),
            ),
            BinaryOperator::NotEqual => component_wise_op!(
                BinaryOperator::NotEqual; self, location, left, right, span;
                ScalarKind::Sint | ScalarKind::Uint | ScalarKind::Bool =>
                    |builder, lhs, rhs, name| LLVMBuildICmp(builder, LLVMIntPredicate::LLVMIntNE, lhs, rhs, name),
                ScalarKind::Float =>
                    |builder, lhs, rhs, name| LLVMBuildFCmp(builder, LLVMRealPredicate::LLVMRealUNE, lhs, rhs, name),
            ),
            BinaryOperator::Less => component_wise_op!(
                BinaryOperator::Less; self, location, left, right, span;
                ScalarKind::Uint | ScalarKind::Bool =>
                    |builder, lhs, rhs, name| LLVMBuildICmp(builder, LLVMIntPredicate::LLVMIntULT, lhs, rhs, name),
                ScalarKind::Sint =>
                    |builder, lhs, rhs, name| LLVMBuildICmp(builder, LLVMIntPredicate::LLVMIntSLT, lhs, rhs, name),
                ScalarKind::Float =>
                    |builder, lhs, rhs, name| LLVMBuildFCmp(builder, LLVMRealPredicate::LLVMRealOLT, lhs, rhs, name),
            ),
            BinaryOperator::LessEqual => component_wise_op!(
                BinaryOperator::LessEqual; self, location, left, right, span;
                ScalarKind::Uint | ScalarKind::Bool =>
                    |builder, lhs, rhs, name| LLVMBuildICmp(builder, LLVMIntPredicate::LLVMIntULE, lhs, rhs, name),
                ScalarKind::Sint =>
                    |builder, lhs, rhs, name| LLVMBuildICmp(builder, LLVMIntPredicate::LLVMIntSLE, lhs, rhs, name),
                ScalarKind::Float =>
                    |builder, lhs, rhs, name| LLVMBuildFCmp(builder, LLVMRealPredicate::LLVMRealOLE, lhs, rhs, name),
            ),
            BinaryOperator::Greater => component_wise_op!(
                BinaryOperator::Greater; self, location, left, right, span;
                ScalarKind::Uint | ScalarKind::Bool =>
                    |builder, lhs, rhs, name| LLVMBuildICmp(builder, LLVMIntPredicate::LLVMIntUGT, lhs, rhs, name),
                ScalarKind::Sint =>
                    |builder, lhs, rhs, name| LLVMBuildICmp(builder, LLVMIntPredicate::LLVMIntSGT, lhs, rhs, name),
                ScalarKind::Float =>
                    |builder, lhs, rhs, name| LLVMBuildFCmp(builder, LLVMRealPredicate::LLVMRealOGT, lhs, rhs, name),
            ),
            BinaryOperator::GreaterEqual => component_wise_op!(
                BinaryOperator::GreaterEqual; self, location, left, right, span;
                ScalarKind::Uint | ScalarKind::Bool =>
                    |builder, lhs, rhs, name| LLVMBuildICmp(builder, LLVMIntPredicate::LLVMIntUGE, lhs, rhs, name),
                ScalarKind::Sint =>
                    |builder, lhs, rhs, name| LLVMBuildICmp(builder, LLVMIntPredicate::LLVMIntSGE, lhs, rhs, name),
                ScalarKind::Float =>
                    |builder, lhs, rhs, name| LLVMBuildFCmp(builder, LLVMRealPredicate::LLVMRealOGE, lhs, rhs, name),
            ),
            BinaryOperator::And => component_wise_op!(
                BinaryOperator::And; self, location, left, right, span;
                ScalarKind::Sint | ScalarKind::Uint => LLVMBuildAnd,
            ),
            BinaryOperator::ExclusiveOr => component_wise_op!(
                BinaryOperator::ExclusiveOr; self, location, left, right, span;
                ScalarKind::Sint | ScalarKind::Uint => LLVMBuildXor,
            ),
            BinaryOperator::InclusiveOr => component_wise_op!(
                BinaryOperator::InclusiveOr; self, location, left, right, span;
                ScalarKind::Sint | ScalarKind::Uint => LLVMBuildOr,
            ),
            BinaryOperator::LogicalAnd => component_wise_op!(
                BinaryOperator::LogicalAnd; self, location, left, right, span;
                ScalarKind::Bool => LLVMBuildAnd,
            ),
            BinaryOperator::LogicalOr => component_wise_op!(
                BinaryOperator::LogicalOr; self, location, left, right, span;
                ScalarKind::Bool => LLVMBuildOr,
            ),
            BinaryOperator::ShiftLeft => component_wise_op!(
                BinaryOperator::ShiftLeft; self, location, left, right, span;
                ScalarKind::Sint | ScalarKind::Uint => LLVMBuildShl,
            ),
            BinaryOperator::ShiftRight => component_wise_op!(
                BinaryOperator::ShiftLeft; self, location, left, right, span;
                ScalarKind::Uint => LLVMBuildLShr,
                ScalarKind::Sint => LLVMBuildAShr,
            ),
        }
    }
}
