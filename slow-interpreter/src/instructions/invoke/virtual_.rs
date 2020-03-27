use std::rc::Rc;
use crate::instructions::invoke::resolved_class;
use std::sync::Arc;
use rust_jvm_common::classfile::{MethodInfo, ACC_NATIVE, ACC_ABSTRACT};
use crate::rust_jni::get_all_methods;
use crate::interpreter_util::{run_function, check_inited_class};
use rust_jvm_common::classnames::ClassName;
use std::ops::Deref;
use crate::instructions::invoke::native::run_native_method;
use classfile_view::view::HasAccessFlags;
use classfile_view::view::ptype_view::PTypeView;
use crate::java_values::{JavaValue, Object};
use crate::{StackEntry, InterpreterState};
use crate::runtime_class::RuntimeClass;
use descriptor_parser::{MethodDescriptor, parse_method_descriptor};


/**
Should only be used for an actual invoke_virtual instruction.
Otherwise we have a better method for invoke_virtual w/ resolution
*/
pub fn invoke_virtual_instruction(state: &mut InterpreterState, current_frame: Rc<StackEntry>, cp: u16) {
    let (_resolved_class, method_name, expected_descriptor) = match resolved_class(state, current_frame.clone(), cp) {
        None => return,
        Some(o) => { o }
    };
    invoke_virtual(state, current_frame, &method_name, &expected_descriptor)
}

pub fn invoke_virtual_method_i(state: &mut InterpreterState, current_frame: Rc<StackEntry>, expected_descriptor: MethodDescriptor, target_class: Arc<RuntimeClass>, target_method_i: usize, target_method: &MethodInfo) -> () {
    invoke_virtual_method_i_impl(state, current_frame.clone(), expected_descriptor, target_class, target_method_i, target_method)
}

fn invoke_virtual_method_i_impl(
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


pub fn invoke_virtual_unparsed(state: &mut InterpreterState, current_frame: Rc<StackEntry>, method_name: &String, desc: &String) -> () {
    invoke_virtual(state, current_frame, method_name, &parse_method_descriptor(desc).unwrap())
}

/*
args should be on the stack
*/
pub fn invoke_virtual(state: &mut InterpreterState, current_frame: Rc<StackEntry>, method_name: &String, md: &MethodDescriptor) -> () {
    //The resolved method must not be an instance initialization method,or the class or interface initialization method (§2.9)
    if method_name == "<init>" ||
        method_name == "<clinit>" {
        panic!()//should have been caught by verifier, though perhaps it is possible to reach this w/ invokedynamic todo
    }
    //todo implement locking on synchronized methods

//If the resolved method is not signature polymorphic ( §2.9), then the invokevirtual instruction proceeds as follows.
//we assume that it isn't signature polymorphic for now todo

//Let C be the class of objectref.
    let this_pointer = {
        let operand_stack = current_frame.operand_stack.borrow();
        &operand_stack[operand_stack.len() - md.parameter_types.len() - 1].clone()
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

    let (final_target_class, new_i) = virtual_method_lookup(state, &current_frame, &method_name, md, c);
    let final_classfile = &final_target_class.classfile;
    let target_method = &final_classfile.methods[new_i];
    let final_descriptor = parse_method_descriptor(target_method.descriptor_str(&final_classfile).as_str()).unwrap();
    invoke_virtual_method_i(state, current_frame.clone(), final_descriptor, final_target_class.clone(), new_i, target_method)
}

pub fn virtual_method_lookup(state: &mut InterpreterState, current_frame: &Rc<StackEntry>, method_name: &String, md: &MethodDescriptor, c: Arc<RuntimeClass>) -> (Arc<RuntimeClass>, usize) {
    let all_methods = get_all_methods(state, current_frame.clone(), c.clone());
    let (final_target_class, new_i) = all_methods.iter().find(|(c, i)| {
        let method_view = c.class_view.method_view_i(*i);
        let cur_name = method_view.name();
        let cur_desc = method_view.desc();
        let expected_name = &method_name;
        &&cur_name == expected_name &&
            !method_view.is_static() &&
            !method_view.is_abstract() &&
            if method_view.is_signature_polymorphic() {
                method_view.desc().parameter_types[0] == PTypeView::array(PTypeView::object()).to_ptype() &&
                    method_view.desc().return_type == PTypeView::object().to_ptype() &&
                    md.parameter_types.last()
                        .and_then(|x| PTypeView::from_ptype(x).try_unwrap_ref_type().map(|x2|x2.clone()))
                        .map(|x| x.unwrap_name() == ClassName::member_name())
                        .unwrap_or(false);//todo this is currently under construction.
                unimplemented!()
            } else {
                md.parameter_types == cur_desc.parameter_types //we don't check return types b/c these could be subclassed
            }
    }).unwrap();
    (final_target_class.clone(), *new_i)
}