use std::sync::Arc;

use classfile_view::view::HasAccessFlags;
use rust_jvm_common::descriptor_parser::parse_method_descriptor;

use crate::{InterpreterStateGuard, JVMState};
use crate::instructions::invoke::special::invoke_special_impl;
use crate::interpreter::WasException;
use crate::java_values::{default_value, JavaValue, Object};
use crate::runtime_class::RuntimeClass;

//todo jni should really live in interpreter state

pub fn push_new_object<'gc_life>(
    jvm: &'_ JVMState<'gc_life>,
    int_state: &'_ mut InterpreterStateGuard<'gc_life, '_>,
    runtime_class: &'_ Arc<RuntimeClass<'gc_life>>,
) {
    let object_pointer = JavaValue::new_object(jvm, runtime_class.clone());
    let new_obj = JavaValue::Object(todo!()/*object_pointer.clone()*/);
    let loader = jvm.classes.read().unwrap().get_initiating_loader(runtime_class);
    default_init_fields(jvm, &object_pointer.as_ref().unwrap().unwrap_normal_object().objinfo.class_pointer, &object_pointer.clone().unwrap());
    int_state.current_frame_mut().push(new_obj);
}

fn default_init_fields<'gc_life>(
    jvm: &'_ JVMState<'gc_life>,
    current_class_pointer: &'gc_life Arc<RuntimeClass<'gc_life>>,
    object_pointer: &Arc<Object<'gc_life>>) {
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
            let val = default_value(type_.clone());

            object_pointer.unwrap_normal_object().set_var(current_class_pointer.clone(), field.field_name(), val, type_);
            // unsafe {
            // *object_pointer.fields.get(&name).unwrap().get().as_mut().unwrap() = val;
            // }
        }
    }
}

pub fn run_constructor<'gc_life>(
    jvm: &'_ JVMState<'gc_life>,
    int_state: &'_ mut InterpreterStateGuard<'gc_life, '_>,
    target_classfile: Arc<RuntimeClass<'gc_life>>,
    full_args: Vec<JavaValue<'gc_life>>,
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
    invoke_special_impl(jvm, int_state, &md, method_view.method_i(), target_classfile.clone())
}