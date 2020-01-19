use crate::InterpreterState;
use std::rc::Rc;
use crate::CallStackEntry;
use verification::verifier::instructions::branches::get_method_descriptor;
use rust_jvm_common::utils::code_attribute;
use rust_jvm_common::classfile::ACC_NATIVE;
use rust_jvm_common::classfile::ACC_STATIC;
use crate::interpreter_util::run_function;
use classfile_parser::types::MethodDescriptor;
use std::sync::Arc;
use rust_jvm_common::loading::Loader;
use rust_jvm_common::classfile::MethodInfo;
use classfile_parser::types::parse_method_descriptor;
use rust_jvm_common::utils::method_name;
use rust_jvm_common::utils::extract_string_from_utf8;
use rust_jvm_common::classfile::ACC_ABSTRACT;
use rust_jvm_common::unified_types::ParsedType;
use crate::interpreter_util::check_inited_class;
use crate::native::run_native_method;
use runtime_common::java_values::JavaValue;
use runtime_common::runtime_class::RuntimeClass;
use rust_jni::LibJavaLoading;


pub fn invoke_special(state: &mut InterpreterState, current_frame: &Rc<CallStackEntry>, jni: &LibJavaLoading, cp: u16) -> () {
    let loader_arc = current_frame.class_pointer.loader.clone();
    let (method_class_type, method_name, parsed_descriptor) = get_method_descriptor(cp as usize, &current_frame.class_pointer.classfile, loader_arc.clone());
    let method_class_name = match method_class_type {
        ParsedType::Class(c) => c.class_name,
        _ => panic!()
    };
    let target_class = check_inited_class(state, &method_class_name, current_frame.clone(), loader_arc.clone(), jni);
    let (target_m_i, target_m) = find_target_method(loader_arc.clone(), method_name, &parsed_descriptor, &target_class);
    let mut args = vec![];
    let max_locals = code_attribute(target_m).unwrap().max_locals;
    for _ in 0..max_locals {
        args.push(JavaValue::Top);
    }
    args[0] = current_frame.operand_stack.borrow_mut().pop().unwrap();
    for i in 1..(parsed_descriptor.parameter_types.len() + 1) {
        args[i] = current_frame.operand_stack.borrow_mut().pop().unwrap();
        //todo does ordering end up correct
    }
    let next_entry = CallStackEntry {
        last_call_stack: Some(current_frame.clone()),
        class_pointer: target_class,
        method_i: target_m_i as u16,
        local_vars: args,
        operand_stack: vec![].into(),
        pc: 0.into(),
        pc_offset: 0.into()
    };
    run_function(state, Rc::new(next_entry), jni);
    if state.terminate || state.throw{
        unimplemented!()
    }
    if state.function_return {
        state.function_return = false;
        return;
    }
}

pub fn invoke_virtual(state: &mut InterpreterState, current_frame: Rc<CallStackEntry>, cp: u16,jni: &LibJavaLoading){
    let classfile = &current_frame.class_pointer.classfile;
    let loader_arc = &current_frame.class_pointer.loader;
    let (class_name_type, expected_method_name, expected_descriptor) = get_method_descriptor(cp as usize, &classfile.clone(), loader_arc.clone());
    let class_name = match class_name_type {
        ParsedType::Class(c) => c.class_name,
        ParsedType::ArrayReferenceType(_) => unimplemented!(),
        _ => panic!()
    };
//    dbg!(class_name_);
//    dbg!(expected_method_name);
//    dbg!(class_name(&current_frame.class_pointer.classfile).get_referred_name());
    let target_class = check_inited_class(state, &class_name, current_frame.clone(), loader_arc.clone(),jni);
    let (target_method_i,target_method) = find_target_method(loader_arc.clone(), expected_method_name, &expected_descriptor, &target_class);
    if target_method.access_flags & ACC_ABSTRACT == 0 {
        let mut args = vec![];
        let max_locals = code_attribute(target_method).unwrap().max_locals;

        for _ in 0..max_locals{
            args.push(JavaValue::Top);
        }
        args[0] = current_frame.operand_stack.borrow_mut().pop().unwrap();
        for i in 0..expected_descriptor.parameter_types.len(){
            args[i] = current_frame.operand_stack.borrow_mut().pop().unwrap();
            //todo does ordering end up correct
        }
        let next_entry = CallStackEntry {
            last_call_stack: Some(current_frame),
            class_pointer: target_class,
            method_i: target_method_i as u16,
            local_vars: args,
            operand_stack: vec![].into(),
            pc: 0.into(),
            pc_offset: 0.into()
        };
        run_function(state,Rc::new(next_entry),jni);
        if state.throw || state.terminate {
            unimplemented!();
        }
        if state.function_return {
            state.function_return = false;
            return;
        }
    }else {
        unimplemented!()
    }
}

