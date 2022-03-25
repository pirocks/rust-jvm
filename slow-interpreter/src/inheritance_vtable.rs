use std::collections::{HashMap, HashSet};
use std::ffi::c_void;
use std::ops::Deref;
use std::sync::Arc;

use another_jit_vm_ir::IRMethodID;
use classfile_view::view::HasAccessFlags;
use gc_memory_layout_common::{AllocatedTypeID};
use rust_jvm_common::{InheritanceMethodID, MethodId};
use rust_jvm_common::loading::LoaderName;

use crate::class_loading::assert_loaded_class;
use crate::jit::state::runtime_class_to_allocated_object_type;
use crate::JVMState;
use crate::runtime_class::RuntimeClass;

#[derive(Clone, Debug)]
pub struct ResolvedInvokeVirtual {
    pub address: *const c_void,
    pub ir_method_id: IRMethodID,
    pub method_id: MethodId,
    pub new_frame_size: usize,
}

#[derive(Clone, Debug)]
pub struct UnResolvedInvokeVirtual {
    methodid: MethodId,
}

pub struct VTables {
    //todo make into vecs later
    table: HashMap<AllocatedTypeID, HashMap<InheritanceMethodID, ResolvedInvokeVirtual>>,
    before_compilation_table: HashMap<AllocatedTypeID, HashMap<InheritanceMethodID, UnResolvedInvokeVirtual>>,
}

impl VTables {
    pub fn new() -> Self {
        Self {
            table: Default::default(),
            before_compilation_table: Default::default(),
        }
    }

    pub fn notify_load<'gc>(&mut self, jvm: &'gc JVMState<'gc>, rc: Arc<RuntimeClass<'gc>>) {
        if rc.cpdtype().is_array() || rc.cpdtype().is_primitive(){
            return;
        }
        let allocated_object_type = runtime_class_to_allocated_object_type(rc.deref(), LoaderName::BootstrapLoader, None);//todo loader and thread id
        let allocated_object_id = jvm.gc.memory_region.lock().unwrap().lookup_or_add_type(&allocated_object_type);
        self.notify_load_impl(jvm, rc, allocated_object_id)
    }

    fn notify_load_impl<'gc>(&mut self, jvm: &'gc JVMState<'gc>, rc: Arc<RuntimeClass<'gc>>, allocated_object_id: AllocatedTypeID) {
        let class_view = rc.view();
        for method in class_view.methods(){
            let method_view = class_view.method_view_i(method.method_i());
            if !method_view.is_static() {
                let method_id = jvm.method_table.write().unwrap().get_method_id(rc.clone(),method_view.method_i());
                let inheritance_method_id = todo!()/*jvm.inheritance_ids.read().unwrap().lookup(jvm, method_id)*/;
                self.before_compilation_table.entry(allocated_object_id).or_default().insert(inheritance_method_id, UnResolvedInvokeVirtual { methodid: method_id });
            }
        }
        if let Some(super_name) = class_view.super_name() {
            let super_rc = assert_loaded_class(jvm, super_name.into());
            self.notify_load_impl(jvm, super_rc, allocated_object_id);
        }
    }

    pub fn notify_compile_or_recompile<'gc>(&mut self, jvm: &'gc JVMState<'gc>, method_id: MethodId, resolved: ResolvedInvokeVirtual) {
        let (class, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
        let class_view = class.view();
        let method_view = class_view.method_view_i(method_i);
        if !method_view.is_static() {
            let inheritance_method_id = todo!()/*jvm.inheritance_ids.read().unwrap().lookup(jvm, method_id)*/;
            let allocated_object_type = runtime_class_to_allocated_object_type(class.deref(), LoaderName::BootstrapLoader, None);//todo loader and thread id
            let allocated_object_id = jvm.gc.memory_region.lock().unwrap().lookup_or_add_type(&allocated_object_type);
            self.table.entry(allocated_object_id).or_default().insert(inheritance_method_id, resolved);
        }
    }

    pub fn lookup_resolved(&self, runtime_type: AllocatedTypeID, inheritance_method_id: InheritanceMethodID) -> Result<ResolvedInvokeVirtual, NotCompiledYet> {
        match self.table.get(&runtime_type).unwrap().get(&inheritance_method_id) {
            Some(resolved) => {
                Ok(resolved.clone())
            }
            None => {
                let needs_compiling = self.before_compilation_table.get(&runtime_type).unwrap().get(&inheritance_method_id).unwrap();
                Err(NotCompiledYet { needs_compiling: needs_compiling.methodid })
            }
        }
    }

    pub fn lookup_all<'gc>(&self, jvm: &'gc JVMState<'gc>, inheritance_method_id: InheritanceMethodID) -> HashSet<String> {
        let mut res = HashSet::new();
        for (_allocated_type_id, resolved) in &self.table {
            if let Some(resolved) = resolved.get(&inheritance_method_id) {
                let method_id = resolved.method_id;
                res.insert(jvm.method_table.read().unwrap().lookup_method_string(method_id, &jvm.string_pool));
            }
        }
        res
    }
}

#[derive(Debug, Copy, Clone)]
pub struct NotCompiledYet {
    pub needs_compiling: MethodId,
}