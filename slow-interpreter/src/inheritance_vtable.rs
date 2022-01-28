use std::collections::HashMap;
use std::ffi::c_void;
use another_jit_vm_ir::IRMethodID;
use gc_memory_layout_common::AllocatedTypeID;
use rust_jvm_common::MethodId;
use crate::inheritance_method_ids::InheritanceMethodID;
use crate::JVMState;

pub struct ResolvedInvokeVirtual{
    pub address: *const c_void,
    pub ir_method_id: IRMethodID,
    pub method_id: MethodId,
    pub new_frame_size: usize
}

pub struct VTables {
    //todo make into vecs later
    table: HashMap<AllocatedTypeID, HashMap<InheritanceMethodID, ResolvedInvokeVirtual>>
}

impl VTables {
    pub fn notify_compile_or_recompile(&mut self, jvm: &'gc_life JVMState<'gc_life>, method_id: MethodId, resolved: ResolvedInvokeVirtual){
        todo!()
    }

    pub fn lookup_resolved(&self, runtime_type: AllocatedTypeID, inheritance_method_id: InheritanceMethodID){
        todo!()
    }
}