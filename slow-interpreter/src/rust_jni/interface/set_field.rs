use std::ops::Deref;
use jvmti_jni_bindings::{jboolean, jbyte, jchar, jclass, jdouble, jfieldID, jfloat, jint, jlong, JNIEnv, jobject, jshort};

use crate::new_java_values::NewJavaValueHandle;
use crate::{JavaValueCommon, NewJavaValue};
use crate::runtime_class::static_vars;
use crate::rust_jni::interface::{get_interpreter_state, get_state};
use crate::rust_jni::native_util::{from_jclass, from_object_new};
use crate::utils::throw_npe;

unsafe fn set_field<'gc>(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID, val: NewJavaValue<'gc,'_>) {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let (rc, field_i) = jvm.field_table.write().unwrap().lookup(field_id_raw as usize);
    let view = rc.view();
    let name = view.field(field_i as usize).field_name();
    let notnull = match from_object_new(jvm, obj) {
        Some(x) => x,
        None => return throw_npe(jvm, /*int_state*/todo!()),
    };
    notnull.unwrap_normal_object_ref().set_var(&rc, name, val);
}

pub unsafe extern "C" fn set_boolean_field(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID, val: jboolean) {
    set_field(env, obj, field_id_raw, NewJavaValue::Boolean(val))
}

pub unsafe extern "C" fn set_byte_field(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID, val: jbyte) {
    set_field(env, obj, field_id_raw, NewJavaValue::Byte(val))
}

pub unsafe extern "C" fn set_short_field(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID, val: jshort) {
    set_field(env, obj, field_id_raw, NewJavaValue::Short(val))
}

pub unsafe extern "C" fn set_char_field(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID, val: jchar) {
    set_field(env, obj, field_id_raw, NewJavaValue::Char(val))
}

pub unsafe extern "C" fn set_int_field(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID, val: jint) {
    set_field(env, obj, field_id_raw, NewJavaValue::Int(val));
}

pub unsafe extern "C" fn set_long_field(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID, val: jlong) {
    set_field(env, obj, field_id_raw, NewJavaValue::Long(val));
}

pub unsafe extern "C" fn set_float_field(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID, val: jfloat) {
    set_field(env, obj, field_id_raw, NewJavaValue::Float(val));
}

pub unsafe extern "C" fn set_double_field(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID, val: jdouble) {
    set_field(env, obj, field_id_raw, NewJavaValue::Double(val));
}

pub unsafe extern "C" fn set_object_field(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID, val: jobject) {
    set_field(env, obj, field_id_raw, NewJavaValueHandle::from_optional_object(from_object_new(get_state(env), val)).as_njv());
}

pub unsafe extern "C" fn set_static_boolean_field(env: *mut JNIEnv, clazz: jclass, field_id_raw: jfieldID, value: jboolean) {
    set_static_field(env, clazz, field_id_raw, NewJavaValueHandle::Boolean(value));
}

pub unsafe extern "C" fn set_static_byte_field(env: *mut JNIEnv, clazz: jclass, field_id_raw: jfieldID, value: jbyte) {
    set_static_field(env, clazz, field_id_raw, NewJavaValueHandle::Byte(value));
}

pub unsafe extern "C" fn set_static_short_field(env: *mut JNIEnv, clazz: jclass, field_id_raw: jfieldID, value: jshort) {
    set_static_field(env, clazz, field_id_raw, NewJavaValueHandle::Short(value));
}

pub unsafe extern "C" fn set_static_char_field(env: *mut JNIEnv, clazz: jclass, field_id_raw: jfieldID, value: jchar) {
    set_static_field(env, clazz, field_id_raw, NewJavaValueHandle::Char(value));
}

pub unsafe extern "C" fn set_static_int_field(env: *mut JNIEnv, clazz: jclass, field_id_raw: jfieldID, value: jint) {
    set_static_field(env, clazz, field_id_raw, NewJavaValueHandle::Int(value));
}

pub unsafe extern "C" fn set_static_long_field(env: *mut JNIEnv, clazz: jclass, field_id_raw: jfieldID, value: jlong) {
    set_static_field(env, clazz, field_id_raw, NewJavaValueHandle::Long(value));
}

pub unsafe extern "C" fn set_static_float_field(env: *mut JNIEnv, clazz: jclass, field_id_raw: jfieldID, value: jfloat) {
    set_static_field(env, clazz, field_id_raw, NewJavaValueHandle::Float(value));
}

pub unsafe extern "C" fn set_static_double_field(env: *mut JNIEnv, clazz: jclass, field_id_raw: jfieldID, value: jdouble) {
    set_static_field(env, clazz, field_id_raw, NewJavaValueHandle::Double(value));
}

pub unsafe extern "C" fn set_static_object_field(env: *mut JNIEnv, clazz: jclass, field_id_raw: jfieldID, value: jobject) {
    let jvm = get_state(env);
    let value = from_object_new(jvm, value);
    set_static_field(env, clazz, field_id_raw, NewJavaValueHandle::from_optional_object(value));
}

unsafe fn set_static_field<'gc>(env: *mut JNIEnv, clazz: jclass, field_id_raw: jfieldID, value: NewJavaValueHandle<'gc>) {
    let jvm = get_state(env);
    //todo create a field conversion function.
    let (rc, field_i) = jvm.field_table.read().unwrap().lookup(field_id_raw as usize);
    let view = &rc.view();
    let field_name = view.field(field_i as usize).field_name();
    let static_class = from_jclass(jvm, clazz).as_runtime_class(jvm);
    static_vars(static_class.deref(),jvm).set(field_name, value);
}