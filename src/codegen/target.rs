use std::ffi::CStr;

use llvm_sys::{
    core::LLVMDisposeMessage,
    target::{LLVMOpaqueTargetData, LLVM_InitializeNativeTarget},
    target_machine::{
        LLVMCodeGenOptLevel, LLVMCodeModel, LLVMCreateTargetDataLayout, LLVMCreateTargetMachine,
        LLVMGetDefaultTargetTriple, LLVMGetHostCPUFeatures, LLVMGetHostCPUName,
        LLVMGetTargetFromTriple, LLVMOpaqueTargetMachine, LLVMRelocMode, LLVMTarget,
    },
};

pub(super) struct TargetInfo {
    pub(super) _target: *mut LLVMTarget,
    pub(super) _machine: *mut LLVMOpaqueTargetMachine,
    pub(super) data_layout: *mut LLVMOpaqueTargetData,
}
unsafe impl Send for TargetInfo {}
impl TargetInfo {
    pub fn native() -> Self {
        assert!(
            unsafe { LLVM_InitializeNativeTarget() } == 0,
            "No native target to compile for"
        );

        let triple = unsafe { LLVMGetDefaultTargetTriple() };
        let mut target = core::ptr::null_mut();
        let mut error = core::ptr::null_mut();
        if unsafe { LLVMGetTargetFromTriple(triple, &mut target, &mut error) } != 0 {
            let c_str = unsafe { CStr::from_ptr(error) };
            let string = c_str.to_string_lossy().to_string();
            unsafe { LLVMDisposeMessage(error) };
            panic!("LLVM Error: {string}");
        }

        let machine = unsafe {
            LLVMCreateTargetMachine(
                target,
                triple,
                LLVMGetHostCPUName(),
                LLVMGetHostCPUFeatures(),
                LLVMCodeGenOptLevel::LLVMCodeGenLevelAggressive,
                LLVMRelocMode::LLVMRelocDefault,
                LLVMCodeModel::LLVMCodeModelDefault,
            )
        };

        let data_layout = unsafe { LLVMCreateTargetDataLayout(machine) };

        Self {
            _target: target,
            _machine: machine,
            data_layout,
        }
    }
}
