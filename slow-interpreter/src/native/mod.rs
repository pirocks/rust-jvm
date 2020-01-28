use crate::InterpreterState;
use crate::StackEntry;
use std::rc::Rc;
use std::sync::Arc;
use runtime_common::runtime_class::RuntimeClass;
use rust_jvm_common::utils::{extract_string_from_utf8, method_name};
use classfile_parser::types::parse_method_descriptor;
use rust_jvm_common::classfile::{ACC_STATIC, ACC_NATIVE};
use runtime_common::java_values::JavaValue;
use std::cell::RefCell;
use std::borrow::Borrow;
use crate::rust_jni::{call, call_impl};
use crate::instructions::invoke::setup_virtual_args;

pub fn run_native_method(
    state: &mut InterpreterState,
    frame: Rc<StackEntry>,
    class: Arc<RuntimeClass>,
    method_i: usize
) {
    //todo only works for static void methods atm
    let classfile = &class.classfile;
    let method = &classfile.methods[method_i];
    assert!(method.access_flags & ACC_NATIVE > 0);
    let descriptor_str = extract_string_from_utf8(&classfile.constant_pool[method.descriptor_index as usize]);
    let parsed = parse_method_descriptor(&class.loader, descriptor_str.as_str()).unwrap();
    let mut args = vec![];
    //todo should have some setup args functions
    if method.access_flags & ACC_STATIC > 0 {
        for _ in parsed.parameter_types {
            args.push(frame.pop());
        }
        args.reverse();
    }else {
        setup_virtual_args(&frame, &parsed, &mut args, (parsed.parameter_types.len() + 1) as u16)
    }
    if method_name(classfile, method) == "desiredAssertionStatus0".to_string() {//todo and descriptor matches and class matches
        frame.push(JavaValue::Boolean(false))
    } else if method_name(classfile, method) == "arraycopy".to_string() {
        let src = args[0].clone().unwrap_array();
        let src_pos = args[1].clone().unwrap_int() as usize;
        let dest = args[2].clone().unwrap_array();
        let dest_pos = args[3].clone().unwrap_int() as usize;
        let length = args[4].clone().unwrap_int() as usize;
        for i in 0..length {
            let borrowed: &RefCell<Vec<JavaValue>> = src.borrow();
            let temp = (borrowed.borrow())[src_pos + i].borrow().clone();
            dest.borrow_mut()[dest_pos + i] = temp;
        }
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
            call_impl(state, frame.clone(), class, args, parsed.return_type, &res_fn).unwrap()
        } else {
            call(state, frame.clone(), class.clone(), method_i, args, parsed.return_type).unwrap()
        };
        match result {
            None => {}
            Some(res) => frame.push(res),
        }
    }
}