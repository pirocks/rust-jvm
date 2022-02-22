use std::ops::Deref;
use std::sync::Arc;

use libffi::middle::Arg;
use libffi::middle::Type;

use jvmti_jni_bindings::{jboolean, jbyte, jchar, jclass, jdouble, jfloat, jint, jlong, JNIEnv, jobject, jshort};
use rust_jvm_common::compressed_classfile::CPDType;

use crate::java_values::JavaValue;
use crate::NewJavaValue;
use crate::runtime_class::RuntimeClass;
use crate::rust_jni::interface::local_frame::new_local_ref;
use crate::rust_jni::native_util::{to_object, to_object_new};

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

pub fn to_native_type(t: &CPDType) -> Type {
    match t {
        CPDType::ByteType => Type::i8(),
        CPDType::CharType => Type::u16(),
        CPDType::DoubleType => Type::f64(),
        CPDType::FloatType => Type::f32(),
        CPDType::IntType => Type::i32(),
        CPDType::LongType => Type::i64(),
        CPDType::ShortType => Type::i16(),
        CPDType::BooleanType => Type::u8(),
        CPDType::Ref(_) => Type::usize(),
        _ => panic!(),
    }
}

pub unsafe fn to_native<'gc_life>(env: *mut JNIEnv, j: NewJavaValue<'gc_life, '_>, t: &CPDType) -> Arg {
    match t {
        CPDType::ByteType => Arg::new(Box::into_raw(Box::new(j.unwrap_int() as i8)).as_ref().unwrap() as &jbyte),
        CPDType::CharType => Arg::new(Box::into_raw(Box::new(j.unwrap_int() as u16)).as_ref().unwrap() as &jchar),
        CPDType::DoubleType => Arg::new(Box::into_raw(Box::new(j.unwrap_double_strict())).as_ref().unwrap() as &jdouble),
        CPDType::FloatType => Arg::new(Box::into_raw(Box::new(j.unwrap_float_strict())).as_ref().unwrap() as &jfloat),
        CPDType::IntType => Arg::new(Box::into_raw(Box::new(j.unwrap_int())).as_ref().unwrap() as &jint),
        CPDType::LongType => Arg::new(Box::into_raw(Box::new(j.unwrap_long_strict())).as_ref().unwrap() as &jlong),
        CPDType::Ref(_) => {
            let object_ptr = new_local_ref(env, to_object_new(j.unwrap_object_alloc()));
            Arg::new(Box::into_raw(Box::new(object_ptr)).as_ref().unwrap() as &jobject)
        }
        CPDType::ShortType => Arg::new(Box::into_raw(Box::new(j.unwrap_int() as i16)).as_ref().unwrap() as &jshort),
        CPDType::BooleanType => Arg::new(Box::into_raw(Box::new(j.unwrap_int() as u8)).as_ref().unwrap() as &jboolean),
        _ => panic!(),
    }
}

pub unsafe fn free_native<'gc_life, 'l>(j: NewJavaValue<'gc_life,'l>, t: &CPDType, to_free: &mut Arg) {
    match t {
        CPDType::ByteType => {
            Box::<jbyte>::from_raw(to_free.0 as *mut jbyte);
        }
        CPDType::CharType => {
            Box::<jchar>::from_raw(to_free.0 as *mut jchar);
        }
        CPDType::DoubleType => {
            Box::<jdouble>::from_raw(to_free.0 as *mut jdouble);
        }
        CPDType::FloatType => {
            Box::<jfloat>::from_raw(to_free.0 as *mut jfloat);
        }
        CPDType::IntType => {
            Box::<jint>::from_raw(to_free.0 as *mut jint);
        }
        CPDType::LongType => {
            Box::<jlong>::from_raw(to_free.0 as *mut jlong);
        }
        CPDType::Ref(_) => {
            Box::<jobject>::from_raw(to_free.0 as *mut jobject);
        }
        CPDType::ShortType => {
            Box::<jshort>::from_raw(to_free.0 as *mut jshort);
        }
        CPDType::BooleanType => {
            Box::<jshort>::from_raw(to_free.0 as *mut jshort);
        }
        _ => panic!(),
    }
}