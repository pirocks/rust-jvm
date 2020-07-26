#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

//all of these functions should be implemented in libjvm
use std::mem::transmute;

use crate::java_values::JavaValue;
use crate::JVMState;

pub fn get_object_volatile(args: &mut Vec<JavaValue>) -> Option<JavaValue> {
    let temp = args[1].unwrap_object().unwrap();
    let array_idx = args[2].unwrap_long() as usize;
    let res = &temp.unwrap_array().elems.borrow()[array_idx];
    res.clone().into()
}

pub fn freeMemory(args: &mut Vec<JavaValue>) -> Option<JavaValue> {
    unsafe {
        libc::free(transmute(args[1].unwrap_long()))
    };
    None
}

pub fn getByte__J(args: &mut Vec<JavaValue>) -> Option<JavaValue> {
    unsafe {
        let ptr: *mut i8 = transmute(args[1].unwrap_long());
        JavaValue::Byte(ptr.read()).into()
    }
}

pub fn putLong__JJ(args: &mut Vec<JavaValue>) -> Option<JavaValue> {
    unsafe {
        let ptr: *mut i64 = transmute(args[1].unwrap_long());
        let val = args[2].unwrap_long();
        ptr.write(val);
    }
    None
}

pub fn allocate_memory(args: &mut Vec<JavaValue>) -> Option<JavaValue> {
    let res: i64 = unsafe {
        transmute(libc::malloc(transmute(args[1].unwrap_long())))
    };
    JavaValue::Long(res).into()
}


pub fn shouldBeInitialized(state: &'static JVMState, args: &mut Vec<JavaValue>) -> Option<JavaValue> {
    let class_name_to_check = args[1].cast_class().as_type();
    JavaValue::Boolean(state.classes.initialized_classes.read().unwrap().get(&class_name_to_check).is_some() as u8).into()
}
