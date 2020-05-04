use crate::interpreter_util::check_inited_class;
use rust_jvm_common::classnames::{ClassName};
use std::sync::Arc;
use rust_jvm_common::classfile::Interface;
use std::ops::Deref;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use crate::java_values::JavaValue;
use crate::{JVMState, StackEntry};
use crate::runtime_class::RuntimeClass;
use crate::java_values::Object::{Array, Object};
use crate::java_values;
use descriptor_parser::parse_field_type;


pub fn arraylength(current_frame: & StackEntry) -> () {
    let array_o = current_frame.pop().unwrap_object().unwrap();
    let array = array_o.unwrap_array();
    current_frame.push(JavaValue::Int(array.elems.borrow().len() as i32));
}


pub fn invoke_checkcast(jvm: & JVMState, current_frame: & StackEntry, cp: u16) {
    let possibly_null = current_frame.pop().unwrap_object();
    if possibly_null.is_none() {
        current_frame.push(JavaValue::Object(possibly_null));
        return;
    }
    let object = possibly_null.unwrap();
    match object.deref(){
        Object(o) => {
            let view = &current_frame.class_pointer.view();
            let instance_of_class_name = view.constant_pool_view(cp as usize).unwrap_class().class_name().unwrap_name();
            let instanceof_class = check_inited_class(jvm, &instance_of_class_name, current_frame.class_pointer.loader(jvm).clone());
            let object_class = o.class_pointer.clone();
            if inherits_from(jvm, &object_class, &instanceof_class) {
                current_frame.push(JavaValue::Object(object.clone().into()));
                return;
            } else {
                // current_frame.print_stack_trace();
                unimplemented!()
            }
        },
        Array(a) => {
            let current_frame_class = &current_frame.class_pointer.view();
            let instance_of_class = current_frame_class
                .constant_pool_view(cp as usize)
                .unwrap_class()
                .class_name()
                .unwrap_name();
            let (should_be_empty, expected_type_wrapped) = parse_field_type( instance_of_class.get_referred_name().as_str()).unwrap();
            assert!(should_be_empty.is_empty());
            let expected_type = expected_type_wrapped.unwrap_array_type();
            let cast_succeeds = match &a.elem_type {
                PTypeView::Ref(_) => {
                    let actual_runtime_class = check_inited_class(jvm, &a.elem_type.unwrap_class_type(), current_frame.class_pointer.loader(jvm).clone());
                    let expected_runtime_class = check_inited_class(jvm, &expected_type.unwrap_class_type(), current_frame.class_pointer.loader(jvm).clone());
                    inherits_from(jvm, &actual_runtime_class, &expected_runtime_class)
                },
                _ => {
                    a.elem_type == PTypeView::from_ptype(&expected_type)
                }
            };
            if cast_succeeds{
                current_frame.push(JavaValue::Object(object.clone().into()));
                return;
            }else{
                unimplemented!()
            }
        }
    }
    //todo dup with instance off
}


pub fn invoke_instanceof(state: & JVMState, current_frame: & StackEntry, cp: u16) {
    let possibly_null = current_frame.pop().unwrap_object();
    if possibly_null.is_none() {
        current_frame.push(JavaValue::Int(0));
        return;
    }
    let unwrapped = possibly_null.unwrap();
    let view = &current_frame.class_pointer.view();
    let instance_of_class_type = view.constant_pool_view(cp as usize).unwrap_class().class_name();
    // assert!(instance_of_class_type.try_unwrap_name().is_none());
    instance_of_impl(state, current_frame, unwrapped, instance_of_class_type);
}

pub fn instance_of_impl(jvm: &JVMState, current_frame: &StackEntry, unwrapped: Arc<java_values::Object>, instance_of_class_type: ReferenceTypeView) {
    match unwrapped.deref() {
        Array(array) => {
            match instance_of_class_type {
                ReferenceTypeView::Class(instance_of_class_name) => {
                    if instance_of_class_name == ClassName::serializable() ||
                        instance_of_class_name == ClassName::cloneable() {
                        unimplemented!()//todo need to handle serializable and the like
                    } else {
                        current_frame.push(JavaValue::Int(0))
                    }
                },
                ReferenceTypeView::Array(a) => {
                    if a.deref() == &array.elem_type {
                        current_frame.push(JavaValue::Int(1))
                    }
                },
            }
        },
        Object(object) => {
            match instance_of_class_type {
                ReferenceTypeView::Class(instance_of_class_name) => {
                    let instanceof_class = check_inited_class(jvm, &instance_of_class_name, current_frame.class_pointer.loader(jvm).clone());
                    let object_class = object.class_pointer.clone();
                    if inherits_from(jvm, &object_class, &instanceof_class) {
                        current_frame.push(JavaValue::Int(1))
                    } else {
                        current_frame.push(JavaValue::Int(0))
                    }
                },
                ReferenceTypeView::Array(_) => current_frame.push(JavaValue::Int(0)),
            }
        },
    };
}

fn runtime_super_class(jvm: & JVMState, inherits: &Arc<RuntimeClass>) -> Option<Arc<RuntimeClass>> {
    if inherits.view().super_name().is_some() {
        Some(check_inited_class(jvm, &inherits.view().super_name().unwrap(), inherits.loader(jvm).clone()))
    } else {
        None
    }
}


fn runtime_interface_class(jvm: & JVMState, class_: &Arc<RuntimeClass>, i: Interface) -> Arc<RuntimeClass> {
    let intf_name = class_.view().constant_pool_view(i as usize).unwrap_class().class_name().unwrap_name();
    check_inited_class(jvm, &intf_name,  class_.loader(jvm).clone())
}

//todo this really shouldn't need state or Arc<RuntimeClass>
pub fn inherits_from(state: & JVMState, inherits: &Arc<RuntimeClass>, parent: &Arc<RuntimeClass>) -> bool {
    if &inherits.view().name() == &parent.view().name(){
        return true;
    }
    let interfaces_match = inherits.view().interfaces().enumerate().any(|(i,_)| {
        let interface = runtime_interface_class(state, inherits, i as u16);
        &interface.view().name() == &parent.view().name()
    });

    (match runtime_super_class(state, inherits) {
        None => false,
        Some(super_) => {
            //todo why is this not an impl function?
            &super_.view().name() == &parent.view().name() ||
                inherits_from(state, &super_, parent)
        }
    }) || interfaces_match
}