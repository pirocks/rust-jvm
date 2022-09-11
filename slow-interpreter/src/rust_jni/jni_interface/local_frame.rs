use std::collections::HashSet;
use std::ptr::null_mut;

use jvmti_jni_bindings::{jint, JNI_OK, JNIEnv, jobject};

use crate::better_java_stack::native_frame::NativeFrame;
use crate::java_values::GcManagedObject;
use crate::new_java_values::allocated_objects::AllocatedObject;
use crate::rust_jni::jni_interface::{get_interpreter_state, get_state};
use crate::rust_jni::native_util::{from_object_new, to_object, to_object_new};

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
    let popped = pop_current_native_local_refs(interpreter_state); //.pop().expect("Attempted to pop local native frame, but no such local frame exists");
    if result.is_null() {
        null_mut()
    } else {
        //no freeing need occur here
        popped.get(&result).unwrap();
        let mut get_top_frame = get_top_local_ref_frame(todo!()/*interpreter_state*/).clone();
        get_top_frame.insert(result);
        set_local_refs_top_frame(todo!()/*interpreter_state*/, get_top_frame);
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
    push_current_native_local_refs(todo!()/*interpreter_state*/, HashSet::new());
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
        return null_mut();
    }
    let interpreter_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let rust_obj = from_object_new(jvm, ref_).unwrap();
    new_local_ref_internal_new(rust_obj.as_allocated_obj(), interpreter_state)
}

pub unsafe fn new_local_ref_public<'gc, 'l>(rust_obj: Option<GcManagedObject<'gc>>, interpreter_state: &mut NativeFrame<'gc, 'l>) -> jobject {
    if rust_obj.is_none() {
        return null_mut();
    }
    new_local_ref_internal(rust_obj.unwrap(), interpreter_state)
    //todo use match
}

pub unsafe fn new_local_ref_public_new<'gc, 'l>(rust_obj: Option<AllocatedObject<'gc, '_>>, interpreter_state: &mut NativeFrame<'gc, 'l>) -> jobject {
    if rust_obj.is_none() {
        return null_mut();
    }
    new_local_ref_internal_new(rust_obj.unwrap(), interpreter_state)
    //todo use match
}


pub unsafe fn new_local_ref_internal_new<'gc, 'l>(rust_obj: AllocatedObject<'gc, '_>, interpreter_state: &mut NativeFrame<'gc, 'l>) -> jobject {
    let c_obj = to_object_new(rust_obj.into());
    let mut new_local_ref_frame = get_top_local_ref_frame(interpreter_state).clone();
    new_local_ref_frame.insert(c_obj);
    set_local_refs_top_frame(interpreter_state, new_local_ref_frame);
    c_obj
}

unsafe fn new_local_ref_internal<'gc, 'l>(rust_obj: GcManagedObject<'gc>, interpreter_state: &mut NativeFrame<'gc, 'l>) -> jobject {
    let c_obj = to_object(rust_obj.clone().into());
    let mut new_local_ref_frame = get_top_local_ref_frame(interpreter_state).clone();
    new_local_ref_frame.insert(c_obj);
    set_local_refs_top_frame(interpreter_state, new_local_ref_frame);
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
    let mut top_local_ref = get_top_local_ref_frame(interpreter_state).clone();
    top_local_ref.remove(&obj);
    set_local_refs_top_frame(interpreter_state, top_local_ref)
}

fn get_top_local_ref_frame<'gc, 'l>(interpreter_state: &mut NativeFrame<'gc, 'l>) -> HashSet<jobject> {
    current_native_local_refs(interpreter_state).pop().unwrap()
}

fn set_local_refs_top_frame<'gc, 'l>(interpreter_state: &mut NativeFrame<'gc, 'l>, new: HashSet<jobject>) {
    *interpreter_state.frame_info_mut().native_local_refs.last_mut().unwrap() = new
}

fn pop_current_native_local_refs<'gc, 'l>(interpreter_state: &mut NativeFrame<'gc, 'l>) -> HashSet<jobject> {
    todo!()/*match interpreter_state.int_state.as_mut().unwrap().deref_mut() {
        /*InterpreterState::LegacyInterpreter { .. } => todo!(),*/
        InterpreterState::Jit { call_stack, .. } => FrameView::new(call_stack.current_frame_ptr(), call_stack, null_mut()).pop_local_refs(),
    }*/
}

fn push_current_native_local_refs<'gc, 'l>(interpreter_state: &mut NativeFrame<'gc, 'l>, to_push: HashSet<jobject>) {
    todo!()/*match interpreter_state.int_state.as_mut().unwrap().deref_mut() {
        /*InterpreterState::LegacyInterpreter { .. } => todo!(),*/
        InterpreterState::Jit { call_stack, .. } => FrameView::new(call_stack.current_frame_ptr(), call_stack, null_mut()).push_local_refs(to_push),
    }*/
}

fn current_native_local_refs<'gc, 'l>(interpreter_state: &mut NativeFrame<'gc, 'l>) -> Vec<HashSet<jobject>> {
    // assert!(interpreter_state.current_frame().is_opaque() || interpreter_state.current_frame().is_native_method());
    interpreter_state.frame_info_mut().native_local_refs.clone()
}