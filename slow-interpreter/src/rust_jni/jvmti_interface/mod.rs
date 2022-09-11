
use jvmti_jni_bindings::*;
use rust_jvm_common::FieldId;

use crate::{InterpreterStateGuard, JavaValue, NewAsObjectOrJavaValue};
use crate::class_objects::get_or_create_class_object;
use crate::get_thread_or_error;
use crate::rust_jni::jvmti_interface::locals::set_local;
use crate::rust_jni::jni_interface::jvmti::{get_interpreter_state, get_state};
use crate::rust_jni::jni_interface::local_frame::new_local_ref_public;
use crate::rust_jni::native_util::from_jclass;
use crate::rust_jni::native_util::from_object;
use crate::utils::pushable_frame_todo;

pub mod event_callbacks;

//todo handle early return message here?
#[macro_export]
macro_rules! null_check {
    ($ptr: expr) => {
        if $ptr.is_null() {
            return crate::rust_jni::jvmti_interface::jvmtiError_JVMTI_ERROR_NULL_POINTER;
        }
    };
}


///Get Max Locals
//
//     jvmtiError
//     GetMaxLocals(jvmtiEnv* env,
//                 jmethodID method,
//                 jint* max_ptr)
//
// For the method indicated by method, return the number of local variable slots used by the method, including the local variables used to pass parameters to the method on its invocation.
//
// See max_locals in The Java™ Virtual Machine Specification, Chapter 4.7.3.
//
// Phase	Callback Safe	Position	Since
// may only be called during the start or the live phase 	No 	68	1.0
//
// Capabilities
// Required Functionality
//
// Parameters
// Name 	Type 	Description
// method	jmethodID	The method to query.
// max_ptr	jint*	On return, points to the maximum number of local slots
//
// Agent passes a pointer to a jint. On return, the jint has been set.
//
// Errors
// This function returns either a universal error or one of the following errors
// Error 	Description
// JVMTI_ERROR_INVALID_METHODID	method is not a jmethodID.
// JVMTI_ERROR_NATIVE_METHOD	method is a native method.
// JVMTI_ERROR_NULL_POINTER	max_ptr is NULL.
pub unsafe extern "C" fn get_max_locals(env: *mut jvmtiEnv, method: jmethodID, max_ptr: *mut jint) -> jvmtiError {
    null_check!(max_ptr);
    let jvm = get_state(env);
    let (runtime_class, index) = match jvm.method_table.read().unwrap().try_lookup(method as usize) {
        None => return jvmtiError_JVMTI_ERROR_INVALID_METHODID,
        Some(method_id) => method_id,
    };
    let max_locals = match runtime_class.view().method_view_i(index).code_attribute() {
        None => return jvmtiError_JVMTI_ERROR_NATIVE_METHOD,
        Some(res) => res,
    }
        .max_locals;
    max_ptr.write(max_locals as i32);
    jvmtiError_JVMTI_ERROR_NONE
}

//Get Field Declaring Class
//
//     jvmtiError
//     GetFieldDeclaringClass(jvmtiEnv* env,
//                 jclass klass,
//                 jfieldID field,
//                 jclass* declaring_class_ptr)
//
// For the field indicated by klass and field return the class that defined it via declaring_class_ptr. The declaring class will either be klass, a superclass, or an implemented jni_interface.
//
// Phase	Callback Safe	Position	Since
// may only be called during the start or the live phase 	No 	61	1.0
//
// Capabilities
// Required Functionality
//
// Parameters
// Name 	Type 	Description
// klass	jclass	The class to query.
// field	jfieldID	The field to query.
// declaring_class_ptr	jclass*	On return, points to the declaring class
//
// Agent passes a pointer to a jclass. On return, the jclass has been set. The object returned by declaring_class_ptr is a JNI local reference and must be managed.
//
// Errors
// This function returns either a universal error or one of the following errors
// Error 	Description
// JVMTI_ERROR_INVALID_CLASS	klass is not a class object or the class has been unloaded.
// JVMTI_ERROR_INVALID_FIELDID	field is not a jfieldID.
// JVMTI_ERROR_NULL_POINTER	declaring_class_ptr is NULL.

pub unsafe extern "C" fn get_field_declaring_class(env: *mut jvmtiEnv, _klass: jclass, field: jfieldID, declaring_class_ptr: *mut jclass) -> jvmtiError {
    let jvm = get_state(env);
    null_check!(declaring_class_ptr);
    let field_id: FieldId = field as usize;
    let (runtime_class, index) = jvm.field_table.read().unwrap().lookup(field_id);
    let type_ = runtime_class.view().field(index as usize).field_type();
    let int_state = get_interpreter_state(env);
    let res_object = new_local_ref_public(
        match get_or_create_class_object(jvm, type_, pushable_frame_todo()/*int_state*/) {
            Ok(res) => res.to_gc_managed(),
            Err(_) => return jvmtiError_JVMTI_ERROR_INTERNAL,
        }
            .into(),
        todo!()/*int_state*/
    );
    declaring_class_ptr.write(res_object);
    return jvmtiError_JVMTI_ERROR_NONE;
}

