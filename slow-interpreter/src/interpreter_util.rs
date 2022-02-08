use std::sync::Arc;

use classfile_view::view::HasAccessFlags;
use rust_jvm_common::compressed_classfile::CMethodDescriptor;
use rust_jvm_common::compressed_classfile::names::MethodName;

use crate::{InterpreterStateGuard, JVMState, NewJavaValue};
use crate::class_loading::check_initing_or_inited_class;
use crate::instructions::invoke::special::invoke_special_impl;
use crate::interpreter::WasException;
use crate::java_values::{default_value, GcManagedObject, JavaValue};
use crate::runtime_class::RuntimeClass;
use std::convert::AsRef;
use crate::new_java_values::AllocatedObjectHandle;

//todo jni should really live in interpreter state

pub fn new_object<'gc_life, 'l>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life,'l>, runtime_class: &'_ Arc<RuntimeClass<'gc_life>>) -> AllocatedObjectHandle<'gc_life> {
    check_initing_or_inited_class(jvm, int_state, runtime_class.cpdtype()).expect("todo");
    let object_handle = JavaValue::new_object(jvm, runtime_class.clone());
    let object_jv = object_handle.new_java_value().to_jv();
    let _loader = jvm.classes.read().unwrap().get_initiating_loader(runtime_class);
    default_init_fields(jvm, &object_jv.unwrap_object_nonnull().unwrap_normal_object().objinfo.class_pointer, &object_jv.unwrap_object_nonnull().clone());
    object_handle
}

fn default_init_fields<'gc_life>(jvm: &'gc_life JVMState<'gc_life>, current_class_pointer: &Arc<RuntimeClass<'gc_life>>, object_pointer: &GcManagedObject<'gc_life>) {
    if let Some(super_) = current_class_pointer.unwrap_class_class().parent.as_ref() {
        default_init_fields(jvm, super_, object_pointer);
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
            let val = default_value(type_.clone()).to_jv();

            todo!()
            /*object_pointer.unwrap_normal_object().set_var(current_class_pointer.clone(), field.field_name(), val);*/
            // unsafe {
            // *object_pointer.fields.get(&name).unwrap().get().as_mut().unwrap() = val;
            // }
        }
    }
}

pub fn run_constructor<'gc_life, 'l>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life,'l>, target_classfile: Arc<RuntimeClass<'gc_life>>, full_args: Vec<JavaValue<'gc_life>>, descriptor: &CMethodDescriptor) -> Result<(), WasException> {
    let target_classfile_view = target_classfile.view();
    let method_view = target_classfile_view.lookup_method(MethodName::constructor_init(), descriptor).unwrap();
    let md = method_view.desc();
    let res = invoke_special_impl(jvm, int_state, md, method_view.method_i(), target_classfile.clone(), full_args)?;
    assert!(res.is_none());
    Ok(())
}