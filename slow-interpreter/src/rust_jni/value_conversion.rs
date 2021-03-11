use std::ffi::c_void;
use std::ops::Deref;
use std::sync::Arc;

use libffi::middle::Arg;
use libffi::middle::Type;

use jvmti_jni_bindings::jclass;
use rust_jvm_common::ptype::PType;

use crate::java_values::{JavaValue, Object};
use crate::runtime_class::RuntimeClass;
use crate::rust_jni::native_util::to_object;

pub fn runtime_class_to_native(runtime_class: Arc<RuntimeClass>) -> Arg {
    let boxed_arc = Box::new(runtime_class);
    let arc_pointer = Box::into_raw(boxed_arc);
    let pointer_ref = Box::leak(Box::new(arc_pointer));
    Arg::new(pointer_ref)
}


pub unsafe fn native_to_runtime_class(clazz: jclass) -> Arc<RuntimeClass> {
    let boxed_arc = Box::from_raw(clazz as *mut Arc<RuntimeClass>);
    boxed_arc.deref().clone()
}


pub fn to_native_type(t: &PType) -> Type {

    match t {
        PType::ByteType => Type::i8(),
        PType::CharType => Type::u16(),
        PType::DoubleType => Type::f64(),
        PType::FloatType => Type::f32(),
        PType::IntType => Type::i32(),
        PType::LongType => Type::i64(),
        PType::ShortType => Type::i16(),
        PType::BooleanType => Type::u8(),
        PType::Ref(_) => Type::pointer(),
        _ => panic!(),
    }
}


pub fn to_native(j: JavaValue, t: &PType) -> Arg {
    match t {
        PType::ByteType => {
            Arg::new(Box::leak(Box::new(j.unwrap_int() as i8)))//todo free after call
        }
        PType::CharType => {
            Arg::new(Box::leak(Box::new(j.unwrap_int() as u16)))//todo free after call
        }
        PType::DoubleType => {
            Arg::new(Box::leak(Box::new(j.unwrap_double())))//todo free after call
        }
        PType::FloatType => {
            Arg::new(Box::leak(Box::new(j.unwrap_float())))//todo free after call
        }
        PType::IntType => {
            Arg::new(Box::leak(Box::new(j.unwrap_int())))//todo free after call
        }
        PType::LongType => {
            Arg::new(Box::leak(Box::new(j.unwrap_long())))//todo free after call
        }
        PType::Ref(_) => {
            match j.unwrap_object() {
                None => Arg::new(&(std::ptr::null() as *const Object)),
                Some(op) => {
                    unsafe {
                        let object_ptr = to_object(op.into()) as *mut c_void;
                        let ref_box = Box::new(object_ptr);
                        //todo don;t forget to free later, and/or do this with lifetimes
                        Arg::new/*::<*mut c_void>*/(Box::leak(ref_box))
                    }
                }
            }
        }
        PType::ShortType => {
            Arg::new(Box::leak(Box::new(j.unwrap_int() as i16)))//todo free after call
        }
        PType::BooleanType => {
            Arg::new(Box::leak(Box::new(j.unwrap_int() as u8)))//todo free after call
        }
        _ => panic!(),
    }
}


pub fn free_native(j: JavaValue, t: &PType, to_free: &mut Arg) {
    match t {
        PType::ByteType => {
            Box::from_raw::<i8>(to_free.0)
        }
        PType::CharType => {
            Box::from_raw::<u16>(to_free.0)
        }
        PType::DoubleType => {
            Arg::new(Box::leak(Box::new(j.unwrap_double())))//todo free after call
        }
        PType::FloatType => {
            Arg::new(Box::leak(Box::new(j.unwrap_float())))//todo free after call
        }
        PType::IntType => {
            Arg::new(Box::leak(Box::new(j.unwrap_int())))//todo free after call
        }
        PType::LongType => {
            Arg::new(Box::leak(Box::new(j.unwrap_long())))//todo free after call
        }
        PType::Ref(_) => {
            match j.unwrap_object() {
                None => Arg::new(&(std::ptr::null() as *const Object)),
                Some(op) => {
                    unsafe {
                        let object_ptr = to_object(op.into()) as *mut c_void;
                        let ref_box = Box::new(object_ptr);
                        //todo don;t forget to free later, and/or do this with lifetimes
                        Arg::new/*::<*mut c_void>*/(Box::leak(ref_box))
                    }
                }
            }
        }
        PType::ShortType => {
            Arg::new(Box::leak(Box::new(j.unwrap_int() as i16)))//todo free after call
        }
        PType::BooleanType => {
            Arg::new(Box::leak(Box::new(j.unwrap_int() as u8)))//todo free after call
        }
        _ => panic!(),
    }
}
