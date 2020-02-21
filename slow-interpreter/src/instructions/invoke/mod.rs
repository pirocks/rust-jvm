use crate::InterpreterState;
use std::rc::Rc;
use verification::verifier::instructions::branches::get_method_descriptor;
use rust_jvm_common::classfile::{ACC_NATIVE, ACC_STATIC, InvokeInterface};
use crate::interpreter_util::run_function;
use std::sync::Arc;
use rust_jvm_common::loading::LoaderArc;
use rust_jvm_common::classfile::MethodInfo;
use rust_jvm_common::classfile::ACC_ABSTRACT;
use crate::interpreter_util::check_inited_class;
use runtime_common::java_values::{JavaValue, Object, ArrayObject};
use runtime_common::runtime_class::RuntimeClass;
use runtime_common::StackEntry;
use std::cell::Ref;
use crate::rust_jni::{call_impl, call, mangling, get_all_methods};
use std::borrow::Borrow;
use utils::lookup_method_parsed;
use rust_jvm_common::classnames::{class_name, ClassName};
use std::intrinsics::transmute;
use descriptor_parser::{MethodDescriptor, parse_method_descriptor};
use rust_jvm_common::view::ptype_view::{PTypeView, ReferenceTypeView};
use rust_jvm_common::view::ClassView;
use std::ops::Deref;

pub mod special;

fn resolved_class(state: &mut InterpreterState, current_frame: Rc<StackEntry>, cp: u16) -> Option<(Arc<RuntimeClass>, String, MethodDescriptor)> {
    let classfile = &current_frame.class_pointer.classfile;
    let loader_arc = &current_frame.class_pointer.loader;
    let (class_name_type, expected_method_name, expected_descriptor) = get_method_descriptor(cp as usize, &ClassView::from(classfile.clone()));
    let class_name_ = match class_name_type {
        PTypeView::Ref(r) => {
            match r {
                ReferenceTypeView::Class(c) => c,
                ReferenceTypeView::Array(_a) => {
                    if expected_method_name == "clone".to_string() {
                        //todo replace with proper native impl
                        let temp = current_frame.pop().unwrap_object().unwrap();
                        let to_clone_array = temp.unwrap_array();
                        current_frame.push(JavaValue::Object(Some(Arc::new(Object::Array(ArrayObject { elems: to_clone_array.elems.clone(), elem_type: to_clone_array.elem_type.clone() })))));
                        return None;
                    } else {
                        unimplemented!();
                    }
                }
            }
        }
        _ => panic!()
    };
    //todo should I be trusting these descriptors, or should i be using the runtime class on top of the operant stack
    let resolved_class = check_inited_class(state, &class_name_, current_frame.clone().into(), loader_arc.clone());
    (resolved_class, expected_method_name, expected_descriptor).into()
}

pub mod virtual_;

pub mod static_;

pub fn find_target_method(
    state: &mut InterpreterState,
    loader_arc: LoaderArc,
    expected_method_name: String,
    parsed_descriptor: &MethodDescriptor,
    target_class: Arc<RuntimeClass>,
) -> (usize, Arc<RuntimeClass>) {
    //todo bug need to handle super class, issue with that is need frame/state.
    lookup_method_parsed(state, target_class, expected_method_name, parsed_descriptor, &loader_arc).unwrap()
}

pub mod native;

fn system_array_copy(args: &mut Vec<JavaValue>) -> () {
    let src_o = args[0].clone().unwrap_object();
    let src = src_o.as_ref().unwrap().unwrap_array();
    let src_pos = args[1].clone().unwrap_int() as usize;
    let src_o = args[2].clone().unwrap_object();
    let dest = src_o.as_ref().unwrap().unwrap_array();
    let dest_pos = args[3].clone().unwrap_int() as usize;
    let length = args[4].clone().unwrap_int() as usize;
    for i in 0..length {
        let borrowed: Ref<Vec<JavaValue>> = src.elems.borrow();
        let temp = (borrowed.borrow())[src_pos + i].borrow().clone();
        dest.elems.borrow_mut()[dest_pos + i] = temp;
    }
}

pub mod interface;