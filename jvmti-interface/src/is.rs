use jvmti_jni_bindings::{jboolean, jclass, JNI_FALSE, JNI_TRUE, jvmtiEnv, jvmtiError, jvmtiError_JVMTI_ERROR_INVALID_CLASS, jvmtiError_JVMTI_ERROR_NONE};

use slow_interpreter::jvm_state::JVMState;
use slow_interpreter::rust_jni::jvmti::get_state;
use slow_interpreter::rust_jni::native_util::try_from_jclass;

///Is Array Class
///
///     jvmtiError
///     IsArrayClass(jvmtiEnv* env,
///                 jclass klass,
///                 jboolean* is_array_class_ptr)
///
/// Determines whether a class object reference represents an array. The jboolean result is JNI_TRUE if the class is an array, JNI_FALSE otherwise.
///
/// Phase	Callback Safe	Position	Since
/// may only be called during the start or the live phase 	No 	56	1.0
///
/// Capabilities
/// Required Functionality
///
/// Parameters
/// Name 	Type 	Description
/// klass	jclass	The class to query.
/// is_array_class_ptr	jboolean*	On return, points to the boolean result of this function.
///
/// Agent passes a pointer to a jboolean. On return, the jboolean has been set.
///
/// Errors
/// This function returns either a universal error or one of the following errors
/// Error 	Description
/// JVMTI_ERROR_INVALID_CLASS	klass is not a class object or the class has been unloaded.
/// JVMTI_ERROR_NULL_POINTER	is_array_class_ptr is NULL.
pub unsafe extern "C" fn is_array_class(env: *mut jvmtiEnv, klass: jclass, is_array_class_ptr: *mut jboolean) -> jvmtiError {
    let jvm = get_state(env);
    let tracing_guard = jvm.config.tracing.trace_jdwp_function_enter(jvm, "IsArrayClass");
    let res = match is_array_impl(jvm, klass) {
        Ok(res) => res,
        Err(err) => return jvm.config.tracing.trace_jdwp_function_exit(tracing_guard, err),
    };
    is_array_class_ptr.write(res);
    jvm.config.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}

pub unsafe fn is_array_impl<'gc>(jvm: &'gc JVMState<'gc>, cls: jclass) -> Result<u8, jvmtiError> {
    let jclass = match try_from_jclass(jvm, cls) {
        None => return Result::Err(jvmtiError_JVMTI_ERROR_INVALID_CLASS),
        Some(jclass) => jclass,
    };
    let rtype = jclass.as_type(jvm);
    let is_array = rtype.is_array();
    Result::Ok((if is_array { JNI_TRUE } else { JNI_FALSE }) as jboolean)
}

/// Is Interface
///
///     jvmtiError
///     IsInterface(jvmtiEnv* env,
///                 jclass klass,
///                 jboolean* is_interface_ptr)
///
/// Determines whether a class object reference represents an jni_interface.
/// The jboolean result is JNI_TRUE if the "class" is actually an jni_interface, JNI_FALSE otherwise.
///
/// Phase	Callback Safe	Position	Since
/// may only be called during the start or the live phase 	No 	55	1.0
///
/// Capabilities
/// Required Functionality
///
/// Parameters
/// Name 	Type 	Description
/// klass	jclass	The class to query.
/// is_interface_ptr	jboolean*	On return, points to the boolean result of this function.
///
/// Agent passes a pointer to a jboolean. On return, the jboolean has been set.
///
/// Errors
/// This function returns either a universal error or one of the following errors
/// Error 	Description
/// JVMTI_ERROR_INVALID_CLASS	klass is not a class object or the class has been unloaded.
/// JVMTI_ERROR_NULL_POINTER	is_interface_ptr is NULL.
pub unsafe extern "C" fn is_interface(env: *mut jvmtiEnv, klass: jclass, is_interface_ptr: *mut jboolean) -> jvmtiError {
    let jvm = get_state(env);
    let tracing_guard = jvm.config.tracing.trace_jdwp_function_enter(jvm, "IsInterface");
    null_check!(is_interface_ptr);
    let is_interface = match try_from_jclass(jvm, klass) {
        None => return jvm.config.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_INVALID_CLASS),
        Some(jclass) => jclass,
    }
        .as_runtime_class(jvm)
        .view()
        .is_interface();
    let res = if is_interface { JNI_TRUE } else { JNI_FALSE };
    is_interface_ptr.write(res as jboolean);
    jvm.config.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}