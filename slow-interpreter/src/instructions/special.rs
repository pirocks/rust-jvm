use std::rc::Rc;
use crate::interpreter_util::check_inited_class;
use rust_jvm_common::classnames::{ClassName, class_name};
use std::sync::Arc;
use rust_jvm_common::classfile::Interface;
use std::ops::Deref;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use classfile_view::view::descriptor_parser::parse_field_type;
use crate::java_values::JavaValue;
use crate::{InterpreterState, StackEntry};
use crate::runtime_class::RuntimeClass;
use crate::java_values::Object::{Array, Object};


pub fn arraylength(current_frame: &Rc<StackEntry>) -> () {
    let array_o = current_frame.pop().unwrap_object().unwrap();
    let array = array_o.unwrap_array();
    current_frame.push(JavaValue::Int(array.elems.borrow().len() as i32));
}


pub fn invoke_checkcast(state: &mut InterpreterState, current_frame: &Rc<StackEntry>, cp: u16) {
    let possibly_null = current_frame.pop().unwrap_object();
    if possibly_null.is_none() {
        current_frame.push(JavaValue::Object(possibly_null));
        return;
    }
    let object = possibly_null.unwrap();
    match object.deref(){
        Object(o) => {
            let classfile = &current_frame.class_pointer.classfile;
            let instance_of_class_name = classfile.extract_class_from_constant_pool_name(cp);
            let instanceof_class = check_inited_class(state, &ClassName::Str(instance_of_class_name), current_frame.clone().into(), current_frame.class_pointer.loader.clone());
            let object_class = o.class_pointer.clone();
            if inherits_from(state, &object_class, &instanceof_class) {
                current_frame.push(JavaValue::Object(object.clone().into()));
                return;
            } else {
                current_frame.print_stack_trace();
                unimplemented!()
            }
        },
        Array(a) => {
            let current_frame_class = &current_frame.class_pointer.classfile;
            let instance_of_class_str = current_frame_class.extract_class_from_constant_pool_name(cp);
            let (should_be_empty, expected_type_wrapped) = parse_field_type( instance_of_class_str.as_str()).unwrap();
            assert!(should_be_empty.is_empty());
            let expected_type = expected_type_wrapped.unwrap_array_type();
            let cast_succeeds = match &a.elem_type {
                PTypeView::Ref(_) => {
                    let actual_runtime_class = check_inited_class(state,&a.elem_type.unwrap_class_type(),current_frame.clone().into(),current_frame.class_pointer.loader.clone());
                    let expected_runtime_class = check_inited_class(state,&expected_type.unwrap_class_type(),current_frame.clone().into(),current_frame.class_pointer.loader.clone());
                    inherits_from(state,&actual_runtime_class,&expected_runtime_class)
                },
                _ => {
                    a.elem_type == expected_type
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


pub fn invoke_instanceof(state: &mut InterpreterState, current_frame: &Rc<StackEntry>, cp: u16) {
    let possibly_null = current_frame.pop().unwrap_object();
    if possibly_null.is_none() {
        current_frame.push(JavaValue::Int(0));
        return;
    }
    let unwrapped = possibly_null.unwrap();
    let classfile = &current_frame.class_pointer.class_view;
    let instance_of_class_type = classfile.constant_pool_view(cp as usize).unwrap_class().class_name();
    // assert!(instance_of_class_type.try_unwrap_name().is_none());
    match unwrapped.deref(){
        Array(array) => {
            match instance_of_class_type {
                ReferenceTypeView::Class(instance_of_class_name) => {
                    if instance_of_class_name == ClassName::serializable() ||
                        instance_of_class_name == ClassName::cloneable(){
                        unimplemented!()//todo need to handle serializable and the like
                    }else {
                        current_frame.push(JavaValue::Int(0))
                    }
                },
                ReferenceTypeView::Array(a) => {
                    if a.deref() == &array.elem_type{
                        current_frame.push(JavaValue::Int(1))
                    }
                },
            }
        },
        Object(object) => {
            match instance_of_class_type {
                ReferenceTypeView::Class(instance_of_class_name) => {
                    let instanceof_class = check_inited_class(state, &instance_of_class_name, current_frame.clone().into(), current_frame.class_pointer.loader.clone());
                    let object_class = object.class_pointer.clone();
                    if inherits_from(state, &object_class, &instanceof_class) {
                        current_frame.push(JavaValue::Int(1))
                    } else {
                        current_frame.push(JavaValue::Int(0))
                    }
                },
                ReferenceTypeView::Array(_) => current_frame.push(JavaValue::Int(0)),
            }
        },
    }
}

fn runtime_super_class(state: &mut InterpreterState, inherits: &Arc<RuntimeClass>) -> Option<Arc<RuntimeClass>> {
    if inherits.classfile.has_super_class() {
        Some(check_inited_class(state, &inherits.classfile.super_class_name().unwrap(), None, inherits.loader.clone()))
    } else {
        None
    }
}


fn runtime_interface_class(state: &mut InterpreterState, class_: &Arc<RuntimeClass>, i: Interface) -> Arc<RuntimeClass> {
    let intf_name = class_.classfile.extract_class_from_constant_pool_name(i);
    check_inited_class(state, &ClassName::Str(intf_name), None, class_.loader.clone())
}

//todo this really shouldn't need state or Arc<RuntimeClass>
pub fn inherits_from(state: &mut InterpreterState, inherits: &Arc<RuntimeClass>, parent: &Arc<RuntimeClass>) -> bool {
    if class_name(&inherits.classfile) == class_name(&parent.classfile){
        return true;
    }
    let interfaces_match = inherits.classfile.interfaces.iter().any(|x| {
        let interface = runtime_interface_class(state, inherits, *x);
        class_name(&interface.classfile) == class_name(&parent.classfile)
    });

    (match runtime_super_class(state, inherits) {
        None => false,
        Some(super_) => {
            //todo why is this not an impl function?
            class_name(&super_.classfile) == class_name(&parent.classfile) ||
                inherits_from(state, &super_, parent)
        }
    }) || interfaces_match
}