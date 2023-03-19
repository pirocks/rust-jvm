use std::collections::HashSet;
use std::sync::Arc;

use itertools::Itertools;

use jvmti_jni_bindings::{jobject};
use runtime_class_stuff::RuntimeClass;
use rust_jvm_common::{MethodId};
use rust_jvm_common::loading::LoaderName;
use rust_jvm_common::opaque_id_table::OpaqueID;

use crate::java_values::{JavaValue};
use crate::jvm_state::JVMState;
use crate::NewJavaValue;


#[derive(Clone)]
pub struct JavaFramePush<'gc, 'k> {
    pub(crate) method_id: MethodId,
    pub(crate) local_vars: Vec<NewJavaValue<'gc, 'k>>,
    pub(crate) operand_stack: Vec<NewJavaValue<'gc, 'k>>,
}

#[derive(Clone)]
pub struct NativeFramePush<'gc, 'k> {
    pub(crate) method_id: MethodId,
    pub(crate) native_local_refs: Vec<HashSet<jobject>>,
    pub(crate) local_vars: Vec<NewJavaValue<'gc, 'k>>,
    pub(crate) operand_stack: Vec<NewJavaValue<'gc, 'k>>,
}

#[derive(Clone)]
pub struct OpaqueFramePush {
    pub(crate) opaque_id: OpaqueID,
    pub(crate) native_local_refs: Vec<HashSet<jobject>>,
}

#[derive(Clone)]
pub enum StackEntryPush<'gc, 'k> {
    Java(JavaFramePush<'gc, 'k>),
    // a native function call frame
    Native(NativeFramePush<'gc, 'k>),
    Opaque(OpaqueFramePush),
}

impl<'gc, 'k> StackEntryPush<'gc, 'k> {
    pub fn new_native_frame(jvm: &'gc JVMState<'gc>, class_pointer: Arc<RuntimeClass<'gc>>, method_i: u16, args: Vec<NewJavaValue<'gc, 'k>>) -> NativeFramePush<'gc, 'k> {
        let method_id = jvm.method_table.write().unwrap().get_method_id(class_pointer, method_i);
        NativeFramePush {
            method_id,
            native_local_refs: vec![HashSet::new()],
            local_vars: args,
            operand_stack: vec![],
        }
    }

    pub fn new_java_frame(jvm: &'gc JVMState<'gc>, class_pointer: Arc<RuntimeClass<'gc>>, method_i: u16, args: Vec<NewJavaValue<'gc, 'k>>) -> JavaFramePush<'gc, 'k> {
        let max_locals = class_pointer.view().method_view_i(method_i).code_attribute().unwrap().max_locals;
        assert_eq!(args.len(), max_locals as usize);
        let method_id = jvm.method_table.write().unwrap().get_method_id(class_pointer.clone(), method_i);
        // assert!(jvm.java_vm_state.try_lookup_ir_method_id(OpaqueFrameIdOrMethodID::Method { method_id: method_id as u64 }).is_some());
        let _loader = jvm.classes.read().unwrap().get_initiating_loader(&class_pointer);
        let mut guard = jvm.method_table.write().unwrap();
        let _method_id = guard.get_method_id(class_pointer.clone(), method_i);
        let class_view = class_pointer.view();
        let method_view = class_view.method_view_i(method_i);
        let code = method_view.code_attribute().unwrap();
        let operand_stack = (0..code.max_stack).map(|_| NewJavaValue::Top).collect_vec();
        JavaFramePush {
            method_id,
            local_vars: args,
            operand_stack,
        }
    }

    pub fn new_completely_opaque_frame(jvm: &'gc JVMState<'gc>, loader: LoaderName, operand_stack: Vec<JavaValue<'gc>>, debug_str: &'static str) -> OpaqueFramePush {
        //need a better name here
        assert!(operand_stack.is_empty());
        assert_eq!(loader, LoaderName::BootstrapLoader);// loader should be set from thread loader for new threads
        let opaque_id = jvm.opaque_ids.write().unwrap().new_opaque_id(debug_str);
        OpaqueFramePush {
            opaque_id,
            native_local_refs: vec![],
        }
    }
}
