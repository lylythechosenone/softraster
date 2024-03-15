use llvm_sys::LLVMValue;
use naga::{Arena, Expression, Handle, Type};

use crate::codegen::{FunctionState, Module};

pub(in super::super) trait Location {
    fn get_exprs<'a>(&'a self, module: &'a Module) -> &'a Arena<Expression>;
    fn get_locals(&self) -> &[(*mut LLVMValue, Handle<Type>)];
    fn get_params(&self) -> &[(*mut LLVMValue, Handle<Type>)];
}

impl Location for FunctionState {
    fn get_exprs<'a>(&'a self, _module: &'a Module) -> &'a Arena<Expression> {
        &self.src_function.expressions
    }
    fn get_locals(&self) -> &[(*mut LLVMValue, Handle<Type>)] {
        &self.locals
    }
    fn get_params(&self) -> &[(*mut LLVMValue, Handle<Type>)] {
        &self.params
    }
}

pub(in super::super) struct Global;
impl Location for Global {
    fn get_exprs<'a>(&'a self, module: &'a Module) -> &'a Arena<Expression> {
        &module.src.const_expressions
    }
    fn get_locals(&self) -> &[(*mut LLVMValue, Handle<Type>)] {
        panic!("attempted to access local variables in a global context");
    }
    fn get_params(&self) -> &[(*mut LLVMValue, Handle<Type>)] {
        panic!("attempted to access function parameters in a global context");
    }
}
