use runtime_common::{InterpreterState, StackEntry};
use std::rc::Rc;
use crate::instructions::invoke::resolved_class;
use runtime_common::java_values::{Object, JavaValue};

use runtime_common::runtime_class::RuntimeClass;
use std::sync::Arc;
use rust_jvm_common::classfile::{MethodInfo, ACC_NATIVE, ACC_ABSTRACT};
use crate::rust_jni::get_all_methods;
use crate::interpreter_util::{run_function, check_inited_class};
use rust_jvm_common::classnames::ClassName;
use std::ops::Deref;
use crate::instructions::invoke::native::run_native_method;
use classfile_view::view::descriptor_parser::{MethodDescriptor, parse_method_descriptor};

pub fn invoke_virtual(state: &mut InterpreterState, current_frame: Rc<StackEntry>, cp: u16) {
    let (_resolved_class, method_name, expected_descriptor) = match resolved_class(state, current_frame.clone(), cp) {
        None => return,
        Some(o) => { o }
    };
//The resolved method must not be an instance initialization method,or the class or interface initialization method (§2.9)
    if method_name == "<init>".to_string() ||
        method_name == "<clinit>".to_string() {
        panic!()//should have been caught by verifier
    }

//If the resolved method is not signature polymorphic ( §2.9), thenthe invokevirtual instruction proceeds as follows.
//we assume that it isn't signature polymorphic for now todo

//Let C be the class of objectref.
    let this_pointer = {
        let operand_stack = current_frame.operand_stack.borrow();
        &operand_stack[operand_stack.len() - expected_descriptor.parameter_types.len() - 1].clone()
    };
    let c = match this_pointer.unwrap_object().unwrap().deref() {
        Object::Array(_a) => {
//todo so spec seems vague about this, but basically assume this is an Object
            let object_class = check_inited_class(state, &ClassName::object(), current_frame.clone().into(), current_frame.class_pointer.loader.clone());
            object_class.clone()
        }
        Object::Object(o) => {
            o.class_pointer.clone()
        }
    };
    let all_methods = get_all_methods(state, current_frame.clone(), c.clone());
    let (final_target_class, new_i) = all_methods.iter().find(|(c, m)| {
        let cur_method_info = &c.classfile.methods[*m];
        let cur_name = cur_method_info.method_name(&c.classfile);
        let desc_str = cur_method_info.descriptor_str(&c.classfile);
        let cur_desc = parse_method_descriptor(desc_str.as_str()).unwrap();
        let expected_name = &method_name;
        &cur_name == expected_name &&
            expected_descriptor.parameter_types == cur_desc.parameter_types &&
            !cur_method_info.is_static() &&
            !cur_method_info.is_abstract()
    }).unwrap();
    let final_classfile = &final_target_class.classfile;
    let target_method = &final_classfile.methods[*new_i];
    let final_descriptor = parse_method_descriptor(target_method.descriptor_str(&final_classfile).as_str()).unwrap();
    invoke_virtual_method_i(state, current_frame.clone(), final_descriptor, final_target_class.clone(), *new_i, target_method)
}

pub fn invoke_virtual_method_i(state: &mut InterpreterState, current_frame: Rc<StackEntry>, expected_descriptor: MethodDescriptor, target_class: Arc<RuntimeClass>, target_method_i: usize, target_method: &MethodInfo) -> () {
    invoke_virtual_method_i_impl(state, current_frame.clone(), expected_descriptor, target_class, target_method_i, target_method)
}

pub fn invoke_virtual_method_i_impl(
    state: &mut InterpreterState,
    current_frame: Rc<StackEntry>,
    expected_descriptor: MethodDescriptor,
    target_class: Arc<RuntimeClass>,
    target_method_i: usize,
    target_method: &MethodInfo,
) -> () {
    if target_method.access_flags & ACC_NATIVE > 0 {
        run_native_method(state, current_frame.clone(), target_class, target_method_i)
    } else if target_method.access_flags & ACC_ABSTRACT == 0 {
//todo this is wrong?
        let mut args = vec![];
        let max_locals = target_method.code_attribute().unwrap().max_locals;

        setup_virtual_args(&current_frame, &expected_descriptor, &mut args, max_locals);
        let next_entry = StackEntry {
            last_call_stack: Some(current_frame),
            class_pointer: target_class.clone(),
            method_i: target_method_i as u16,
            local_vars: args.into(),
            operand_stack: vec![].into(),
            pc: 0.into(),
            pc_offset: 0.into(),
        };
        run_function(state, Rc::new(next_entry));
        if state.throw.is_some() || state.terminate {
            return;
        }
        if state.function_return {
            state.function_return = false;
            return;
        }
    } else {
        panic!()
    }
}

//todo we should be going to this first imo. b/c as is we have correctness issues with overloaded impls?
pub fn actually_virtual(state: &mut InterpreterState, current_frame: Rc<StackEntry>, expected_descriptor: MethodDescriptor, target_class: &Arc<RuntimeClass>, target_method: &MethodInfo) -> () {
    let this_pointer = {
        let operand_stack = current_frame.operand_stack.borrow();
        &operand_stack[operand_stack.len() - expected_descriptor.parameter_types.len() - 1].clone()
    };
    let new_target_class = this_pointer.unwrap_object().unwrap().unwrap_normal_object().class_pointer.clone();
    assert_eq!(new_target_class.classfile.access_flags & ACC_ABSTRACT, 0);
//todo so this is incorrect due to subclassing of return value.
    let all_methods = get_all_methods(state, current_frame.clone(), new_target_class.clone());
    let (final_target_class, new_i) = all_methods.iter().find(|(c, m)| {
        let cur_method_info = &c.classfile.methods[*m];
        let cur_name = cur_method_info.method_name(&c.classfile);
        let desc_str = cur_method_info.descriptor_str(&c.classfile);
        let cur_desc = parse_method_descriptor(desc_str.as_str()).unwrap();
        let expected_name = target_method.method_name(&target_class.classfile);

        cur_name == expected_name &&
            expected_descriptor.parameter_types == cur_desc.parameter_types &&
            !cur_method_info.is_static() &&
            !cur_method_info.is_abstract() & &
                !cur_method_info.is_native()
    }).unwrap();
    invoke_virtual_method_i(state, current_frame, expected_descriptor, final_target_class.clone(), *new_i, &final_target_class.classfile.methods[*new_i])
}

pub fn setup_virtual_args(current_frame: &Rc<StackEntry>, expected_descriptor: &MethodDescriptor, args: &mut Vec<JavaValue>, max_locals: u16) {
    for _ in 0..max_locals {
        args.push(JavaValue::Top);
    }
    let mut i = 1;
    for _ in &expected_descriptor.parameter_types {
        let value = current_frame.pop();
        match value.clone() {
            JavaValue::Long(_) | JavaValue::Double(_) => {
                args[i] = JavaValue::Top;
                args[i + 1] = value;
                i += 2
            }
            _ => {
                args[i] = value;
                i += 1
            }
        };
    }
    if expected_descriptor.parameter_types.len() != 0 {
        args[1..i].reverse();
    }
    args[0] = current_frame.pop();
}
