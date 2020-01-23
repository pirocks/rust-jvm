use crate::InterpreterState;
use crate::CallStackEntry;
use std::rc::Rc;
use std::sync::Arc;
use runtime_common::runtime_class::RuntimeClass;
use rust_jvm_common::utils::{extract_string_from_utf8, method_name};
use classfile_parser::types::parse_method_descriptor;
use rust_jvm_common::classfile::ACC_STATIC;
use runtime_common::java_values::JavaValue;
use std::cell::RefCell;
use std::borrow::Borrow;
use crate::rust_jni::call;

pub fn run_native_method(
    state: &mut InterpreterState,
    frame: Rc<CallStackEntry>,
    class: Arc<RuntimeClass>,
    method_i: usize
) {
    //todo only works for static void methods atm
    let classfile = &class.classfile;
    let method = &classfile.methods[method_i];
    assert!(method.access_flags & ACC_STATIC > 0);
    let descriptor_str = extract_string_from_utf8(&classfile.constant_pool[method.descriptor_index as usize]);
    let parsed = parse_method_descriptor(&class.loader, descriptor_str.as_str()).unwrap();
    let mut args = vec![];
//    dbg!(&frame.operand_stack);
    for _ in parsed.parameter_types {
        args.push(frame.operand_stack.borrow_mut().pop().unwrap());
    }
    args.reverse();
    if method_name(classfile, method) == "desiredAssertionStatus0".to_string() {//todo and descriptor matches and class matches
        frame.operand_stack.borrow_mut().push(JavaValue::Boolean(false))
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
        match call(state, frame.clone(),class.clone(), method_i, args, parsed.return_type) {
            None => {}
            Some(res) => frame.operand_stack.borrow_mut().push(res),
        }
    }
}