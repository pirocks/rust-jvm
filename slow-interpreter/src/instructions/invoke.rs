use crate::InterpreterState;
use std::rc::Rc;
use verification::verifier::instructions::branches::get_method_descriptor;
use rust_jvm_common::classfile::{ACC_NATIVE, ACC_STATIC, InvokeInterface};
use crate::interpreter_util::run_function;
use classfile_parser::types::MethodDescriptor;
use std::sync::Arc;
use rust_jvm_common::loading::LoaderArc;
use rust_jvm_common::classfile::MethodInfo;
use rust_jvm_common::classfile::ACC_ABSTRACT;
use rust_jvm_common::unified_types::ParsedType;
use crate::interpreter_util::check_inited_class;
use runtime_common::java_values::JavaValue;
use runtime_common::runtime_class::RuntimeClass;
use runtime_common::StackEntry;
use std::cell::Ref;
use crate::rust_jni::{call_impl, call};
use std::borrow::Borrow;
use utils::lookup_method_parsed;


pub fn invoke_special(state: &mut InterpreterState, current_frame: &Rc<StackEntry>, cp: u16) -> () {
    let loader_arc = current_frame.class_pointer.loader.clone();
    let (method_class_type, method_name, parsed_descriptor) = get_method_descriptor(cp as usize, &current_frame.class_pointer.classfile, loader_arc.clone());
    let method_class_name = match method_class_type {
        ParsedType::Class(c) => c.class_name,
        _ => panic!()
    };
//    trace!("Call:{} {}", method_class_name.get_referred_name(), method_name.clone());
    let target_class = check_inited_class(state, &method_class_name, current_frame.clone().into(), loader_arc.clone());
    let (target_m_i, final_target_class) = find_target_method(state,loader_arc.clone(), method_name.clone(), &parsed_descriptor, target_class);
    let target_m = &final_target_class.classfile.methods[target_m_i];
    let mut args = vec![];
    let max_locals = target_m.code_attribute().unwrap().max_locals;
    for _ in 0..max_locals {
        args.push(JavaValue::Top);
    }
    for i in 1..(parsed_descriptor.parameter_types.len() + 1) {
        args[i] = current_frame.pop();
        //todo does ordering end up correct
    }
    args[1..(parsed_descriptor.parameter_types.len() + 1)].reverse();
    args[0] = current_frame.pop();
//    dbg!(&args);
    let next_entry = StackEntry {
        last_call_stack: Some(current_frame.clone()),
        class_pointer: final_target_class.clone(),
        method_i: target_m_i as u16,
        local_vars: args.into(),
        operand_stack: vec![].into(),
        pc: 0.into(),
        pc_offset: 0.into(),
    };
    run_function(state, Rc::new(next_entry));
    if state.terminate || state.throw {
        unimplemented!()
    }
    if state.function_return {
        state.function_return = false;
//        trace!("Exit:{} {}", method_class_name.get_referred_name(), method_name.clone());
        return;
    }
}

pub fn invoke_virtual(state: &mut InterpreterState, current_frame: Rc<StackEntry>, cp: u16) {
    let classfile = &current_frame.class_pointer.classfile;
    let loader_arc = &current_frame.class_pointer.loader;
    let (class_name_type, expected_method_name, expected_descriptor) = get_method_descriptor(cp as usize, &classfile.clone(), loader_arc.clone());
    let class_name_ = match class_name_type {
        ParsedType::Class(c) => c.class_name,
        ParsedType::ArrayReferenceType(_) => unimplemented!(),
        _ => panic!()
    };
    //todo should I be trusting these descriptors, or should i be using the runtime class on top of the operant stack
    let target_class = check_inited_class(state, &class_name_, current_frame.clone().into(), loader_arc.clone());
    let (target_method_i, final_target_class) = find_target_method(state,loader_arc.clone(), expected_method_name.clone(), &expected_descriptor, target_class);
    invoke_virtual_method_i(state, current_frame, expected_descriptor, final_target_class.clone(), target_method_i, &final_target_class.classfile.methods[target_method_i])
}

