use jvmti_jni_bindings::{jint, jobject, jvmtiEnv, jvmtiError, jvmtiError_JVMTI_ERROR_INVALID_OBJECT, jvmtiError_JVMTI_ERROR_NONE};

use crate::java_values::JavaValue;
use crate::jvmti::{get_interpreter_state, get_state};
use crate::rust_jni::native_util::from_object;

///Get Object Hash Code
///
///     jvmtiError
///     GetObjectHashCode(jvmtiEnv* env,
///                 jobject object,
///                 jint* hash_code_ptr)
///
/// For the object indicated by object, return via hash_code_ptr a hash code.
/// This hash code could be used to maintain a hash table of object references, however, on some implementations this can cause significant performance impacts--in most cases tags will be a more efficient means of associating information with objects.
/// This function guarantees the same hash code value for a particular object throughout its life
///
/// Phase	Callback Safe	Position	Since
/// may only be called during the start or the live phase 	No 	58	1.0
///
/// Capabilities
/// Required Functionality
///
/// Parameters
/// Name 	Type 	Description
/// object	jobject	The object to query.
/// hash_code_ptr	jint*	On return, points to the object's hash code.
///
/// Agent passes a pointer to a jint. On return, the jint has been set.
///
/// Errors
/// This function returns either a universal error or one of the following errors
/// Error 	Description
/// JVMTI_ERROR_INVALID_OBJECT	object is not an object.
/// JVMTI_ERROR_NULL_POINTER	hash_code_ptr is NULL.
pub unsafe extern "C" fn get_object_hash_code(env: *mut jvmtiEnv, object: jobject, hash_code_ptr: *mut jint) -> jvmtiError {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    assert!(jvm.vm_live());
    null_check!(hash_code_ptr);
    let tracing_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "GetObjectHashCode");
    if object.is_null() {
        return jvmtiError_JVMTI_ERROR_INVALID_OBJECT
    }
    let object = JavaValue::Object(from_object(object)).cast_object();
    let res = object.hash_code(jvm, int_state);
    hash_code_ptr.write(res);
    jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}