///Get Class Modifiers
//
//     jvmtiError
//     GetClassModifiers(jvmtiEnv* env,
//                 jclass klass,
//                 jint* modifiers_ptr)
//
// For the class indicated by klass, return the access flags via modifiers_ptr. Access flags are defined in The Java™ Virtual Machine Specification, Chapter 4.
//
// If the class is an array class, then its public, private, and protected modifiers are the same as those of its component type. For arrays of primitives, this component type is represented by one of the primitive classes (for example, java.lang.Integer.TYPE).
//
// If the class is a primitive class, its public modifier is always true, and its protected and private modifiers are always false.
//
// If the class is an array class or a primitive class then its final modifier is always true and its jni_interface modifier is always false. The values of its other modifiers are not determined by this specification.
//
// Phase	Callback Safe	Position	Since
// may only be called during the start or the live phase 	No 	51	1.0
//
// Capabilities
// Required Functionality
//
// Parameters
// Name 	Type 	Description
// klass	jclass	The class to query.
// modifiers_ptr	jint*	On return, points to the current access flags of this class.
//
// Agent passes a pointer to a jint. On return, the jint has been set.
//
// Errors
// This function returns either a universal error or one of the following errors
// Error 	Description
// JVMTI_ERROR_INVALID_CLASS	klass is not a class object or the class has been unloaded.
// JVMTI_ERROR_NULL_POINTER	modifiers_ptr is NULL.
pub unsafe extern "C" fn get_class_modifiers(env: *mut jvmtiEnv, klass: jclass, modifiers_ptr: *mut jint) -> jvmtiError {
    let jvm = get_state(env);
    null_check!(modifiers_ptr);
    //handle klass invalid
    let runtime_class = from_jclass(jvm, klass).as_runtime_class(jvm);
    modifiers_ptr.write(runtime_class.view().access_flags() as u32 as i32);
    jvmtiError_JVMTI_ERROR_NONE
}

pub unsafe extern "C" fn set_local_object(env: *mut jvmtiEnv, thread: jthread, depth: jint, slot: jint, value: jobject) -> jvmtiError {
    set_local(env, thread, depth, slot, JavaValue::Object(todo!() /*from_jclass(jvm,value)*/))
}

pub unsafe extern "C" fn set_local_int(env: *mut jvmtiEnv, thread: jthread, depth: jint, slot: jint, value: jint) -> jvmtiError {
    set_local(env, thread, depth, slot, JavaValue::Int(value))
}

pub unsafe extern "C" fn set_local_long(env: *mut jvmtiEnv, thread: jthread, depth: jint, slot: jint, value: jlong) -> jvmtiError {
    set_local(env, thread, depth, slot, JavaValue::Long(value))
}

pub unsafe extern "C" fn set_local_double(env: *mut jvmtiEnv, thread: jthread, depth: jint, slot: jint, value: jdouble) -> jvmtiError {
    set_local(env, thread, depth, slot, JavaValue::Double(value))
}

pub unsafe extern "C" fn set_local_float(env: *mut jvmtiEnv, thread: jthread, depth: jint, slot: jint, value: jfloat) -> jvmtiError {
    set_local(env, thread, depth, slot, JavaValue::Float(value))
}

///Notify Frame Pop
//
//     jvmtiError
//     NotifyFramePop(jvmtiEnv* env,
//                 jthread thread,
//                 jint depth)
//
// When the frame that is currently at depth is popped from the stack, generate a FramePop event. See the FramePop event for details. Only frames corresponding to non-native Java programming language methods can receive notification.
//
// The specified thread must either be the current thread or the thread must be suspended.
//
// Phase	Callback Safe	Position	Since
// may only be called during the live phase 	No 	20	1.0
//
// Capabilities
// Optional Functionality: might not be implemented for all virtual machines. The following capability (as returned by GetCapabilities) must be true to use this function.
// Capability 	Effect
// can_generate_frame_pop_events	Can set and thus get FramePop events
//
// Parameters
// Name 	Type 	Description
// thread	jthread	The thread of the frame for which the frame pop event will be generated. If thread is NULL, the current thread is used.
// depth	jint	The depth of the frame for which the frame pop event will be generated.
//
// Errors
// This function returns either a universal error or one of the following errors
// Error 	Description
// JVMTI_ERROR_MUST_POSSESS_CAPABILITY 	The environment does not possess the capability can_generate_frame_pop_events. Use AddCapabilities.
// JVMTI_ERROR_OPAQUE_FRAME	The frame at depth is executing a native method.
// JVMTI_ERROR_THREAD_NOT_SUSPENDED	Thread was not suspended and was not the current thread.
// JVMTI_ERROR_INVALID_THREAD	thread is not a thread object.
// JVMTI_ERROR_THREAD_NOT_ALIVE	thread is not live (has not been started or is now dead).
// JVMTI_ERROR_ILLEGAL_ARGUMENT	depth is less than zero.
// JVMTI_ERROR_NO_MORE_FRAMES	There are no stack frames at the specified depth.

