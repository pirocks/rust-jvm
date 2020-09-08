use std::collections::HashSet;
use std::ptr::null_mut;

use jvmti_jni_bindings::{jint, JNI_OK, JNIEnv, jobject};

use crate::InterpreterStateGuard;
use crate::rust_jni::native_util::get_interpreter_state;

///PopLocalFrame
///
/// jobject PopLocalFrame(JNIEnv *env, jobject result);
///
/// Pops off the current local reference frame, frees all the local references, and returns a local reference in the previous local reference frame for the given result object.
///
/// Pass NULL as result if you do not need to return a reference to the previous frame.
///
pub unsafe extern "C" fn pop_local_frame(env: *mut JNIEnv, result: jobject) -> jobject {
    let interpreter_state = get_interpreter_state(env);
    let popped = current_native_local_refs(interpreter_state).pop().expect("Attempted to pop local native frame, but no such local frame exists");
    if result == null_mut() {
        null_mut()
    } else {
        let to_be_preserved = popped.get(&result).unwrap();//todo in future might need something more complex here
        get_top_local_ref_frame(interpreter_state).insert(to_be_preserved.clone());
        to_be_preserved.clone()
    }
}

///PushLocalFrame
///
/// jint PushLocalFrame(JNIEnv *env, jint capacity);
///
/// Creates a new local reference frame, in which at least a given number of local references can be created. Returns 0 on success, a negative number and a pending OutOfMemoryError on failure.
///
/// Note that local references already created in previous local frames are still valid in the current local frame.
pub unsafe extern "C" fn push_local_frame(env: *mut JNIEnv, _capacity: jint) -> jint {
    let interpreter_state = get_interpreter_state(env);
    current_native_local_refs(interpreter_state).push(HashSet::new());
    JNI_OK as jint
}

/// NewLocalRef
///
/// jobject NewLocalRef(JNIEnv *env, jobject ref);
///
/// Creates a new local reference that refers to the same object as ref. The given ref may be a global or local reference. Returns NULL if ref refers to null.
///
pub unsafe extern "C" fn new_local_ref(env: *mut JNIEnv, ref_: jobject) -> jobject {
    let interpreter_state = get_interpreter_state(env);
    get_top_local_ref_frame(interpreter_state).insert(ref_);
    ref_
}

/// DeleteLocalRef
///
/// void DeleteLocalRef(JNIEnv *env, jobject localRef);
///
/// Deletes the local reference pointed to by localRef.
///
pub unsafe extern "C" fn delete_local_ref(env: *mut JNIEnv, obj: jobject) {
    let interpreter_state = get_interpreter_state(env);
    get_top_local_ref_frame(interpreter_state).remove(&obj);
}

fn get_top_local_ref_frame<'l>(interpreter_state: &'l mut InterpreterStateGuard) -> &'l mut HashSet<jobject> {
    current_native_local_refs(interpreter_state).last_mut().unwrap()
}

fn current_native_local_refs<'l>(interpreter_state: &'l mut InterpreterStateGuard) -> &'l mut Vec<HashSet<jobject>> {
    &mut interpreter_state.current_frame_mut().native_local_refs
}