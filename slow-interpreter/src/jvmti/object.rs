use std::mem::transmute;

use jvmti_jni_bindings::{jint, jobject, jvmtiEnv, jvmtiError, jvmtiError_JVMTI_ERROR_NONE};

use crate::java_values::JavaValue;
use crate::jvmti::{get_interpreter_state, get_state};
use crate::rust_jni::native_util::from_object;

pub unsafe extern "C" fn get_object_hash_code(env: *mut jvmtiEnv, object: jobject, hash_code_ptr: *mut jint) -> jvmtiError {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let tracing_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "GetObjectHashCode");
    let object = JavaValue::Object(from_object(transmute(object))).cast_object();
    let res = object.hash_code(jvm, int_state);
    hash_code_ptr.write(res);
    jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}