pub fn invoke_virtual_method_i(state: &mut InterpreterState, current_frame: Rc<StackEntry>, expected_descriptor: MethodDescriptor, target_class: Arc<RuntimeClass>, target_method_i: usize, target_method: &MethodInfo) -> () {
    if target_method.access_flags & ACC_NATIVE > 0 {
        run_native_method(state, current_frame.clone(), target_class, target_method_i)
    } else if target_method.access_flags & ACC_ABSTRACT == 0 {
        let mut args = vec![];
        let max_locals = target_method.code_attribute().unwrap().max_locals;

//        dbg!(target_method.method_name(&target_class.classfile));

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
        if state.throw || state.terminate {
            unimplemented!();
        }
        if state.function_return {
            state.function_return = false;
//            trace!("Exit:{} {}", class_name(&target_class.classfile).get_referred_name(), expected_method_name);
            return;
        }
    } else {
        unimplemented!()
    }
}

pub fn setup_virtual_args(current_frame: &Rc<StackEntry>, expected_descriptor: &MethodDescriptor, args: &mut Vec<JavaValue>, max_locals: u16) {
    for _ in 0..(max_locals) {// todo +1
        args.push(JavaValue::Top);
    }
    for i in 1..(expected_descriptor.parameter_types.len() + 1) {
        args[i] = current_frame.pop();
        //todo does ordering end up correct
    }
//    dbg!(&args[1..(expected_descriptor.parameter_types.len() + 1)]);
    if expected_descriptor.parameter_types.len() != 0 {
        args[1..(expected_descriptor.parameter_types.len() + 1)].reverse();
    }
    args[0] = current_frame.pop();
}

pub fn run_invoke_static(state: &mut InterpreterState, current_frame: Rc<StackEntry>, cp: u16) {
//todo handle monitor enter and exit
//handle init cases
    let classfile = &current_frame.class_pointer.classfile;
    let loader_arc = &current_frame.class_pointer.loader;
    let (class_name_type, expected_method_name, expected_descriptor) = get_method_descriptor(cp as usize, &classfile.clone(), loader_arc.clone());
    let class_name = match class_name_type {
        ParsedType::Class(c) => c.class_name,
        _ => panic!()
    };
    let target_class = check_inited_class(state, &class_name, current_frame.clone().into(), loader_arc.clone());
    let (target_method_i, final_target_method) = find_target_method(state,loader_arc.clone(), expected_method_name.clone(), &expected_descriptor, target_class);

    invoke_static_impl(state, current_frame, expected_descriptor, final_target_method.clone(), target_method_i, &final_target_method.classfile.methods[target_method_i]);
}

pub fn invoke_static_impl(
    state: &mut InterpreterState,
    current_frame: Rc<StackEntry>,
    expected_descriptor: MethodDescriptor,
    target_class: Arc<RuntimeClass>,
    target_method_i: usize,
    target_method: &MethodInfo,
) -> () {
    let mut args = vec![];
    if target_method.access_flags & ACC_NATIVE == 0 {
//        dbg!(&target_class.static_vars.borrow().get("savedProps"));
        assert!(target_method.access_flags & ACC_STATIC > 0);
        assert_eq!(target_method.access_flags & ACC_ABSTRACT, 0);
        let max_locals = target_method.code_attribute().unwrap().max_locals;
//        dbg!(&target_class.static_vars.borrow().get("savedProps"));
        for _ in 0..max_locals {
            args.push(JavaValue::Top);
        }
//        dbg!(&target_class.static_vars.borrow().get("savedProps"));
        for i in 0..expected_descriptor.parameter_types.len() {
            args[i] = current_frame.pop();
            //todo does ordering end up correct
        }
        args[0..expected_descriptor.parameter_types.len()].reverse();
//        dbg!(&target_class.static_vars.borrow().get("savedProps"));
        let next_entry = StackEntry {
            last_call_stack: Some(current_frame),
            class_pointer: target_class,
            method_i: target_method_i as u16,
            local_vars: args.into(),
            operand_stack: vec![].into(),
            pc: 0.into(),
            pc_offset: 0.into(),
        };
        run_function(state, Rc::new(next_entry));
        if state.throw || state.terminate {
            unimplemented!();
        }
        if state.function_return {
            state.function_return = false;
            return;
        }
    } else {
//        dbg!(&target_class.static_vars.borrow().get("savedProps"));
        //only works for static void
        run_native_method(state, current_frame.clone(), target_class.clone(), target_method_i);
//        dbg!(&target_class.static_vars.borrow().get("savedProps"));
    }
}

