use naga::{Expression, Handle};

#[derive(Clone, Copy)]
pub(super) enum Index {
    Dynamic(Handle<Expression>),
    Constant(usize),
}
