use std::ops::Deref;
use std::sync::Arc;

use libffi::middle::Arg;
use libffi::middle::Type;

use jvmti_jni_bindings::{jboolean, jbyte, jchar, jclass, jdouble, jfloat, jint, jlong, JNIEnv, jshort};
use runtime_class_stuff::RuntimeClass;
use rust_jvm_common::compressed_classfile::CPDType;

use crate::{JavaValueCommon, NewJavaValue};
use crate::rust_jni::ffi_arg_holder::ArgBoxesToFree;
use crate::rust_jni::interface::local_frame::new_local_ref;
use crate::rust_jni::native_util::to_object_new;

pub fn runtime_class_to_native<'gc>(runtime_class: Arc<RuntimeClass<'gc>>) -> Arg {
    let boxed_arc = Box::new(runtime_class);
    let arc_pointer = Box::into_raw(boxed_arc);
    let pointer_ref = Box::leak(Box::new(arc_pointer));
    Arg::new(pointer_ref)
}

pub unsafe fn native_to_runtime_class<'gc>(clazz: jclass) -> Arc<RuntimeClass<'gc>> {
    let boxed_arc = Box::from_raw(clazz as *mut Arc<RuntimeClass<'gc>>);
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
        CPDType::Class(_) => Type::usize(),
        CPDType::Array { .. } => Type::usize(),
        _ => panic!(),
    }
}

pub unsafe fn to_native<'gc>(env: *mut JNIEnv, arg_boxes: &mut ArgBoxesToFree, j: NewJavaValue<'gc, '_>, t: &CPDType) -> Arg {
    match t {
        CPDType::ByteType => Arg::new(arg_boxes.new_generic(j.unwrap_int() as jbyte).as_ref()),
        CPDType::CharType => Arg::new(arg_boxes.new_generic(j.unwrap_int() as jchar).as_ref()),
        CPDType::DoubleType => Arg::new(arg_boxes.new_generic(j.unwrap_double_strict() as jdouble).as_ref()),
        CPDType::FloatType => Arg::new(arg_boxes.new_generic(j.unwrap_float_strict() as jfloat).as_ref()),
        CPDType::IntType => Arg::new(arg_boxes.new_generic(j.unwrap_int() as jint).as_ref()),
        CPDType::LongType => Arg::new(arg_boxes.new_generic(j.unwrap_long_strict() as jlong).as_ref()),
        CPDType::ShortType => Arg::new(arg_boxes.new_generic(j.unwrap_int() as jshort).as_ref()),
        CPDType::BooleanType => Arg::new(arg_boxes.new_generic(j.unwrap_int() as jboolean).as_ref()),
        CPDType::Array { .. } | CPDType::Class(_) => {
            let object_ptr = new_local_ref(env, to_object_new(j.unwrap_object_alloc()));
            Arg::new(arg_boxes.new_generic(object_ptr).as_ref())
        }
        _ => panic!(),
    }
}
