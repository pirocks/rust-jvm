use jvmti_jni_bindings::{jboolean, jbyte, jchar, jclass, jdouble, jfieldID, jfloat, jint, jlong, JNIEnv, jobject, jshort};

use crate::java_values::JavaValue;
use crate::rust_jni::native_util::{from_jclass, from_object, get_interpreter_state, get_state};
use crate::utils::throw_npe;

unsafe fn set_field<'gc_life>(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID, val: JavaValue<'gc_life>) {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let (rc, field_i) = jvm.field_table.write().unwrap().lookup(field_id_raw as usize);
    let view = rc.view();
    let name = view.field(field_i as usize).field_name();
    let notnull = match from_object(jvm, obj) {
        Some(x) => x,
        None => return throw_npe(jvm, int_state),
    };
    notnull.unwrap_normal_object().set_var(rc, name, val);
}

pub unsafe extern "C" fn set_boolean_field(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID, val: jboolean) {
    set_field(env, obj, field_id_raw, JavaValue::Boolean(val))
}

pub unsafe extern "C" fn set_byte_field(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID, val: jbyte) {
    set_field(env, obj, field_id_raw, JavaValue::Byte(val))
}

pub unsafe extern "C" fn set_short_field(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID, val: jshort) {
    set_field(env, obj, field_id_raw, JavaValue::Short(val))
}

pub unsafe extern "C" fn set_char_field(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID, val: jchar) {
    set_field(env, obj, field_id_raw, JavaValue::Char(val))
}

pub unsafe extern "C" fn set_int_field(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID, val: jint) {
    set_field(env, obj, field_id_raw, JavaValue::Int(val));
}

pub unsafe extern "C" fn set_long_field(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID, val: jlong) {
    set_field(env, obj, field_id_raw, JavaValue::Long(val));
}

pub unsafe extern "C" fn set_float_field(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID, val: jfloat) {
    set_field(env, obj, field_id_raw, JavaValue::Float(val));
}

pub unsafe extern "C" fn set_double_field(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID, val: jdouble) {
    set_field(env, obj, field_id_raw, JavaValue::Double(val));
}

pub unsafe extern "C" fn set_object_field(env: *mut JNIEnv, obj: jobject, field_id_raw: jfieldID, val: jobject) {
    set_field(env, obj, field_id_raw, JavaValue::Object(from_object(get_state(env), val)));
}


pub unsafe extern "C" fn set_static_boolean_field(env: *mut JNIEnv, clazz: jclass, field_id_raw: jfieldID, value: jboolean) {
    set_static_field(env, clazz, field_id_raw, JavaValue::Boolean(value));
}

pub unsafe extern "C" fn set_static_byte_field(env: *mut JNIEnv, clazz: jclass, field_id_raw: jfieldID, value: jbyte) {
    set_static_field(env, clazz, field_id_raw, JavaValue::Byte(value));
}

pub unsafe extern "C" fn set_static_short_field(env: *mut JNIEnv, clazz: jclass, field_id_raw: jfieldID, value: jshort) {
    set_static_field(env, clazz, field_id_raw, JavaValue::Short(value));
}

pub unsafe extern "C" fn set_static_char_field(env: *mut JNIEnv, clazz: jclass, field_id_raw: jfieldID, value: jchar) {
    set_static_field(env, clazz, field_id_raw, JavaValue::Char(value));
}

pub unsafe extern "C" fn set_static_int_field(env: *mut JNIEnv, clazz: jclass, field_id_raw: jfieldID, value: jint) {
    set_static_field(env, clazz, field_id_raw, JavaValue::Int(value));
}

pub unsafe extern "C" fn set_static_long_field(env: *mut JNIEnv, clazz: jclass, field_id_raw: jfieldID, value: jlong) {
    set_static_field(env, clazz, field_id_raw, JavaValue::Long(value));
}

pub unsafe extern "C" fn set_static_float_field(env: *mut JNIEnv, clazz: jclass, field_id_raw: jfieldID, value: jfloat) {
    set_static_field(env, clazz, field_id_raw, JavaValue::Float(value));
}

pub unsafe extern "C" fn set_static_double_field(env: *mut JNIEnv, clazz: jclass, field_id_raw: jfieldID, value: jdouble) {
    set_static_field(env, clazz, field_id_raw, JavaValue::Double(value));
}

pub unsafe extern "C" fn set_static_object_field(env: *mut JNIEnv, clazz: jclass, field_id_raw: jfieldID, value: jobject) {
    let jvm = get_state(env);
    let value = from_object(jvm, value);
    set_static_field(env, clazz, field_id_raw, JavaValue::Object(value));
}

unsafe fn set_static_field<'gc_life>(env: *mut JNIEnv, clazz: jclass, field_id_raw: jfieldID, value: JavaValue<'gc_life>) {
    let jvm = get_state(env);
    //todo create a field conversion function.
    let (rc, field_i) = jvm.field_table.read().unwrap().lookup(field_id_raw as usize);
    let view = &rc.view();
    let field_name = view.field(field_i as usize).field_name();
    let static_class = from_jclass(jvm, clazz).as_runtime_class(jvm);
    static_class.static_vars().insert(field_name, value);
}



