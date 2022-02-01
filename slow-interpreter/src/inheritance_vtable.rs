use std::collections::{HashMap, HashSet};
use std::ffi::c_void;
use std::ops::Deref;

use another_jit_vm_ir::IRMethodID;
use classfile_view::view::HasAccessFlags;
use gc_memory_layout_common::{AllocatedObjectType, AllocatedTypeID};
use rust_jvm_common::{InheritanceMethodID, MethodId};
use rust_jvm_common::loading::LoaderName;
use threads::signal::kill;

use crate::jit::state::runtime_class_to_allocated_object_type;
use crate::JVMState;

#[derive(Clone, Debug)]
pub struct ResolvedInvokeVirtual {
    pub address: *const c_void,
    pub ir_method_id: IRMethodID,
    pub method_id: MethodId,
    pub new_frame_size: usize,
}

pub struct VTables {
    //todo make into vecs later
    table: HashMap<AllocatedTypeID, HashMap<InheritanceMethodID, ResolvedInvokeVirtual>>,
}

impl VTables {
    pub fn new() -> Self {
        Self {
            table: Default::default()
        }
    }

    pub fn notify_compile_or_recompile(&mut self, jvm: &'gc_life JVMState<'gc_life>, method_id: MethodId, resolved: ResolvedInvokeVirtual) {
        let (class, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
        let class_view = class.view();
        let method_view = class_view.method_view_i(method_i);
        if !method_view.is_static() {
            let inheritance_method_id = jvm.inheritance_ids.read().unwrap().lookup(jvm, method_id);
            let allocated_object_type = runtime_class_to_allocated_object_type(class.deref(), LoaderName::BootstrapLoader, None, jvm.thread_state.get_current_thread_tid_or_invalid());//todo loader and thread id
            let allocated_object_id = jvm.gc.memory_region.lock().unwrap().lookup_or_add_type(&allocated_object_type);
            self.table.entry(allocated_object_id).or_default().insert(inheritance_method_id, resolved);
        }
    }

    pub fn lookup_resolved(&self, runtime_type: AllocatedTypeID, inheritance_method_id: InheritanceMethodID) -> Result<ResolvedInvokeVirtual,NotCompiledYet> {
        match self.table.get(&runtime_type).unwrap().get(&inheritance_method_id) {
            Some(resolved) => {
                Ok(resolved.clone())
            },
            None => {
                // dbg!(self.table.get(&runtime_type).unwrap());
                // dbg!(inheritance_method_id);
                Err(NotCompiledYet{})
            },
        }
    }

    pub fn lookup_all(&self, jvm: &'gc_life JVMState<'gc_life>, inheritance_method_id: InheritanceMethodID) -> HashSet<String> {
        let mut res = HashSet::new();
        for (_allocated_type_id, resolved) in &self.table{
            if let Some(resolved) = resolved.get(&inheritance_method_id) {
                let method_id = resolved.method_id;
                res.insert(jvm.method_table.read().unwrap().lookup_method_string(method_id, &jvm.string_pool));
            }
        }
        res
    }
}

#[derive(Debug, Copy, Clone)]
pub struct NotCompiledYet;