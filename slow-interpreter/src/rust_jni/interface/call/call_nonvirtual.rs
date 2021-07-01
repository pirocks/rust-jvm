use std::ffi::VaList;
use std::mem::transmute;
use std::ptr::null_mut;

use jvmti_jni_bindings::{jboolean, jbyte, jchar, jclass, jdouble, jfloat, jint, jlong, jmethodID, JNIEnv, jobject, jshort, jvalue};
use rust_jvm_common::classnames::ClassName;

use crate::instructions::invoke::special::invoke_special_impl;
use crate::interpreter::WasException;
use crate::java_values::JavaValue;
use crate::method_table::MethodId;
use crate::rust_jni::interface::call::{push_params_onto_frame, VarargProvider};
use crate::rust_jni::interface::local_frame::new_local_ref_public;
use crate::rust_jni::native_util::{from_object, get_interpreter_state, get_state};

pub unsafe extern "C" fn call_nonvirtual_object_method(env: *mut JNIEnv, obj: jobject, clazz: jclass, method_id: jmethodID, mut l: ...) -> jobject {
    let mut vararg_provider = VarargProvider::Dots(&mut l);
    new_local_ref_public(match call_non_virtual(env, obj, clazz, method_id, &mut vararg_provider, false) {
        Ok(res) => res,
        Err(_) => return null_mut()
    }.unwrap_object(), get_interpreter_state(env))
}

pub unsafe extern "C" fn call_nonvirtual_object_method_v(env: *mut JNIEnv, obj: jobject, clazz: jclass, method_id: jmethodID, mut args: VaList) -> jobject {
    let mut vararg_provider = VarargProvider::VaList(&mut args);
    new_local_ref_public(match call_non_virtual(env, obj, clazz, method_id, &mut vararg_provider, false) {
        Ok(res) => res,
        Err(WasException {}) => return null_mut()
    }.unwrap_object(), get_interpreter_state(env))
}

pub unsafe extern "C" fn call_nonvirtual_object_method_a(env: *mut JNIEnv, obj: jobject, clazz: jclass, method_id: jmethodID, args: *const jvalue) -> jobject {
    let mut vararg_provider = VarargProvider::Array(args);
    new_local_ref_public(match call_non_virtual(env, obj, clazz, method_id, &mut vararg_provider, false) {
        Ok(res) => res,
        Err(WasException {}) => return null_mut()
    }.unwrap_object(), get_interpreter_state(env))
}

pub unsafe extern "C" fn call_nonvirtual_boolean_method(env: *mut JNIEnv, obj: jobject, clazz: jclass, method_id: jmethodID, mut l: ...) -> jboolean {
    let mut vararg_provider = VarargProvider::Dots(&mut l);
    match call_non_virtual(env, obj, clazz, method_id, &mut vararg_provider, false) {
        Ok(res) => res,
        Err(WasException {}) => return u8::from(0xde)
    }.unwrap_boolean()
}

pub unsafe extern "C" fn call_nonvirtual_boolean_method_v(env: *mut JNIEnv, obj: jobject, clazz: jclass, method_id: jmethodID, mut args: VaList) -> jboolean {
    let mut vararg_provider = VarargProvider::VaList(&mut args);
    match call_non_virtual(env, obj, clazz, method_id, &mut vararg_provider, false) {
        Ok(res) => res,
        Err(WasException {}) => return u8::from(0xde)
    }.unwrap_boolean()
}

pub unsafe extern "C" fn call_nonvirtual_boolean_method_a(env: *mut JNIEnv, obj: jobject, clazz: jclass, method_id: jmethodID, args: *const jvalue) -> jboolean {
    let mut vararg_provider = VarargProvider::Array(args);
    match call_non_virtual(env, obj, clazz, method_id, &mut vararg_provider, false) {
        Ok(res) => res,
        Err(WasException {}) => return u8::from(0xde)
    }.unwrap_boolean()
}

#[allow(overflowing_literals)]
pub unsafe extern "C" fn call_nonvirtual_byte_method(env: *mut JNIEnv, obj: jobject, clazz: jclass, method_id: jmethodID, mut l: ...) -> jbyte {
    let mut vararg_provider = VarargProvider::Dots(&mut l);
    match call_non_virtual(env, obj, clazz, method_id, &mut vararg_provider, false) {
        Ok(res) => res,
        Err(WasException {}) => return 0xdei8
    }.unwrap_byte()
}

