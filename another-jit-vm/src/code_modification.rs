use std::intrinsics::atomic_xchg;
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
        Self {
            0: Mutex::new(())
        }
    }

    pub fn acquire(&self) -> CodeModificationHandle {
        CodeModificationHandle(self.0.lock().unwrap())
    }
}

#[derive(Copy, Clone, Debug)]
pub struct EditAction {
    pub from: Option<u64>,
    pub to: u64,
    pub location: *mut u64,
}

impl EditAction {
    pub fn do_edit(&self, handle: &CodeModificationHandle) {
        let EditAction { from, to, location } = *self;
        handle.edit_code_at(location, to, from)
    }
}

impl CodeModificationHandle<'_> {
    fn edit_code_at(&self, location: *mut u64, new_val: u64, expected: Option<u64>) {
        unsafe {
            if let Some(expected) = expected {
                assert_eq!(location.read(), expected);
            }
            //todo make this lock free with expected values
            let old = atomic_xchg(location, new_val);
            if let Some(expected) = expected {
                assert_eq!(old, expected);
            }
        }
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

