use jvmti_jni_bindings::{jlong, jobject, jvmtiEnv, jvmtiError, jvmtiError_JVMTI_ERROR_NONE};

use crate::jvmti::get_state;

///Get Tag
///
///     jvmtiError
///     GetTag(jvmtiEnv* env,
///                 jobject object,
///                 jlong* tag_ptr)
///
/// Retrieve the tag associated with an object. The tag is a long value typically used to store a unique identifier or pointer to object information. The tag is set with SetTag.
/// Objects for which no tags have been set return a tag value of zero.
///
/// Phase	Callback Safe	Position	Since
/// may only be called during the start or the live phase 	No 	106	1.0
///
/// Capabilities
/// Optional Functionality: might not be implemented for all virtual machines. The following capability (as returned by GetCapabilities) must be true to use this function.
/// Capability 	Effect
/// can_tag_objects	Can set and get tags, as described in the Heap category.
///
/// Parameters
/// Name 	Type 	Description
/// object	jobject	The object whose tag is to be retrieved.
/// tag_ptr	jlong*	On return, the referenced long is set to the value of the tag.
///
/// Agent passes a pointer to a jlong. On return, the jlong has been set.
///
/// Errors
/// This function returns either a universal error or one of the following errors
/// Error 	Description
/// JVMTI_ERROR_MUST_POSSESS_CAPABILITY 	The environment does not possess the capability can_tag_objects. Use AddCapabilities.
/// JVMTI_ERROR_INVALID_OBJECT	object is not an object.
/// JVMTI_ERROR_NULL_POINTER	tag_ptr is NULL.
pub unsafe extern "C" fn get_tag(env: *mut jvmtiEnv, object: jobject, tag_ptr: *mut jlong) -> jvmtiError {
    let jvm = get_state(env);
    let tracing_guard = jvm.config.tracing.trace_jdwp_function_enter(jvm, "GetTag");
    null_check!(object);
    //todo handle capabilities
    let res = match jvm.jvmti_state().unwrap().tags.read().unwrap().get(&object) {
        None => {
            tag_ptr.write(0);
            jvmtiError_JVMTI_ERROR_NONE
        }
        Some(tag) => {
            tag_ptr.write(*tag);
            jvmtiError_JVMTI_ERROR_NONE
        }
    };
    jvm.config.tracing.trace_jdwp_function_exit(tracing_guard, res)
}


///Set Tag
///
///     jvmtiError
///     SetTag(jvmtiEnv* env,
///                 jobject object,
///                 jlong tag)
///
/// Set the tag associated with an object. The tag is a long value typically used to store a unique identifier or pointer to object information. The tag is visible with GetTag.
///
/// Phase	Callback Safe	Position	Since
/// may only be called during the start or the live phase 	No 	107	1.0
///
/// Capabilities
/// Optional Functionality: might not be implemented for all virtual machines. The following capability (as returned by GetCapabilities) must be true to use this function.
/// Capability 	Effect
/// can_tag_objects	Can set and get tags, as described in the Heap category.
///
/// Parameters
/// Name 	Type 	Description
/// object	jobject	The object whose tag is to be set.
/// tag	jlong	The new value of the tag.
///
/// Errors
/// This function returns either a universal error or one of the following errors
/// Error 	Description
/// JVMTI_ERROR_MUST_POSSESS_CAPABILITY 	The environment does not possess the capability can_tag_objects. Use AddCapabilities.
/// JVMTI_ERROR_INVALID_OBJECT	object is not an object.
pub unsafe extern "C" fn set_tag(env: *mut jvmtiEnv, object: jobject, tag: jlong) -> jvmtiError {
    let jvm = get_state(env);
    //todo handle capabilities
    let tracing_guard = jvm.config.tracing.trace_jdwp_function_enter(jvm, "SetTag");
    jvm.jvmti_state().unwrap().tags.write().unwrap().insert(object, tag);
    jvm.config.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}