pub unsafe extern "C" fn notify_frame_pop(env: *mut jvmtiEnv, thread: jthread, depth: jint) -> jvmtiError {
    let jvm = get_state(env);
    //todo check capability
    let java_thread = get_thread_or_error!(jvm, thread).get_java_thread(jvm);
    let action = |int_state: &mut InterpreterStateGuard| {
        //todo check thread opaque
        /*match int_state.add_should_frame_pop_notify(depth as usize) {
            Ok(_) => jvmtiError_JVMTI_ERROR_NONE,
            Err(err) => match err {
                AddFrameNotifyError::Opaque => jvmtiError_JVMTI_ERROR_OPAQUE_FRAME,
                AddFrameNotifyError::NothingAtDepth => jvmtiError_JVMTI_ERROR_NO_MORE_FRAMES,
            },
        }*/
        jvmtiError_JVMTI_ERROR_NONE
    };

    if java_thread.is_this_thread() {
        action(get_interpreter_state(env))
    } else {
        if todo!() {
            return jvmtiError_JVMTI_ERROR_THREAD_SUSPENDED;
        }
        //todo check thread suspended
        let mut int_state_not_ref = InterpreterStateGuard::RemoteInterpreterState {
            int_state: todo!(),
            thread: java_thread,
            registered: false,
            jvm
        };
        action(&mut int_state_not_ref)
    }
}

///Get Current Thread
//
//     jvmtiError
//     GetCurrentThread(jvmtiEnv* env,
//                 jthread* thread_ptr)
//
// Get the current thread. The current thread is the Java programming language thread which has called the function.
//
// Note that most JVM TI functions that take a thread as an argument will accept NULL to mean the current thread.
//
// Phase	Callback Safe	Position	Since
// may only be called during the start or the live phase 	No 	18	1.1
//
// Capabilities
// Required Functionality
//
// Parameters
// Name 	Type 	Description
// thread_ptr	jthread*	On return, points to the current thread.
//
// Agent passes a pointer to a jthread. On return, the jthread has been set. The object returned by thread_ptr is a JNI local reference and must be managed.
//
// Errors
// This function returns either a universal error or one of the following errors
// Error 	Description
// JVMTI_ERROR_NULL_POINTER	thread_ptr is NULL.
pub unsafe extern "C" fn get_current_thread(env: *mut jvmtiEnv, thread_ptr: *mut jthread) -> jvmtiError {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    null_check!(thread_ptr);
    let current_thread = jvm.thread_state.get_current_thread();
    thread_ptr.write(new_local_ref_public(current_thread.thread_object().object().to_gc_managed().into(), todo!()/*int_state*/));
    jvmtiError_JVMTI_ERROR_NONE
}

///Universal Errors
// The following errors may be returned by any function
//
// JVMTI_ERROR_NONE (0)
//     No error has occurred. This is the error code that is returned on successful completion of the function.
//
// JVMTI_ERROR_NULL_POINTER (100)
//     Pointer is unexpectedly NULL.
//
// JVMTI_ERROR_OUT_OF_MEMORY (110)
//     The function attempted to allocate memory and no more memory was available for allocation.
//
// JVMTI_ERROR_ACCESS_DENIED (111)
//     The desired functionality has not been enabled in this virtual machine.
//
// JVMTI_ERROR_UNATTACHED_THREAD (115)
//     The thread being used to call this function is not attached to the virtual machine. Calls must be made from attached threads. See AttachCurrentThread in the JNI invocation API.
//
// JVMTI_ERROR_INVALID_ENVIRONMENT (116)
//     The JVM TI environment provided is no longer connected or is not an environment.
//
// JVMTI_ERROR_WRONG_PHASE (112)
//     The desired functionality is not available in the current phase. Always returned if the virtual machine has completed running.
//
// JVMTI_ERROR_INTERNAL (113)
//     An unexpected internal error has occurred.
pub fn universal_error() -> jvmtiError {
    jvmtiError_JVMTI_ERROR_INTERNAL
    //todo make this better
}

pub mod breakpoint;
pub mod is;
pub mod methods;
pub mod object;
#[macro_use]
pub mod threads;
#[macro_use]
pub mod frame;
#[macro_use]
pub mod thread_local_storage;
pub mod agent;
pub mod allocate;
pub mod capabilities;
pub mod classes;
pub mod events;
pub mod field;
pub mod locals;
pub mod monitor;
pub mod properties;
pub mod tags;
pub mod version;