use std::ffi::c_uint;

use llvm_sys::{
    core::{
        LLVMBuildCall2, LLVMConstInt, LLVMGetIntrinsicDeclaration, LLVMInt1TypeInContext,
        LLVMInt32TypeInContext, LLVMInt64TypeInContext, LLVMIntrinsicGetType,
        LLVMLookupIntrinsicID,
    },
    LLVMType, LLVMValue,
};

use super::{Generator, EMPTY_CSTR};

impl Generator {
    pub(super) fn const_u64(&mut self, val: u64) -> *mut LLVMValue {
        unsafe { LLVMConstInt(LLVMInt64TypeInContext(self.module.context), val, 0) }
    }
    pub(super) fn const_u32(&mut self, val: u32) -> *mut LLVMValue {
        unsafe { LLVMConstInt(LLVMInt32TypeInContext(self.module.context), val as u64, 0) }
    }
    pub(super) fn const_i64(&mut self, val: i64) -> *mut LLVMValue {
        unsafe { LLVMConstInt(LLVMInt64TypeInContext(self.module.context), val as u64, 1) }
    }
    pub(super) fn const_i32(&mut self, val: i32) -> *mut LLVMValue {
        unsafe { LLVMConstInt(LLVMInt32TypeInContext(self.module.context), val as u64, 1) }
    }
    pub(super) fn const_bool(&mut self, val: bool) -> *mut LLVMValue {
        unsafe { LLVMConstInt(LLVMInt1TypeInContext(self.module.context), val as u64, 0) }
    }

    pub(super) fn build_intrinsic_call(
        &mut self,
        name: &str,
        types: &[*mut LLVMType],
        args: &[*mut LLVMValue],
    ) -> *mut LLVMValue {
        let id = unsafe { LLVMLookupIntrinsicID(name.as_ptr().cast(), name.len()) };
        let decl = unsafe {
            LLVMGetIntrinsicDeclaration(
                self.module.module,
                id,
                types.as_ptr().cast_mut(),
                types.len(),
            )
        };
        let ty = unsafe {
            LLVMIntrinsicGetType(
                self.module.context,
                id,
                types.as_ptr().cast_mut(),
                types.len(),
            )
        };

        unsafe {
            LLVMBuildCall2(
                self.builder.0,
                ty,
                decl,
                args.as_ptr().cast_mut(),
                args.len() as c_uint,
                EMPTY_CSTR,
            )
        }
    }
}
