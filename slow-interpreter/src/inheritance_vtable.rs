use std::collections::HashMap;
use std::ffi::c_void;
use std::ops::Deref;
use another_jit_vm_ir::IRMethodID;
use gc_memory_layout_common::{AllocatedObjectType, AllocatedTypeID};
use rust_jvm_common::{InheritanceMethodID, MethodId};
use rust_jvm_common::loading::LoaderName;
use threads::signal::kill;
use crate::jit::state::runtime_class_to_allocated_object_type;
use crate::JVMState;

#[derive(Clone, Debug)]
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
    pub fn new() -> Self{
        Self{
            table: Default::default()
        }
    }

    pub fn notify_compile_or_recompile(&mut self, jvm: &'gc_life JVMState<'gc_life>, method_id: MethodId, resolved: ResolvedInvokeVirtual){
        let (class,_) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
        let inheritance_method_id = jvm.inheritance_ids.read().unwrap().lookup(jvm,method_id);
        let allocated_object_type = runtime_class_to_allocated_object_type(class.deref(), LoaderName::BootstrapLoader, None, jvm.thread_state.get_current_thread_tid_or_invalid());//todo loader and thread id
        let allocated_object_id = jvm.gc.memory_region.lock().unwrap().lookup_or_add_type(&allocated_object_type);
        self.table.entry(allocated_object_id).or_default().insert(inheritance_method_id,resolved);
    }

    pub fn lookup_resolved(&self, runtime_type: AllocatedTypeID, inheritance_method_id: InheritanceMethodID) -> ResolvedInvokeVirtual{
        self.table.get(&runtime_type).unwrap().get(&inheritance_method_id).unwrap().clone()
    }
}