use std::ptr::null_mut;

use jvmti_jni_bindings::{_jobject, JNIEnv, jstring};
use slow_interpreter::exceptions::WasException;
use slow_interpreter::rust_jni::jni_utils::{get_interpreter_state, get_state, new_local_ref_public_new};
use slow_interpreter::rust_jni::native_util::from_object_new;
use slow_interpreter::stdlib::java::NewAsObjectOrJavaValue;
use slow_interpreter::string_intern::intern_safe;

#[no_mangle]
unsafe extern "system" fn JVM_InternString(env: *mut JNIEnv, str_unsafe: jstring) -> jstring {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let str_obj = match from_object_new(jvm, str_unsafe) {
        Some(x) => x,
        None => todo!()/*return throw_npe_res(jvm, int_state)*/,
    };
    match Ok(new_local_ref_public_new(intern_safe(jvm, str_obj).object().as_allocated_obj().into(), int_state)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            null_mut()
        }
    }
}