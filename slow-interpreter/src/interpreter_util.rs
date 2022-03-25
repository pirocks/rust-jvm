use std::sync::Arc;

use classfile_view::view::HasAccessFlags;
use rust_jvm_common::compressed_classfile::CMethodDescriptor;
use rust_jvm_common::compressed_classfile::names::MethodName;

use crate::{InterpreterStateGuard, JVMState, NewJavaValue};
use crate::class_loading::check_initing_or_inited_class;
use crate::instructions::invoke::special::invoke_special_impl;
use crate::interpreter::WasException;
use crate::java_values::{default_value, JavaValue};
use crate::runtime_class::RuntimeClass;
use std::convert::AsRef;
use crate::new_java_values::{AllocatedObject, AllocatedObjectHandle};

//todo jni should really live in interpreter state

pub fn new_object<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc,'l>, runtime_class: &'_ Arc<RuntimeClass<'gc>>) -> AllocatedObjectHandle<'gc> {
    check_initing_or_inited_class(jvm, int_state, runtime_class.cpdtype()).expect("todo");
    let object_handle = JavaValue::new_object(jvm, runtime_class.clone());
    let object_jv = object_handle.new_java_value();
    let _loader = jvm.classes.read().unwrap().get_initiating_loader(runtime_class);
    default_init_fields(jvm, &runtime_class, object_jv.unwrap_object_alloc().unwrap());
    object_handle
}

fn default_init_fields<'gc, 'k>(jvm: &'gc JVMState<'gc>, current_class_pointer: &Arc<RuntimeClass<'gc>>, object_pointer: AllocatedObject<'gc,'k>) {
    if let Some(super_) = current_class_pointer.unwrap_class_class().parent.as_ref() {
        default_init_fields(jvm, super_, object_pointer.clone());
    }
    for field in current_class_pointer.view().fields() {
        if !field.is_static() {
            //todo should I look for constant val attributes?
            /*let _value_i = match field.constant_value_attribute() {
                None => {}
                Some(_i) => _i,
            };*/
            let name = field.field_name();
            let type_ = field.field_type();
            let val = default_value(&type_);

            object_pointer.set_var(current_class_pointer, field.field_name(), val.as_njv());
            // unsafe {
            // *object_pointer.fields.get(&name).unwrap().get().as_mut().unwrap() = val;
            // }
        }
    }
}

pub fn run_constructor<'gc, 'l, 'k>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc,'l>, target_classfile: Arc<RuntimeClass<'gc>>, full_args: Vec<NewJavaValue<'gc,'k>>, descriptor: &CMethodDescriptor) -> Result<(), WasException> {
    let target_classfile_view = target_classfile.view();
    let method_view = target_classfile_view.lookup_method(MethodName::constructor_init(), descriptor).unwrap();
    let md = method_view.desc();
    let res = invoke_special_impl(jvm, int_state, md, method_view.method_i(), target_classfile.clone(), full_args)?;
    assert!(res.is_none());
    Ok(())
}