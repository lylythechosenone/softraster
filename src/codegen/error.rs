use naga::{BinaryOperator, Handle, Literal, MathFunction, Scalar, Span, Type};

#[derive(PartialEq, Debug)]
pub struct Error {
    pub info: Info,
    pub span: Span,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Info {
    IncompatibleBinary {
        operator: BinaryOperator,
        lhs: Handle<Type>,
        rhs: Option<Handle<Type>>,
    },
    IncompatibleMath {
        function: MathFunction,
        lhs: Handle<Type>,
        rhs: Option<Handle<Type>>,
    },
    NonConstantIndex,
    NonConstantGlobal,
    IllegalScalar(Scalar),
    IllegalLiteral(Literal),
    InvalidComponent(Handle<Type>),
    NotComposable(Handle<Type>),
    NotIndexable(Handle<Type>),
    InvalidFloatSize(u8),
    NotPointer(Handle<Type>),
    NonVectorSwizzle(Handle<Type>),
    NoSuchComponent(Handle<Type>, usize),
    IncorrectReturnType {
        expected: Option<Handle<Type>>,
        found: Option<Handle<Type>>,
    },
    DynamicArrayWithoutPointer,
}
