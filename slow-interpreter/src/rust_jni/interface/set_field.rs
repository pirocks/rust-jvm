use crate::rust_jni::native_util::{from_object, from_jclass};
use jvmti_jni_bindings::{jboolean, jfieldID, jobject, JNIEnv, jlong, jint, jclass, jbyte, jchar, jshort, jdouble, jfloat};
use std::ops::DerefMut;
use crate::rust_jni::interface::util::{FieldID};
use crate::java_values::JavaValue;

unsafe fn set_field(obj: jobject, field_id_raw: jfieldID, val: JavaValue) {
    let field_id = Box::leak(Box::from_raw(field_id_raw as *mut FieldID));
    let view = &field_id.class.view();
    let name = view.field(field_id.field_i as usize).field_name();
    let notnull = from_object(obj).unwrap();
    let mut field_borrow = notnull.unwrap_normal_object().fields.borrow_mut();
    field_borrow.deref_mut().insert(name, val);
}

pub unsafe extern "C" fn set_boolean_field(_env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID, val: jboolean) {
    set_field(obj,field_id_raw,JavaValue::Boolean(val))
}

pub unsafe extern "C" fn set_byte_field(_env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID, val: jbyte) {
    set_field(obj,field_id_raw,JavaValue::Byte(val))
}

pub unsafe extern "C" fn set_short_field(_env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID, val: jshort) {
    set_field(obj,field_id_raw,JavaValue::Short(val))
}

pub unsafe extern "C" fn set_char_field(_env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID, val: jchar) {
    set_field(obj,field_id_raw,JavaValue::Char(val))
}

pub unsafe extern "C" fn set_int_field(_env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID, val: jint) {
    set_field(obj, field_id_raw, JavaValue::Int(val));
}

pub unsafe extern "C" fn set_long_field(_env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID, val: jlong) {
    set_field(obj,field_id_raw,JavaValue::Long(val));
}

pub unsafe extern "C" fn set_float_field(_env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID, val: jfloat) {
    set_field(obj, field_id_raw, JavaValue::Float(val));
}

pub unsafe extern "C" fn set_double_field(_env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID, val: jdouble) {
    set_field(obj, field_id_raw, JavaValue::Double(val));
}

pub unsafe extern "C" fn set_object_field(_env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID, val: jobject) {
    set_field(obj, field_id_raw, JavaValue::Object(from_object(val)));
}


pub unsafe extern "C" fn set_static_object_field(_env: *mut JNIEnv, clazz: jclass, field_id_raw: jfieldID, value: jobject) {
    let field_id = Box::leak(Box::from_raw(field_id_raw as *mut FieldID));//todo leak
    let value = from_object(value);
    let view = &field_id.class.view();
    let field_name = view.field(field_id.field_i).field_name();
    let static_class = from_jclass(clazz).as_runtime_class();
    static_class.static_vars().insert(field_name, JavaValue::Object(value));
}



