use std::ptr::null_mut;
use std::sync::Arc;

use bimap::BiMap;
use by_address::ByAddress;

use jvmti_jni_bindings::{jint, JNI_OK, JNIEnv, jobject};

use crate::InterpreterStateGuard;
use crate::java_values::Object;
use crate::rust_jni::native_util::{from_object, get_interpreter_state, to_object};

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
    if result.is_null() {
        null_mut()
    } else {
        //no freeing need occur here
        let to_be_preserved = popped.get_by_right(&result).unwrap();
        get_top_local_ref_frame(interpreter_state).insert(to_be_preserved.clone(), result);
        result
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
    current_native_local_refs(interpreter_state).push(BiMap::new());
    JNI_OK as jint
}

/// NewLocalRef
///
/// jobject NewLocalRef(JNIEnv *env, jobject ref);
///
/// Creates a new local reference that refers to the same object as ref. The given ref may be a global or local reference. Returns NULL if ref refers to null.
///
pub unsafe extern "C" fn new_local_ref(env: *mut JNIEnv, ref_: jobject) -> jobject {
    if ref_.is_null() {
        return null_mut()
    }
    let interpreter_state = get_interpreter_state(env);
    let rust_obj = from_object(ref_).unwrap();
    new_local_ref_internal(rust_obj, interpreter_state)
}

pub unsafe fn new_local_ref_public(rust_obj: Option<Arc<Object>>, interpreter_state: &mut InterpreterStateGuard) -> jobject {
    if rust_obj.is_none() {
        return null_mut()
    }
    new_local_ref_internal(rust_obj.unwrap(), interpreter_state)
}

unsafe fn new_local_ref_internal(rust_obj: Arc<Object>, interpreter_state: &mut InterpreterStateGuard) -> jobject {
    let c_obj = to_object(rust_obj.clone().into());
    get_top_local_ref_frame(interpreter_state).insert(ByAddress(rust_obj), c_obj);//todo replace from object with a non-leaking alternative
    c_obj
}

/// DeleteLocalRef
///
/// void DeleteLocalRef(JNIEnv *env, jobject localRef);
///
/// Deletes the local reference pointed to by localRef.
///
pub unsafe extern "C" fn delete_local_ref(env: *mut JNIEnv, obj: jobject) {
    if obj.is_null() {
        return;
    }
    let interpreter_state = get_interpreter_state(env);
    get_top_local_ref_frame(interpreter_state).remove_by_right(&obj);
}

fn get_top_local_ref_frame<'l>(interpreter_state: &'l mut InterpreterStateGuard) -> &'l mut BiMap<ByAddress<Arc<Object>>, jobject> {
    current_native_local_refs(interpreter_state).last_mut().unwrap()
}

fn current_native_local_refs<'l>(interpreter_state: &'l mut InterpreterStateGuard) -> &'l mut Vec<BiMap<ByAddress<Arc<Object>>, jobject>> {
    &mut interpreter_state.current_frame_mut().native_local_refs
}