use std::mem::transmute;

use classfile_view::view::HasAccessFlags;
use jvmti_jni_bindings::{jboolean, jclass, jmethodID, JNI_FALSE, JNI_TRUE, jvmtiEnv, jvmtiError, jvmtiError_JVMTI_ERROR_INVALID_CLASS, jvmtiError_JVMTI_ERROR_NONE};

use crate::java::lang::class::JClass;
use crate::java_values::JavaValue;
use crate::jvmti::get_state;
use crate::method_table::MethodId;
use crate::rust_jni::native_util::{from_jclass, from_object, try_from_jclass};

pub unsafe extern "C" fn is_array_class(env: *mut jvmtiEnv, klass: jclass, is_array_class_ptr: *mut jboolean) -> jvmtiError {
    let jvm = get_state(env);
    let tracing_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "IsArrayClass");
    is_array_class_ptr.write(is_array_impl(klass));
    jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}

pub fn is_array_impl(cls: jclass) -> u8 {
    let object_non_null = unsafe { from_object(transmute(cls)).unwrap().clone() };
    let ptype = JavaValue::Object(object_non_null.into()).cast_class().as_type();
    let is_array = ptype.is_array();
    is_array as jboolean
}

/// Is Interface
///
///     jvmtiError
///     IsInterface(jvmtiEnv* env,
///                 jclass klass,
///                 jboolean* is_interface_ptr)
///
/// Determines whether a class object reference represents an interface.
/// The jboolean result is JNI_TRUE if the "class" is actually an interface, JNI_FALSE otherwise.
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
    let tracing_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "IsInterface");
    null_check!(is_interface_ptr);
    let is_interface = match try_from_jclass(klass) {
        None => return jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_INVALID_CLASS),
        Some(jclass) => jclass,
    }.as_runtime_class().view().is_interface();
    let res = if is_interface { JNI_TRUE } else { JNI_FALSE };
    is_interface_ptr.write(res as jboolean);
    jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}

///Is Method Obsolete
///
///     jvmtiError
///     IsMethodObsolete(jvmtiEnv* env,
///                 jmethodID method,
///                 jboolean* is_obsolete_ptr)
///
/// Determine if a method ID refers to an obsolete method version.
///
/// Phase	Callback Safe	Position	Since
/// may only be called during the start or the live phase 	No 	91	1.0 //todo how to detect start phase
///
/// Capabilities
/// Required Functionality
///
/// Parameters
/// Name 	Type 	Description
/// method	jmethodID	The method ID to query.
/// is_obsolete_ptr	jboolean*	On return, points to the boolean result of this function.
///
/// Agent passes a pointer to a jboolean. On return, the jboolean has been set.
///
/// Errors
/// This function returns either a universal error or one of the following errors
/// Error 	Description
/// JVMTI_ERROR_INVALID_METHODID	method is not a jmethodID.
/// JVMTI_ERROR_NULL_POINTER	is_obsolete_ptr is NULL.
pub unsafe extern "C" fn is_method_obsolete(env: *mut jvmtiEnv, _method: jmethodID, is_obsolete_ptr: *mut jboolean) -> jvmtiError {
    let jvm = get_state(env);
    null_check!(is_obsolete_ptr);
    let tracing_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "IsMethodObsolete");
    is_obsolete_ptr.write(false as u8);//todo don't support retransform classes.
    jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}

///Is Method Native
///
///     jvmtiError
///     IsMethodNative(jvmtiEnv* env,
///                 jmethodID method,
///                 jboolean* is_native_ptr)
///
/// For the method indicated by method, return a value indicating whether the method is native via is_native_ptr
///
/// Phase	Callback Safe	Position	Since
/// may only be called during the start or the live phase 	No 	76	1.0
///
/// Capabilities
/// Required Functionality
///
/// Parameters
/// Name 	Type 	Description
/// method	jmethodID	The method to query.
/// is_native_ptr	jboolean*	On return, points to the boolean result of this function.
///
/// Agent passes a pointer to a jboolean. On return, the jboolean has been set.
///
/// Errors
/// This function returns either a universal error or one of the following errors
/// Error 	Description
/// JVMTI_ERROR_INVALID_METHODID	method is not a jmethodID.
/// JVMTI_ERROR_NULL_POINTER	is_native_ptr is NULL.
///
pub unsafe extern "C" fn is_method_native(
    env: *mut jvmtiEnv,
    method: jmethodID,
    is_native_ptr: *mut jboolean,
) -> jvmtiError {
    let jvm = get_state(env);
    let tracing_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "IsMethodObsolete");
    let method_id: MethodId = transmute(method);//todo find a way to get rid of these transmutes
    let (rc, method_i) = jvm.method_table.read().unwrap().lookup(method_id);
    let mv = rc.view().method_view_i(method_i as usize);
    null_check!(is_native_ptr);
    is_native_ptr.write(mv.is_native() as jboolean);
    jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}