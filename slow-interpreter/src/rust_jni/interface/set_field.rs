use crate::rust_jni::native_util::{from_object, from_jclass, get_state};
use jvmti_jni_bindings::{jboolean, jfieldID, jobject, JNIEnv, jlong, jint, jclass, jbyte, jchar, jshort, jdouble, jfloat};
use std::ops::DerefMut;
use crate::java_values::JavaValue;
use std::mem::transmute;

unsafe fn set_field(env: *mut JNIEnv,obj: jobject, field_id_raw: jfieldID, val: JavaValue) {
    let jvm = get_state(env);
    let (rc,field_i) = jvm.field_table.write().unwrap().lookup(transmute(field_id_raw));
    let view = rc.view();
    let name = view.field(field_i as usize).field_name();
    let notnull = from_object(obj).unwrap();
    let mut field_borrow = notnull.unwrap_normal_object().fields.borrow_mut();
    field_borrow.deref_mut().insert(name, val);
}

pub unsafe extern "C" fn set_boolean_field(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID, val: jboolean) {
    set_field(env,obj,field_id_raw,JavaValue::Boolean(val))
}

pub unsafe extern "C" fn set_byte_field(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID, val: jbyte) {
    set_field(env,obj,field_id_raw,JavaValue::Byte(val))
}

pub unsafe extern "C" fn set_short_field(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID, val: jshort) {
    set_field(env,obj,field_id_raw,JavaValue::Short(val))
}

pub unsafe extern "C" fn set_char_field(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID, val: jchar) {
    set_field(env,obj,field_id_raw,JavaValue::Char(val))
}

pub unsafe extern "C" fn set_int_field(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID, val: jint) {
    set_field(env,obj, field_id_raw, JavaValue::Int(val));
}

pub unsafe extern "C" fn set_long_field(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID, val: jlong) {
    set_field(env,obj,field_id_raw,JavaValue::Long(val));
}

pub unsafe extern "C" fn set_float_field(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID, val: jfloat) {
    set_field(env,obj, field_id_raw, JavaValue::Float(val));
}

pub unsafe extern "C" fn set_double_field(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID, val: jdouble) {
    set_field(env,obj, field_id_raw, JavaValue::Double(val));
}

pub unsafe extern "C" fn set_object_field(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID, val: jobject) {
    set_field(env,obj, field_id_raw, JavaValue::Object(from_object(val)));
}


pub unsafe extern "C" fn set_static_object_field(env: *mut JNIEnv, clazz: jclass, field_id_raw: jfieldID, value: jobject) {
    let jvm = get_state(env);
    let (rc, field_i) = jvm.field_table.read().unwrap().lookup(transmute(field_id_raw));
    let value = from_object(value);
    let view = &rc.view();
    let field_name = view.field(field_i as usize).field_name();
    let static_class = from_jclass(clazz).as_runtime_class();
    static_class.static_vars().insert(field_name, JavaValue::Object(value));
}



