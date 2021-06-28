use std::ops::Deref;
use std::sync::Arc;

use libffi::middle::Arg;
use libffi::middle::Type;

use jvmti_jni_bindings::{jboolean, jbyte, jchar, jclass, jdouble, jfloat, jint, jlong, jobject, jshort};
use rust_jvm_common::ptype::PType;

use crate::java_values::JavaValue;
use crate::runtime_class::RuntimeClass;
use crate::rust_jni::native_util::to_object;

pub fn runtime_class_to_native<'gc_life>(runtime_class: Arc<RuntimeClass<'gc_life>>) -> Arg {
    let boxed_arc = Box::new(runtime_class);
    let arc_pointer = Box::into_raw(boxed_arc);
    let pointer_ref = Box::leak(Box::new(arc_pointer));
    Arg::new(pointer_ref)
}


pub unsafe fn native_to_runtime_class<'gc_life>(clazz: jclass) -> Arc<RuntimeClass<'gc_life>> {
    let boxed_arc = Box::from_raw(clazz as *mut Arc<RuntimeClass<'gc_life>>);
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


pub unsafe fn to_native<'gc_life>(j: JavaValue<'gc_life>, t: &PType) -> Arg {
    match t {
        PType::ByteType => {
            Arg::new(Box::into_raw(Box::new(j.unwrap_int() as i8)).as_ref().unwrap() as &jbyte)
        }
        PType::CharType => {
            Arg::new(Box::into_raw(Box::new(j.unwrap_int() as u16)).as_ref().unwrap() as &jchar)
        }
        PType::DoubleType => {
            Arg::new(Box::into_raw(Box::new(j.unwrap_double())).as_ref().unwrap() as &jdouble)
        }
        PType::FloatType => {
            Arg::new(Box::into_raw(Box::new(j.unwrap_float())).as_ref().unwrap() as &jfloat)
        }
        PType::IntType => {
            Arg::new(Box::into_raw(Box::new(j.unwrap_int())).as_ref().unwrap() as &jint)
        }
        PType::LongType => {
            Arg::new(Box::into_raw(Box::new(j.unwrap_long())).as_ref().unwrap() as &jlong)
        }
        PType::Ref(_) => {
            let object_ptr = to_object(j.unwrap_object());
            Arg::new(Box::into_raw(Box::new(object_ptr)).as_ref().unwrap() as &jobject)
        }
        PType::ShortType => {
            Arg::new(Box::into_raw(Box::new(j.unwrap_int() as i16)).as_ref().unwrap() as &jshort)
        }
        PType::BooleanType => {
            Arg::new(Box::into_raw(Box::new(j.unwrap_int() as u8)).as_ref().unwrap() as &jboolean)
        }
        _ => panic!(),
    }
}


pub unsafe fn free_native<'gc_life>(_j: JavaValue<'gc_life>, t: &PType, to_free: &mut Arg) {
    match t {
        PType::ByteType => {
            Box::<jbyte>::from_raw(to_free.0 as *mut jbyte);
        }
        PType::CharType => {
            Box::<jchar>::from_raw(to_free.0 as *mut jchar);
        }
        PType::DoubleType => {
            Box::<jdouble>::from_raw(to_free.0 as *mut jdouble);
        }
        PType::FloatType => {
            Box::<jfloat>::from_raw(to_free.0 as *mut jfloat);
        }
        PType::IntType => {
            Box::<jint>::from_raw(to_free.0 as *mut jint);
        }
        PType::LongType => {
            Box::<jlong>::from_raw(to_free.0 as *mut jlong);
        }
        PType::Ref(_) => {
            Box::<jobject>::from_raw(to_free.0 as *mut jobject);
        }
        PType::ShortType => {
            Box::<jshort>::from_raw(to_free.0 as *mut jshort);
        }
        PType::BooleanType => {
            Box::<jshort>::from_raw(to_free.0 as *mut jshort);
        }
        _ => panic!(),
    }
}
