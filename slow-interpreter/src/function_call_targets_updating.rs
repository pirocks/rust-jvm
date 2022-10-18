use std::collections::HashMap;
use std::ptr::NonNull;

use libc::c_void;

use another_jit_vm::code_modification::{CodeModificationHandle, FunctionCallTarget};
use rust_jvm_common::MethodId;

pub struct FunctionCallTargetsByFunction {
    inner: HashMap<MethodId, Vec<FunctionCallTarget>>,
}


impl FunctionCallTargetsByFunction {
    pub fn new() -> Self {
        Self {
            inner: Default::default()
        }
    }

    pub fn sink_targets(&mut self, targets: HashMap<MethodId, Vec<FunctionCallTarget>>) {
        for (method_id, target) in targets {
            self.inner.entry(method_id).or_default().extend_from_slice(target.as_slice());
        }
    }

    pub fn update_target(&self, method_id: MethodId, new_address: NonNull<c_void>, handle: CodeModificationHandle) {
        if let Some(targets) = self.inner.get(&method_id) {
            for target in targets {
                unsafe { handle.edit_code_at(target.0 as *mut u64, new_address.as_ptr() as u64) }
            }
        }
    }
}