#[allow(overflowing_literals)]
pub unsafe extern "C" fn call_nonvirtual_byte_method_v(env: *mut JNIEnv, obj: jobject, clazz: jclass, method_id: jmethodID, mut args: VaList) -> jbyte {
    let mut vararg_provider = VarargProvider::VaList(&mut args);
    match call_non_virtual(env, obj, clazz, method_id, &mut vararg_provider, false) {
        Ok(res) => res,
        Err(WasException {}) => return 0xdei8
    }.unwrap_byte()
}

#[allow(overflowing_literals)]
pub unsafe extern "C" fn call_nonvirtual_byte_method_a(env: *mut JNIEnv, obj: jobject, clazz: jclass, method_id: jmethodID, args: *const jvalue) -> jbyte {
    let mut vararg_provider = VarargProvider::Array(args);
    match call_non_virtual(env, obj, clazz, method_id, &mut vararg_provider, false) {
        Ok(res) => res,
        Err(WasException {}) => return 0xdei8
    }.unwrap_byte()
}

pub unsafe extern "C" fn call_nonvirtual_char_method(env: *mut JNIEnv, obj: jobject, clazz: jclass, method_id: jmethodID, mut l: ...) -> jchar {
    let mut vararg_provider = VarargProvider::Dots(&mut l);
    match call_non_virtual(env, obj, clazz, method_id, &mut vararg_provider, false) {
        Ok(res) => res,
        Err(WasException {}) => return 0xdeadu16
    }.unwrap_char()
}

pub unsafe extern "C" fn call_nonvirtual_char_method_v(env: *mut JNIEnv, obj: jobject, clazz: jclass, method_id: jmethodID, mut args: VaList) -> jchar {
    let mut vararg_provider = VarargProvider::VaList(&mut args);
    match call_non_virtual(env, obj, clazz, method_id, &mut vararg_provider, false) {
        Ok(res) => res,
        Err(WasException {}) => return 0xdeadu16
    }.unwrap_char()
}

pub unsafe extern "C" fn call_nonvirtual_char_method_a(env: *mut JNIEnv, obj: jobject, clazz: jclass, method_id: jmethodID, args: *const jvalue) -> jchar {
    let mut vararg_provider = VarargProvider::Array(args);
    match call_non_virtual(env, obj, clazz, method_id, &mut vararg_provider, false) {
        Ok(res) => res,
        Err(WasException {}) => return 0xdeadu16
    }.unwrap_char()
}

#[allow(overflowing_literals)]
pub unsafe extern "C" fn call_nonvirtual_short_method(env: *mut JNIEnv, obj: jobject, clazz: jclass, method_id: jmethodID, mut l: ...) -> jshort {
    let mut vararg_provider = VarargProvider::Dots(&mut l);
    match call_non_virtual(env, obj, clazz, method_id, &mut vararg_provider, false) {
        Ok(res) => res,
        Err(WasException {}) => return 0xdeadi16
    }.unwrap_short()
}

#[allow(overflowing_literals)]
pub unsafe extern "C" fn call_nonvirtual_short_method_v(env: *mut JNIEnv, obj: jobject, clazz: jclass, method_id: jmethodID, mut args: VaList) -> jshort {
    let mut vararg_provider = VarargProvider::VaList(&mut args);
    match call_non_virtual(env, obj, clazz, method_id, &mut vararg_provider, false) {
        Ok(res) => res,
        Err(WasException {}) => return 0xdeadi16
    }.unwrap_short()
}

#[allow(overflowing_literals)]
pub unsafe extern "C" fn call_nonvirtual_short_method_a(env: *mut JNIEnv, obj: jobject, clazz: jclass, method_id: jmethodID, args: *const jvalue) -> jshort {
    let mut vararg_provider = VarargProvider::Array(args);
    match call_non_virtual(env, obj, clazz, method_id, &mut vararg_provider, false) {
        Ok(res) => res,
        Err(WasException {}) => return 0xdeadi16
    }.unwrap_short()
}

