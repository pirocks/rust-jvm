use runtime_common::{InterpreterState, StackEntry};
use std::rc::Rc;
use rust_jvm_common::classfile::{ConstantKind, Atype};
use crate::interpreter_util::{push_new_object, check_inited_class};
use rust_jvm_common::classnames::ClassName;
use runtime_common::java_values::{JavaValue, default_value};
use rust_jvm_common::unified_types::ParsedType;

pub fn new(state: &mut InterpreterState, current_frame: &Rc<StackEntry>, cp: usize) -> () {
    let loader_arc = &current_frame.class_pointer.loader;
    let constant_pool = &current_frame.class_pointer.classfile.constant_pool;
    let class_name_index = match &constant_pool[cp as usize].kind {
        ConstantKind::Class(c) => c.name_index,
        _ => panic!()
    };
    let target_class_name = ClassName::Str(constant_pool[class_name_index as usize].extract_string_from_utf8());
    let target_classfile = check_inited_class(state, &target_class_name, current_frame.clone().into(), loader_arc.clone());
    push_new_object(current_frame.clone().into(), &target_classfile);
}


pub fn anewarray(state: &mut InterpreterState, current_frame: Rc<StackEntry>, cp: u16) -> () {
    let len = match current_frame.pop() {
        JavaValue::Int(i) => i,
        _ => panic!()
    };
    let constant_pool = &current_frame.class_pointer.classfile.constant_pool;
    let cp_entry = &constant_pool[cp as usize].kind;
    match cp_entry {
        ConstantKind::Class(c) => {
            let name = constant_pool[c.name_index as usize].extract_string_from_utf8();
            check_inited_class(state, &ClassName::Str(name), current_frame.clone().into(), current_frame.class_pointer.loader.clone());
            current_frame.push(JavaValue::Array(JavaValue::new_vec(len as usize, JavaValue::Object(None))))
        }
        _ => {
            dbg!(cp_entry);
            panic!()
        }
    }
}


pub fn newarray(current_frame: &Rc<StackEntry>, a_type: Atype) -> () {
    let count = match current_frame.pop() {
        JavaValue::Int(i) => { i }
        _ => panic!()
    };
    match a_type {
        Atype::TChar => {
            current_frame.push(JavaValue::Array(JavaValue::new_vec(count as usize, default_value(ParsedType::CharType))));
        }
        _ => unimplemented!()
    }
}