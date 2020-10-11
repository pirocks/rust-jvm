use std::ptr::null_mut;

use jvmti_jni_bindings::{jdouble, jfloat, jint, jlong, jobject, jthread, jvmtiEnv, jvmtiError, jvmtiError_JVMTI_ERROR_ILLEGAL_ARGUMENT, jvmtiError_JVMTI_ERROR_INVALID_SLOT, jvmtiError_JVMTI_ERROR_INVALID_THREAD, jvmtiError_JVMTI_ERROR_NO_MORE_FRAMES, jvmtiError_JVMTI_ERROR_NONE, jvmtiError_JVMTI_ERROR_OPAQUE_FRAME, jvmtiError_JVMTI_ERROR_TYPE_MISMATCH};

use crate::{InterpreterStateGuard, JVMState};
use crate::java::lang::thread::JThread;
use crate::java_values::JavaValue;
use crate::jvmti::{get_interpreter_state, get_state};
use crate::rust_jni::interface::local_frame::new_local_ref_public;
use crate::rust_jni::native_util::from_object;
use crate::stack_entry::StackEntry;

///Get Local Variable - Object
///
///     jvmtiError
///     GetLocalObject(jvmtiEnv* env,
///                 jthread thread,
///                 jint depth,
///                 jint slot,
///                 jobject* value_ptr)
///
/// This function can be used to retrieve the value of a local variable whose type is Object or a subclass of Object.
///
/// Phase	Callback Safe	Position	Since
/// may only be called during the live phase 	No 	21	1.0
///
/// Capabilities
/// Optional Functionality: might not be implemented for all virtual machines.
/// The following capability (as returned by GetCapabilities) must be true to use this function.
/// Capability 	Effect
/// can_access_local_variables	Can set and get local variables
///
/// Parameters
/// Name 	Type 	Description
/// thread	jthread	The thread of the frame containing the variable's value. If thread is NULL, the current thread is used.
/// depth	jint	The depth of the frame containing the variable's value.
/// slot	jint	The variable's slot number.
/// value_ptr	jobject*	On return, points to the variable's value.
///
/// Agent passes a pointer to a jobject. On return, the jobject has been set.
/// The object returned by value_ptr is a JNI local reference and must be managed.
///
/// Errors
/// This function returns either a universal error or one of the following errors
/// Error 	Description
/// JVMTI_ERROR_MUST_POSSESS_CAPABILITY 	The environment does not possess the capability can_access_local_variables. Use AddCapabilities. //todo capabilities
/// JVMTI_ERROR_INVALID_SLOT	Invalid slot.
/// JVMTI_ERROR_TYPE_MISMATCH	The variable type is not Object or a subclass of Object.
/// JVMTI_ERROR_OPAQUE_FRAME	Not a visible frame
/// JVMTI_ERROR_INVALID_THREAD	thread is not a thread object.
/// JVMTI_ERROR_THREAD_NOT_ALIVE	thread is not live (has not been started or is now dead).
/// JVMTI_ERROR_ILLEGAL_ARGUMENT	depth is less than zero.
/// JVMTI_ERROR_NO_MORE_FRAMES	There are no stack frames at the specified depth.
/// JVMTI_ERROR_NULL_POINTER	value_ptr is NULL.
pub unsafe extern "C" fn get_local_object(env: *mut jvmtiEnv, thread: jthread, depth: jint, slot: jint, value_ptr: *mut jobject) -> jvmtiError {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let tracing_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "GetLocalObject");
    assert!(jvm.vm_live());
    null_check!(value_ptr);
    let var = match get_local_t(jvm, int_state, thread, depth, slot) {
        Ok(var) => var,
        Err(err) => return jvm.tracing.trace_jdwp_function_exit(tracing_guard, err),
    };
    // dbg!(&var);
    match var {
        JavaValue::Top => value_ptr.write(null_mut()),//todo is this correct?
        _ => {
            let possibly_object = var.try_unwrap_object();
            match possibly_object {
                None => {
                    return jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_TYPE_MISMATCH);
                }
                Some(obj) => value_ptr.write(new_local_ref_public(obj, int_state)),
            }
        }
    }
    jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}

unsafe fn get_local_primitive_type<T>(env: *mut jvmtiEnv, thread: jthread, depth: jint, slot: jint, value_ptr: *mut T, unwrap_function: fn(JavaValue) -> Option<T>) -> jvmtiError {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let tracing_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "GetLocalObject");
    assert!(jvm.vm_live());
    null_check!(value_ptr);
    let var = match get_local_t(jvm, int_state, thread, depth, slot) {
        Ok(var) => { var }
        Err(err) => return jvm.tracing.trace_jdwp_function_exit(tracing_guard, err),
    };
    match unwrap_function(var) {
        None => {
            return jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_TYPE_MISMATCH);
        }
        Some(unwrapped) => value_ptr.write(unwrapped)
    }
    jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}