#[allow(overflowing_literals)]
pub unsafe extern "C" fn call_nonvirtual_int_method(env: *mut JNIEnv, obj: jobject, clazz: jclass, method_id: jmethodID, mut l: ...) -> jint {
    let mut vararg_provider = VarargProvider::Dots(&mut l);
    match call_non_virtual(env, obj, clazz, method_id, &mut vararg_provider, false) {
        Ok(res) => res,
        Err(WasException {}) => return i32::from(0xdeaddead)
    }.unwrap_int()
}

#[allow(overflowing_literals)]
pub unsafe extern "C" fn call_nonvirtual_int_method_v(env: *mut JNIEnv, obj: jobject, clazz: jclass, method_id: jmethodID, mut args: VaList) -> jint {
    let mut vararg_provider = VarargProvider::VaList(&mut args);
    match call_non_virtual(env, obj, clazz, method_id, &mut vararg_provider, false) {
        Ok(res) => res,
        Err(WasException {}) => return i32::from(0xdeaddead)
    }.unwrap_int()
}

#[allow(overflowing_literals)]
pub unsafe extern "C" fn call_nonvirtual_int_method_a(env: *mut JNIEnv, obj: jobject, clazz: jclass, method_id: jmethodID, args: *const jvalue) -> jint {
    let mut vararg_provider = VarargProvider::Array(args);
    match call_non_virtual(env, obj, clazz, method_id, &mut vararg_provider, false) {
        Ok(res) => res,
        Err(WasException {}) => return i32::from(0xdeaddead)
    }.unwrap_int()
}

#[allow(overflowing_literals)]
pub unsafe extern "C" fn call_nonvirtual_long_method(env: *mut JNIEnv, obj: jobject, clazz: jclass, method_id: jmethodID, mut l: ...) -> jlong {
    let mut vararg_provider = VarargProvider::Dots(&mut l);
    match call_non_virtual(env, obj, clazz, method_id, &mut vararg_provider, false) {
        Ok(res) => res,
        Err(WasException {}) => return i64::from(0xdeaddead)
    }.unwrap_long()
}

#[allow(overflowing_literals)]
pub unsafe extern "C" fn call_nonvirtual_long_method_v(env: *mut JNIEnv, obj: jobject, clazz: jclass, method_id: jmethodID, mut args: VaList) -> jlong {
    let mut vararg_provider = VarargProvider::VaList(&mut args);
    match call_non_virtual(env, obj, clazz, method_id, &mut vararg_provider, false) {
        Ok(res) => res,
        Err(WasException {}) => return i64::from(0xdeaddead)
    }.unwrap_long()
}

#[allow(overflowing_literals)]
pub unsafe extern "C" fn call_nonvirtual_long_method_a(env: *mut JNIEnv, obj: jobject, clazz: jclass, method_id: jmethodID, args: *const jvalue) -> jlong {
    let mut vararg_provider = VarargProvider::Array(args);
    match call_non_virtual(env, obj, clazz, method_id, &mut vararg_provider, false) {
        Ok(res) => res,
        Err(WasException {}) => return i64::from(0xdeaddead)
    }.unwrap_long()
}

pub unsafe extern "C" fn call_nonvirtual_float_method(env: *mut JNIEnv, obj: jobject, clazz: jclass, method_id: jmethodID, mut l: ...) -> jfloat {
    let mut vararg_provider = VarargProvider::Dots(&mut l);
    match call_non_virtual(env, obj, clazz, method_id, &mut vararg_provider, false) {
        Ok(res) => res,
        Err(WasException {}) => return f32::NAN
    }.unwrap_float()
}

pub unsafe extern "C" fn call_nonvirtual_float_method_v(env: *mut JNIEnv, obj: jobject, clazz: jclass, method_id: jmethodID, mut args: VaList) -> jfloat {
    let mut vararg_provider = VarargProvider::VaList(&mut args);
    match call_non_virtual(env, obj, clazz, method_id, &mut vararg_provider, false) {
        Ok(res) => res,
        Err(WasException {}) => return f32::NAN
    }.unwrap_float()
}

