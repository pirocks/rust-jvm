use std::ops::Deref;
use std::sync::Arc;

use classfile_view::view::{ClassView, HasAccessFlags};
use rust_jvm_common::classfile::ObjectVariableInfo;
use rust_jvm_common::descriptor_parser::parse_method_descriptor;

use crate::{InterpreterStateGuard, JVMState};
use crate::instructions::invoke::special::invoke_special_impl;
use crate::interpreter::WasException;
use crate::java_values::{default_value, JavaValue, ObjectFieldsAndClass};
use crate::runtime_class::RuntimeClass;

//todo jni should really live in interpreter state

pub fn push_new_object(
    jvm: &JVMState,
    int_state: &mut InterpreterStateGuard,
    runtime_class: &Arc<RuntimeClass>
) {
    let object_pointer = JavaValue::new_object(jvm, runtime_class.clone());
    let new_obj = JavaValue::Object(object_pointer.clone());
    let loader = jvm.classes.read().unwrap().get_initiating_loader(runtime_class);
    default_init_fields(jvm, &object_pointer.unwrap().unwrap_normal_object().objinfo);
    int_state.current_frame_mut().push(new_obj);
}

fn default_init_fields(
    jvm: &JVMState,
    object_pointer: &ObjectFieldsAndClass) {
    if let Some(super_) = object_pointer.parent.as_ref() {
        default_init_fields(jvm, super_.deref());
    }
    for field in object_pointer.class_pointer.view().fields() {
        if !field.is_static() {
            //todo should I look for constant val attributes?
            /*let _value_i = match field.constant_value_attribute() {
                None => {}
                Some(_i) => _i,
            };*/
            let name = field.field_name();
            let type_ = field.field_type();
            let val = default_value(type_);
            unsafe {
                *object_pointer.fields.get(&name).unwrap().get().as_mut().unwrap() = val;
            }
        }
    }
}

pub fn run_constructor(
    state: &JVMState,
    int_state: &mut InterpreterStateGuard,
    target_classfile: Arc<RuntimeClass>,
    full_args: Vec<JavaValue>,
    descriptor: String,
) -> Result<(), WasException> {
    let target_classfile_view = target_classfile.view();
    let method_view = target_classfile_view.lookup_method(&"<init>".to_string(), &parse_method_descriptor(descriptor.as_str()).unwrap()).unwrap();
    let md = method_view.desc();
    let this_ptr = full_args[0].clone();
    let actual_args = &full_args[1..];
    int_state.push_current_operand_stack(this_ptr);
    for arg in actual_args {
        int_state.push_current_operand_stack(arg.clone());
    }
    invoke_special_impl(state, int_state, &md, method_view.method_i(), target_classfile.clone())
}