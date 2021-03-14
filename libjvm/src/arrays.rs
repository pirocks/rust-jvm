use std::ops::Deref;
use std::sync::Arc;

use jvmti_jni_bindings::{jclass, jint, jintArray, JNIEnv, jobject, jvalue};
use slow_interpreter::instructions::new::a_new_array_from_name;
use slow_interpreter::interpreter::WasException;
use slow_interpreter::java_values::{JavaValue, Object};
use slow_interpreter::rust_jni::interface::local_frame::new_local_ref_public;
use slow_interpreter::rust_jni::native_util::{from_jclass, from_object, get_interpreter_state, get_state, to_object};
use slow_interpreter::utils::{throw_illegal_arg_res, throw_npe, throw_npe_res};

#[no_mangle]
unsafe extern "system" fn JVM_AllocateNewArray(env: *mut JNIEnv, obj: jobject, currClass: jclass, length: jint) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetArrayLength(env: *mut JNIEnv, arr: jobject) -> jint {
    match get_array(env, arr) {
        Ok(jv) => {
            jv.unwrap_array().mut_array().len() as i32
        }
        Err(WasException {}) => -1 as i32
    }
}

unsafe fn get_array(env: *mut JNIEnv, arr: jobject) -> Result<JavaValue, WasException> {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    match from_object(arr) {
        None => {
            throw_npe_res(jvm, int_state)?;
            unreachable!()
        }
        Some(possibly_arr) => {
            match possibly_arr.deref() {
                Object::Array(_) => {
                    Ok(JavaValue::Object(from_object(arr)))
                }
                Object::Object(obj) => {
                    throw_illegal_arg_res(jvm, int_state)?;
                    unreachable!()
                }
            }
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_GetArrayElement(env: *mut JNIEnv, arr: jobject, index: jint) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetPrimitiveArrayElement(env: *mut JNIEnv, arr: jobject, index: jint, wCode: jint) -> jvalue {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_SetArrayElement(env: *mut JNIEnv, arr: jobject, index: jint, val: jobject) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_SetPrimitiveArrayElement(env: *mut JNIEnv, arr: jobject, index: jint, v: jvalue, vCode: ::std::os::raw::c_uchar) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_NewArray(env: *mut JNIEnv, eltClass: jclass, length: jint) -> jobject {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let array_type_name = from_jclass(eltClass).as_runtime_class(jvm).ptypeview();
    a_new_array_from_name(jvm, int_state, length, array_type_name);
    new_local_ref_public(int_state.pop_current_operand_stack().unwrap_object(), int_state)
}

#[no_mangle]
unsafe extern "system" fn JVM_NewMultiArray(env: *mut JNIEnv, eltClass: jclass, dim: jintArray) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ArrayCopy(env: *mut JNIEnv, ignored: jclass, src: jobject, src_pos: jint, dst: jobject, dst_pos: jint, length: jint) {
    unimplemented!()
}