pub unsafe extern "C" fn call_nonvirtual_float_method_a(env: *mut JNIEnv, obj: jobject, clazz: jclass, method_id: jmethodID, args: *const jvalue) -> jfloat {
    let mut vararg_provider = VarargProvider::Array(args);
    match call_non_virtual(env, obj, clazz, method_id, &mut vararg_provider, false) {
        Ok(res) => res,
        Err(WasException {}) => return f32::NAN
    }.unwrap_float()
}

pub unsafe extern "C" fn call_nonvirtual_double_method(env: *mut JNIEnv, obj: jobject, clazz: jclass, method_id: jmethodID, mut l: ...) -> jdouble {
    let mut vararg_provider = VarargProvider::Dots(&mut l);
    match call_non_virtual(env, obj, clazz, method_id, &mut vararg_provider, false) {
        Ok(res) => res,
        Err(WasException {}) => return f64::NAN
    }.unwrap_double()
}

pub unsafe extern "C" fn call_nonvirtual_double_method_v(env: *mut JNIEnv, obj: jobject, clazz: jclass, method_id: jmethodID, mut args: VaList) -> jdouble {
    let mut vararg_provider = VarargProvider::VaList(&mut args);
    match call_non_virtual(env, obj, clazz, method_id, &mut vararg_provider, false) {
        Ok(res) => res,
        Err(WasException {}) => return f64::NAN
    }.unwrap_double()
}

pub unsafe extern "C" fn call_nonvirtual_double_method_a(env: *mut JNIEnv, obj: jobject, clazz: jclass, method_id: jmethodID, args: *const jvalue) -> jdouble {
    let mut vararg_provider = VarargProvider::Array(args);
    match call_non_virtual(env, obj, clazz, method_id, &mut vararg_provider, false) {
        Ok(res) => res,
        Err(WasException {}) => return f64::NAN
    }.unwrap_double()
}

pub unsafe extern "C" fn call_nonvirtual_void_method(env: *mut JNIEnv, obj: jobject, clazz: jclass, method_id: jmethodID, mut l: ...) {
    let mut vararg_provider = VarargProvider::Dots(&mut l);
    match call_non_virtual(env, obj, clazz, method_id, &mut vararg_provider, true) {
        Ok(res) => res,
        Err(WasException {}) => return
    };
}

pub unsafe extern "C" fn call_nonvirtual_void_method_v(env: *mut JNIEnv, obj: jobject, clazz: jclass, method_id: jmethodID, mut args: VaList) {
    let mut vararg_provider = VarargProvider::VaList(&mut args);
    match call_non_virtual(env, obj, clazz, method_id, &mut vararg_provider, true) {
        Ok(res) => res,
        Err(WasException {}) => return
    };
}

pub unsafe extern "C" fn call_nonvirtual_void_method_a(env: *mut JNIEnv, obj: jobject, clazz: jclass, method_id: jmethodID, args: *const jvalue) {
    let mut vararg_provider = VarargProvider::Array(args);
    match call_non_virtual(env, obj, clazz, method_id, &mut vararg_provider, true) {
        Ok(res) => res,
        Err(WasException {}) => return
    };
}


unsafe fn call_non_virtual<'gc_life>(env: *mut JNIEnv, obj: jobject, _clazz: jclass, method_id: jmethodID, mut vararg_provider: &mut VarargProvider, is_void: bool) -> Result<JavaValue<'gc_life>, WasException> {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let method_id: MethodId = transmute(method_id);
    //todo what to do on invalid methodID, here and more generally
    let (rc, i, method_desc) = match jvm.method_table.read().unwrap().try_lookup(method_id) {
        None => todo!(),
        Some((rc, i)) => {
            (rc.clone(), i, rc.clone().view().method_view_i(i).desc())
        }
    };
    int_state.push_current_operand_stack(JavaValue::Object(todo!()/*from_jclass(jvm,obj)*/));
    push_params_onto_frame(jvm, &mut vararg_provider, int_state, &method_desc);
    invoke_special_impl(jvm, int_state, &method_desc, i, rc)?;
    if !is_void {
        int_state.pop_current_operand_stack(ClassName::object().into());
    }
    Ok(JavaValue::Top)
}