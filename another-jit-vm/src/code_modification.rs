use std::sync::{Mutex, MutexGuard};

use libc::c_void;

pub struct CodeModificationHandle<'l>(MutexGuard<'l, ()>);

impl Drop for CodeModificationHandle<'_> {
    fn drop(&mut self) {
        unsafe {
            core::arch::x86_64::__cpuid_count(0, 0);
        }
    }
}

pub struct GlobalCodeEditingLock(Mutex<()>);

impl GlobalCodeEditingLock {
    pub fn new() -> Self {
        Self(Mutex::new(()))
    }

    pub fn acquire(&self) -> CodeModificationHandle {
        CodeModificationHandle(self.0.lock().unwrap())
    }
}

impl CodeModificationHandle<'_> {
    pub unsafe fn edit_code_at(&self, location: *mut u64, new_val: u64) {
        location.write(new_val);
        //todo still needs to be xchg b/c self modifying code
        // unsafe { atomic_xchg(location, new_val); }
    }
}


#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct FunctionCallTarget(pub *mut *const c_void);

pub struct AssemblerFunctionCallTarget {
    pub modification_target: AssemblerRuntimeModificationTarget,
    pub method_id: usize,
}


pub enum AssemblerRuntimeModificationTarget {
    MovQ {
        instruction_number: usize,
    }
}

