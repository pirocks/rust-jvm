use std::ffi::CString;

use classfile_view::view::HasAccessFlags;
use jvmti_jni_bindings::{jboolean, jclass, jint, jlocation, jmethodID, jvmtiEnv, jvmtiError, jvmtiError_JVMTI_ERROR_NONE};

use crate::class_objects::get_or_create_class_object;
use crate::jvmti::{get_interpreter_state, get_state};
use method_table::from_jmethod_id;
use crate::rust_jni::interface::local_frame::new_local_ref_public;
use crate::utils::pushable_frame_todo;

pub unsafe extern "C" fn get_method_name(env: *mut jvmtiEnv, method: jmethodID, name_ptr: *mut *mut ::std::os::raw::c_char, signature_ptr: *mut *mut ::std::os::raw::c_char, generic_ptr: *mut *mut ::std::os::raw::c_char) -> jvmtiError {
    let jvm = get_state(env);
    let tracing_guard = jvm.config.tracing.trace_jdwp_function_enter(jvm, "GetMethodName");
    let method_id = from_jmethod_id(method);
    let (class, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap(); //todo handle error
    let class_view = class.view();
    let mv = class_view.method_view_i(method_i);
    let name = mv.name().0.to_str(&jvm.string_pool);
    let desc_str = mv.desc_str().to_str(&jvm.string_pool);
    if !generic_ptr.is_null() {
        // unimplemented!()//todo figure out what this is
    }
    if !signature_ptr.is_null() {
        signature_ptr.write(CString::new(desc_str).unwrap().into_raw())
    }
    if !name_ptr.is_null() {
        name_ptr.write(CString::new(name).unwrap().into_raw())
    }
    jvm.config.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}

pub unsafe extern "C" fn get_arguments_size(env: *mut jvmtiEnv, method: jmethodID, size_ptr: *mut jint) -> jvmtiError {
    let jvm = get_state(env);
    let tracing_guard = jvm.config.tracing.trace_jdwp_function_enter(jvm, "GetArgumentsSize");
    let method_id = from_jmethod_id(method);
    let (rc, i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap(); //todo handle error
    let rc_view = rc.view();
    let mv = rc_view.method_view_i(i);
    size_ptr.write(mv.num_args() as i32);
    jvm.config.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}

pub unsafe extern "C" fn get_method_modifiers(env: *mut jvmtiEnv, method: jmethodID, modifiers_ptr: *mut jint) -> jvmtiError {
    let jvm = get_state(env);
    let tracing_guard = jvm.config.tracing.trace_jdwp_function_enter(jvm, "GetMethodModifiers");
    let method_id = from_jmethod_id(method);
    let (class, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
    let modifiers = class.view().method_view_i(method_i).access_flags();
    modifiers_ptr.write(modifiers as jint);
    jvm.config.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}

pub unsafe extern "C" fn get_method_location(env: *mut jvmtiEnv, method: jmethodID, start_location_ptr: *mut jlocation, end_location_ptr: *mut jlocation) -> jvmtiError {
    let jvm = get_state(env);
    let tracing_guard = jvm.config.tracing.trace_jdwp_function_enter(jvm, "GetMethodLocation");
    let method_id = from_jmethod_id(method);
    let (class, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap(); //todo handle error
    match class.view().method_view_i(method_i).real_code_attribute() {
        None => {
            start_location_ptr.write(-1);
            end_location_ptr.write(-1);
        }
        Some(code) => {
            start_location_ptr.write(0);
            end_location_ptr.write(code.code_raw.len() as i64);
        }
    };
    jvm.config.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}

pub unsafe extern "C" fn get_method_declaring_class(env: *mut jvmtiEnv, method: jmethodID, declaring_class_ptr: *mut jclass) -> jvmtiError {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let tracing_guard = jvm.config.tracing.trace_jdwp_function_enter(jvm, "GetMethodDeclaringClass");
    let method_id = from_jmethod_id(method);
    let runtime_class = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap().0; //todo handle error
    let class_object = get_or_create_class_object(jvm, runtime_class.cpdtype(), pushable_frame_todo()/*int_state*/); //todo fix this type verbosity thing
    declaring_class_ptr.write(new_local_ref_public(class_object.unwrap().to_gc_managed().into(), int_state));
    jvm.config.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
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
    let tracing_guard = jvm.config.tracing.trace_jdwp_function_enter(jvm, "IsMethodObsolete");
    is_obsolete_ptr.write(false as u8); //todo don't support retransform classes.
    jvm.config.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
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
pub unsafe extern "C" fn is_method_native(env: *mut jvmtiEnv, method: jmethodID, is_native_ptr: *mut jboolean) -> jvmtiError {
    let jvm = get_state(env);
    let tracing_guard = jvm.config.tracing.trace_jdwp_function_enter(jvm, "IsMethodObsolete");
    let method_id = from_jmethod_id(method); //todo find a way to get rid of these transmutes
    let (rc, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
    let rc_view = rc.view();
    let mv = rc_view.method_view_i(method_i);
    null_check!(is_native_ptr);
    is_native_ptr.write(mv.is_native() as jboolean);
    jvm.config.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}

pub unsafe extern "C" fn is_method_synthetic(env: *mut jvmtiEnv, method: jmethodID, is_synthetic_ptr: *mut jboolean) -> jvmtiError {
    let jvm = get_state(env);
    let tracing_guard = jvm.config.tracing.trace_jdwp_function_enter(jvm, "IsMethodSynthetic");
    let method_id = from_jmethod_id(method);
    let (class, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap(); //todo handle error
    let synthetic = class.view().method_view_i(method_i).is_synthetic();
    is_synthetic_ptr.write(synthetic as u8);
    jvm.config.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}
