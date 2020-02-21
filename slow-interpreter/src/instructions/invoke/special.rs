use crate::interpreter_util::{run_function, check_inited_class};
use std::rc::Rc;
use runtime_common::{StackEntry, InterpreterState};
use crate::instructions::invoke::virtual_::setup_virtual_args;
use crate::instructions::invoke::find_target_method;
use rust_jvm_common::classfile::{ACC_NATIVE, MethodInfo};
use descriptor_parser::MethodDescriptor;
use verification::verifier::instructions::branches::get_method_descriptor;
use rust_jvm_common::view::ClassView;
use runtime_common::runtime_class::RuntimeClass;
use std::sync::Arc;
use crate::instructions::invoke::native::run_native_method;

pub fn invoke_special(state: &mut InterpreterState, current_frame: &Rc<StackEntry>, cp: u16) -> () {
    let loader_arc = current_frame.class_pointer.loader.clone();
    let (method_class_type, method_name, parsed_descriptor) = get_method_descriptor(cp as usize, &ClassView::from(current_frame.class_pointer.classfile.clone()));
    let method_class_name = method_class_type.unwrap_class_type();
//    trace!("Call:{} {}", method_class_name.get_referred_name(), method_name.clone());
    let target_class = check_inited_class(state, &method_class_name, current_frame.clone().into(), loader_arc.clone());
    let (target_m_i, final_target_class) = find_target_method(state, loader_arc.clone(), method_name.clone(), &parsed_descriptor, target_class);
    let target_m = &final_target_class.classfile.methods[target_m_i];
    invoke_special_impl(state, current_frame, &parsed_descriptor, target_m_i, final_target_class.clone(), target_m);
//    if method_name == "<init>"{
//        dbg!(&current_frame.operand_stack);
//    }
}

pub fn invoke_special_impl(state: &mut InterpreterState, current_frame: &Rc<StackEntry>, parsed_descriptor: &MethodDescriptor, target_m_i: usize, final_target_class: Arc<RuntimeClass>, target_m: &MethodInfo) -> () {
    if target_m.access_flags & ACC_NATIVE > 0 {
        run_native_method(state, current_frame.clone(), final_target_class, target_m_i);
    } else {
        let mut args = vec![];
//        dbg!(method_class_name.get_referred_name());
//        dbg!(&method_name);
        let max_locals = target_m.code_attribute().unwrap().max_locals;
        setup_virtual_args(current_frame, &parsed_descriptor, &mut args, max_locals);
        let next_entry = StackEntry {
            last_call_stack: Some(current_frame.clone()),
            class_pointer: final_target_class.clone(),
            method_i: target_m_i as u16,
            local_vars: args.into(),
            operand_stack: vec![].into(),
            pc: 0.into(),
            pc_offset: 0.into(),
        };
//        dbg!(target_m.code_attribute());
        run_function(state, Rc::new(next_entry));
        if state.throw.is_some() || state.terminate {
            return;
        }
        if state.function_return {
            state.function_return = false;
            return;
//        trace!("Exit:{} {}", method_class_name.get_referred_name(), method_name.clone());
        }
    }
}
