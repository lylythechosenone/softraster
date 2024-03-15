use std::{ffi::CStr, fmt::Debug, io::Write};

use llvm_sys::{
    bit_writer::LLVMWriteBitcodeToMemoryBuffer,
    core::{
        LLVMContextDispose, LLVMDisposeMemoryBuffer, LLVMDisposeMessage, LLVMGetBufferSize,
        LLVMGetBufferStart, LLVMPrintModuleToString,
    },
    LLVMContext, LLVMModule,
};
use naga::Module as SrcModule;

pub struct Module {
    pub(super) context: *mut LLVMContext,
    pub(super) module: *mut LLVMModule,
    pub(super) src: SrcModule,
}
impl Module {
    pub fn write_bitcode(&self, mut to: impl Write) -> std::io::Result<()> {
        let buffer = unsafe { LLVMWriteBitcodeToMemoryBuffer(self.module) };
        let ptr: *const u8 = unsafe { LLVMGetBufferStart(buffer) }.cast();
        let slice = unsafe { core::slice::from_raw_parts(ptr, LLVMGetBufferSize(buffer)) };

        to.write_all(slice)?;

        unsafe {
            LLVMDisposeMemoryBuffer(buffer);
        }

        Ok(())
    }

    pub fn write_string(&self, mut to: impl Write) -> std::io::Result<()> {
        let s = unsafe { LLVMPrintModuleToString(self.module) };
        let c_str = unsafe { CStr::from_ptr(s) };

        to.write_all(c_str.to_bytes())?;

        unsafe { LLVMDisposeMessage(s) }

        Ok(())
    }

    #[must_use]
    pub fn src(&self) -> &SrcModule {
        &self.src
    }
}
unsafe impl Send for Module {}
impl Drop for Module {
    fn drop(&mut self) {
        unsafe { LLVMContextDispose(self.context) }
    }
}
impl Debug for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = unsafe { LLVMPrintModuleToString(self.module) };
        let c_str = unsafe { CStr::from_ptr(s) };
        let as_str = c_str.to_str().map_err(|_| std::fmt::Error)?;

        f.write_str(as_str)?;

        unsafe { LLVMDisposeMessage(s) }

        Ok(())
    }
}

pub mod ext {
    use naga::Module as SrcModule;

    use super::Module;
    use crate::codegen::{error::Error, Generator};

    #[allow(clippy::module_name_repetitions)]
    pub trait CodegenExt: Sized {
        fn generate(self, file_name: &str) -> Result<Module, Error>;
    }
    impl CodegenExt for SrcModule {
        fn generate(self, file_name: &str) -> Result<Module, Error> {
            Generator::new(self, file_name).generate()
        }
    }
}
