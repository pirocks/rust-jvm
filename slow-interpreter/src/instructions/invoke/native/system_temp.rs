use std::borrow::Borrow;
use std::cell::Ref;

use crate::java_values::JavaValue;

pub fn system_array_copy(args: &mut Vec<JavaValue>) -> () {
    let src_o = args[0].clone().unwrap_object();
    let src = src_o.as_ref().unwrap().unwrap_array();
    let src_pos = args[1].clone().unwrap_int() as usize;
    let dest_o = args[2].clone().unwrap_object();
    let dest = dest_o.as_ref().unwrap().unwrap_array();
    let dest_pos = args[3].clone().unwrap_int() as usize;
    let length = args[4].clone().unwrap_int() as usize;
//    if Arc::ptr_eq(src_o.as_ref().unwrap(),dest_o.as_ref().unwrap()) && src_pos == dest_pos{
    //prevents issues with a refcell already being borrowed, and then being mutably borrowed
//        return;
//    }
    for i in 0..length {
        let borrowed: Ref<Vec<JavaValue>> = src.elems.borrow();
        let temp = (borrowed.borrow())[src_pos + i].borrow().clone();
        std::mem::drop(borrowed);
        dest.elems.borrow_mut()[dest_pos + i] = temp;
    }
}