///Get Local Variable - Int
///
///     jvmtiError
///     GetLocalInt(jvmtiEnv* env,
///                 jthread thread,
///                 jint depth,
///                 jint slot,
///                 jint* value_ptr)
///
/// This function can be used to retrieve the value of a local variable whose type is int, short, char, byte, or boolean.
///
/// Phase	Callback Safe	Position	Since
/// may only be called during the live phase 	No 	22	1.0
///
/// Capabilities
/// Optional Functionality: might not be implemented for all virtual machines. The following capability (as returned by GetCapabilities) must be true to use this function.
/// Capability 	Effect
/// can_access_local_variables	Can set and get local variables
///
/// Parameters
/// Name 	Type 	Description
/// thread	jthread	The thread of the frame containing the variable's value. If thread is NULL, the current thread is used.
/// depth	jint	The depth of the frame containing the variable's value.
/// slot	jint	The variable's slot number.
/// value_ptr	jint*	On return, points to the variable's value.
///
/// Agent passes a pointer to a jint. On return, the jint has been set.
///
/// Errors
/// This function returns either a universal error or one of the following errors
/// Error 	Description
/// JVMTI_ERROR_MUST_POSSESS_CAPABILITY 	The environment does not possess the capability can_access_local_variables. Use AddCapabilities.
/// JVMTI_ERROR_INVALID_SLOT	Invalid slot.
/// JVMTI_ERROR_TYPE_MISMATCH	The variable type is not int, short, char, byte, or boolean.
/// JVMTI_ERROR_OPAQUE_FRAME	Not a visible frame
/// JVMTI_ERROR_INVALID_THREAD	thread is not a thread object.
/// JVMTI_ERROR_THREAD_NOT_ALIVE	thread is not live (has not been started or is now dead).
/// JVMTI_ERROR_ILLEGAL_ARGUMENT	depth is less than zero.
/// JVMTI_ERROR_NO_MORE_FRAMES	There are no stack frames at the specified depth.
/// JVMTI_ERROR_NULL_POINTER	value_ptr is NULL.
pub unsafe extern "C" fn get_local_int(env: *mut jvmtiEnv, thread: jthread, depth: jint, slot: jint, value_ptr: *mut jint) -> jvmtiError {
    get_local_primitive_type(env, thread, depth, slot, value_ptr, |x| x.try_unwrap_int())
}

pub unsafe extern "C" fn get_local_float(env: *mut jvmtiEnv, thread: jthread, depth: jint, slot: jint, value_ptr: *mut jfloat) -> jvmtiError {
    get_local_primitive_type(env, thread, depth, slot, value_ptr, |x| x.try_unwrap_float())
}

pub unsafe extern "C" fn get_local_double(env: *mut jvmtiEnv, thread: jthread, depth: jint, slot: jint, value_ptr: *mut jdouble) -> jvmtiError {
    get_local_primitive_type(env, thread, depth, slot, value_ptr, |x| x.try_unwrap_double())
}

pub unsafe extern "C" fn get_local_long(env: *mut jvmtiEnv, thread: jthread, depth: jint, slot: jint, value_ptr: *mut jlong) -> jvmtiError {
    get_local_primitive_type(env, thread, depth, slot, value_ptr, |x| x.try_unwrap_long())
}


unsafe fn get_local_t(jvm: &JVMState, int_state: &mut InterpreterStateGuard, thread: jthread, depth: jint, slot: jint) -> Result<JavaValue, jvmtiError> {
    if depth < 0 {
        return Result::Err(jvmtiError_JVMTI_ERROR_ILLEGAL_ARGUMENT);
    }

    let jthread = if !thread.is_null() {
        match JavaValue::Object(from_object(thread)).try_cast_thread() {
            None => return Result::Err(jvmtiError_JVMTI_ERROR_INVALID_THREAD),
            Some(jt) => jt,
        }
    } else {
        JThread::current_thread(jvm, int_state)
    };
    let java_thread = jthread.get_java_thread(jvm);
    let call_stack = &java_thread.interpreter_state.read().unwrap().call_stack;
    let stack_frame: &StackEntry = match call_stack.get(call_stack.len() - 1 - depth as usize) {
        None => return Result::Err(jvmtiError_JVMTI_ERROR_NO_MORE_FRAMES),
        Some(entry) => entry,
    };
    if stack_frame.is_native() {
        return Result::Err(jvmtiError_JVMTI_ERROR_OPAQUE_FRAME);
    }
    dbg!(stack_frame.local_vars());
    let var = stack_frame.local_vars().get(slot as usize).cloned();
    var.map(Result::Ok).unwrap_or(Result::Err(jvmtiError_JVMTI_ERROR_INVALID_SLOT))
}
