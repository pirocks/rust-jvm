use std::borrow::Borrow;

use crate::interpreter_state::InterpreterStateGuard;
use crate::java_values::JavaValue;
use crate::jvm_state::JVMState;
use crate::utils::throw_npe;

pub fn system_array_copy(jvm: &JVMState, int_state: &mut InterpreterStateGuard, args: &mut Vec<JavaValue>) {
    let src_o = args[0].clone().unwrap_object();
    let src = match src_o.as_ref() {
        Some(x) => x,
        None => return throw_npe(jvm, int_state),
    }.unwrap_array();
    let src_pos = args[1].clone().unwrap_int();
    let dest_o = args[2].clone().unwrap_object();
    let dest = match dest_o.as_ref() {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state)
        },
    }.unwrap_array();
    let dest_pos = args[3].clone().unwrap_int();
    let length = args[4].clone().unwrap_int();
    if src_pos < 0
        || dest_pos < 0
        || length < 0
        || src_pos + length > src.mut_array().len() as i32
        || dest_pos + length > dest.mut_array().len() as i32 {
        unimplemented!()
    }
    let mut to_copy = vec![];
    for i in 0..(length as usize) {
        let borrowed = src.mut_array();
        let temp = (borrowed)[src_pos as usize + i].borrow().clone();
        to_copy.push(temp);
    }
    for i in 0..(length as usize) {
        let borrowed = dest.mut_array();
        borrowed[dest_pos as usize + i] = to_copy[i].clone();
    }
}
