use std::collections::HashMap;
use libc::c_void;
use another_jit_vm::code_modification::{CodeModificationHandle, EditAction, FunctionCallTarget};
use rust_jvm_common::MethodId;
use std::ptr::NonNull;

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

    pub fn update_target(&self, method_id: MethodId, old_address: Option<NonNull<c_void>>, new_address: NonNull<c_void>, handle: CodeModificationHandle) {
        for targets in self.inner.get(&method_id) {
            for target in targets {
                let edit_action = EditAction {
                    from: old_address.map(|old_address|old_address.as_ptr() as u64),
                    to: new_address.as_ptr() as u64,
                    location: target.0 as *mut u64,
                };
                edit_action.do_edit(&handle);
            }
        }
    }
}