pub fn run_invoke_static(state: &mut InterpreterState, current_frame: Rc<CallStackEntry>, cp: u16,jni: &LibJavaLoading) {
//todo handle monitor enter and exit
//handle init cases
    let classfile = &current_frame.class_pointer.classfile;
    let loader_arc = &current_frame.class_pointer.loader;
    let (class_name_type, expected_method_name, expected_descriptor) = get_method_descriptor(cp as usize, &classfile.clone(), loader_arc.clone());
    let class_name = match class_name_type {
        ParsedType::Class(c) => c.class_name,
        _ => panic!()
    };
    let target_class = check_inited_class(state, &class_name, current_frame.clone(), loader_arc.clone(),jni);
    let (target_method_i,target_method) = find_target_method(loader_arc.clone(), expected_method_name, &expected_descriptor, &target_class);
    let mut args = vec![];

    if target_method.access_flags & ACC_NATIVE == 0{
        assert!(target_method.access_flags & ACC_STATIC > 0);
        assert_eq!(target_method.access_flags & ACC_ABSTRACT, 0);
        let max_locals = code_attribute(target_method).unwrap().max_locals;

        for _ in 0..max_locals{
            args.push(JavaValue::Top);
        }

        for i in 0..expected_descriptor.parameter_types.len(){
            args[i] = current_frame.operand_stack.borrow_mut().pop().unwrap();
            //todo does ordering end up correct
        }
        let next_entry = CallStackEntry {
            last_call_stack: Some(current_frame),
            class_pointer: target_class,
            method_i: target_method_i as u16,
            local_vars: args,
            operand_stack: vec![].into(),
            pc: 0.into(),
            pc_offset: 0.into()
        };
        run_function(state,Rc::new(next_entry),jni);
        if state.throw || state.terminate {
            unimplemented!();
        }
        if state.function_return {
            state.function_return = false;
            return;
        }
    }else{
        //only works for static void
        run_native_method(state,current_frame.clone(),target_class,target_method_i,jni);
    }
}

pub fn find_target_method<'l>(
    loader_arc: Arc<dyn Loader + Send + Sync>,
    expected_method_name: String,
    parsed_descriptor: & MethodDescriptor,
    target_class: &'l Arc<RuntimeClass>
) -> (usize,&'l MethodInfo) {
    target_class.classfile.methods.iter().enumerate().find(|(_, m)| {
        if method_name(&target_class.classfile, m) == expected_method_name {
            let target_class_descriptor_str = extract_string_from_utf8(&target_class.classfile.constant_pool[m.descriptor_index as usize]);
            let actual = parse_method_descriptor(&loader_arc, target_class_descriptor_str.as_str()).unwrap();
            actual.parameter_types.len() == parsed_descriptor.parameter_types.len() &&
                actual.parameter_types.iter().zip(parsed_descriptor.parameter_types.iter()).all(|(a, b)| a == b) &&
                actual.return_type == parsed_descriptor.return_type
        } else {
            false
        }
    }).unwrap()
}