pub fn find_target_method(
    state: &mut InterpreterState,
    loader_arc: LoaderArc,
    expected_method_name: String,
    parsed_descriptor: &MethodDescriptor,
    target_class: Arc<RuntimeClass>
) -> (usize, Arc<RuntimeClass>) {
    //todo bug need to handle super class, issue with that is need frame/state.
    lookup_method_parsed(state,target_class,expected_method_name,parsed_descriptor,&loader_arc).unwrap()
}


pub fn run_native_method(
    state: &mut InterpreterState,
    frame: Rc<StackEntry>,
    class: Arc<RuntimeClass>,
    method_i: usize,
) {
    //todo only works for static void methods atm
    let classfile = &class.classfile;
    let method = &classfile.methods[method_i];
    assert!(method.access_flags & ACC_NATIVE > 0);
    let parsed = MethodDescriptor::from(method, classfile, &class.loader);
    let mut args = vec![];
    //todo should have some setup args functions
    if method.access_flags & ACC_STATIC > 0 {
        for _ in parsed.parameter_types {
            args.push(frame.pop());
        }
        args.reverse();
    } else {
        setup_virtual_args(&frame, &parsed, &mut args, (parsed.parameter_types.len() + 1) as u16)
    }
    if method.method_name(classfile) == "desiredAssertionStatus0".to_string() {//todo and descriptor matches and class matches
        frame.push(JavaValue::Boolean(false))
    } else if method.method_name(classfile) == "arraycopy".to_string() {
        system_array_copy(&mut args)
    } else {
        let result = if state.jni.registered_natives.borrow().contains_key(&class) &&
            state.jni.registered_natives.borrow().get(&class).unwrap().borrow().contains_key(&(method_i as u16))
        {
            //todo dup
            let res_fn = {
                let reg_natives = state.jni.registered_natives.borrow();
                let reg_natives_for_class = reg_natives.get(&class).unwrap().borrow();
                reg_natives_for_class.get(&(method_i as u16)).unwrap().clone()
            };
            let res = call_impl(state, frame.clone(), class.clone(), args, parsed.return_type, &res_fn).unwrap();
            res
        } else {
            let res = call(state, frame.clone(), class.clone(), method_i, args, parsed.return_type).unwrap();
            res
        };
        match result {
            None => {}
            Some(res) => frame.push(res),
        }
    }
}

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



pub fn invoke_interface(state: &mut InterpreterState, current_frame: Rc<StackEntry>, invoke_interface: InvokeInterface) {
    invoke_interface.count;
    let classfile = &current_frame.class_pointer.classfile;
    let loader_arc = &current_frame.class_pointer.loader;
    let (class_name_type, expected_method_name, expected_descriptor) = get_method_descriptor(invoke_interface.index as usize, &classfile.clone(), loader_arc.clone());
    let class_name_ = match class_name_type {
        ParsedType::Class(c) => c.class_name,
        ParsedType::ArrayReferenceType(_) => unimplemented!(),
        _ => panic!()
    };
    //todo should I be trusting these descriptors, or should i be using the runtime class on top of the operant stack
    let _target_class = check_inited_class(state, &class_name_, current_frame.clone().into(), loader_arc.clone());
    let mut args = vec![];
    let checkpoint = current_frame.operand_stack.borrow().clone();
    setup_virtual_args(&current_frame, &expected_descriptor, &mut args, expected_descriptor.parameter_types.len() as u16  + 1);
    let this_pointer_o = args[0].unwrap_object().unwrap();
    let this_pointer = this_pointer_o.unwrap_object();
    current_frame.operand_stack.replace(checkpoint);
    let target_class = this_pointer.class_pointer.clone();
//    dbg!(invoke_interface.count);
//    dbg!(class_name(&target_class.classfile));
    let (target_method_i, final_target_class) = find_target_method(state,loader_arc.clone(), expected_method_name.clone(), &expected_descriptor, target_class);

    invoke_virtual_method_i(state, current_frame, expected_descriptor, final_target_class.clone(), target_method_i, &final_target_class.classfile.methods[target_method_i]);
}