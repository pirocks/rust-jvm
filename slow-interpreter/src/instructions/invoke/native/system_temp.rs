use std::borrow::Borrow;
use std::cell::Ref;

use jvmti_jni_bindings::jint;

use crate::java_values::JavaValue;

pub fn system_array_copy(args: &mut Vec<JavaValue>) {
    let src_o = args[0].clone().unwrap_object();
    let src = src_o.as_ref().unwrap().unwrap_array();
    let src_pos = args[1].clone().unwrap_int();
    let dest_o = args[2].clone().unwrap_object();
    let dest = dest_o.as_ref().unwrap().unwrap_array();
    let dest_pos = args[3].clone().unwrap_int();
    let length = args[4].clone().unwrap_int();
    if src_pos < 0
        || dest_pos < 0
        || length < 0
        || src_pos + length > src.elems.borrow().len() as i32
        || dest_pos + length > dest.elems.borrow().len() as i32 {
        unimplemented!()
    }
    let mut to_copy = vec![];
    for i in 0..(length as usize) {
        let borrowed: Ref<Vec<JavaValue>> = src.elems.borrow();
        let temp = (borrowed.borrow())[src_pos as usize + i].borrow().clone();
        to_copy.push(temp);
    }
    for i in 0..(length as usize) {
        let mut borrowed = dest.elems.borrow_mut();
        borrowed[dest_pos as usize + i] = to_copy[i].clone();
    }
}
