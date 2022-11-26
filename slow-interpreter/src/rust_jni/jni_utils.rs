use std::collections::HashSet;
use std::ffi::c_void;
use std::ptr::null_mut;

use jvmti_jni_bindings::{JNIEnv, jobject};
use jvmti_jni_bindings::jni_interface::{JNIEnvNamedReservedPointers, JNINativeInterfaceNamedReservedPointers};

use crate::{JVMState, WasException};
use crate::better_java_stack::native_frame::NativeFrame;
use crate::java_values::GcManagedObject;
use crate::new_java_values::allocated_objects::AllocatedObject;
use crate::rust_jni::native_util::{to_object, to_object_new};

pub fn get_top_local_ref_frame<'gc, 'l>(interpreter_state: &mut NativeFrame<'gc, 'l>) -> HashSet<jobject> {
    current_native_local_refs(interpreter_state).pop().unwrap()
}

pub fn set_local_refs_top_frame<'gc, 'l>(interpreter_state: &mut NativeFrame<'gc, 'l>, new: HashSet<jobject>) {
    *interpreter_state.frame_info_mut().native_local_refs.last_mut().unwrap() = new
}

pub fn pop_current_native_local_refs<'gc, 'l>(interpreter_state: &mut NativeFrame<'gc, 'l>) -> HashSet<jobject> {
    todo!()/*match interpreter_state.int_state.as_mut().unwrap().deref_mut() {
        /*InterpreterState::LegacyInterpreter { .. } => todo!(),*/
        InterpreterState::Jit { call_stack, .. } => FrameView::new(call_stack.current_frame_ptr(), call_stack, null_mut()).pop_local_refs(),
    }*/
}

pub fn push_current_native_local_refs<'gc, 'l>(interpreter_state: &mut NativeFrame<'gc, 'l>, to_push: HashSet<jobject>) {
    todo!()/*match interpreter_state.int_state.as_mut().unwrap().deref_mut() {
        /*InterpreterState::LegacyInterpreter { .. } => todo!(),*/
        InterpreterState::Jit { call_stack, .. } => FrameView::new(call_stack.current_frame_ptr(), call_stack, null_mut()).push_local_refs(to_push),
    }*/
}

pub fn current_native_local_refs<'gc, 'l>(interpreter_state: &mut NativeFrame<'gc, 'l>) -> Vec<HashSet<jobject>> {
    // assert!(interpreter_state.current_frame().is_opaque() || interpreter_state.current_frame().is_native_method());
    interpreter_state.frame_info_mut().native_local_refs.clone()
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


pub fn with_jni_interface<'gc, 'l, T>(jvm: &'gc JVMState<'gc>, int_state: &mut NativeFrame<'gc, 'l>, was_exception: &mut Option<WasException<'gc>>, with_interface: impl FnOnce(*mut *const JNINativeInterfaceNamedReservedPointers) -> T) -> T {
    let jvm_ptr = jvm as *const JVMState<'gc> as *const c_void as *mut c_void; //todo this is mut/const thing is annoying
    let int_state_ptr = int_state as *mut NativeFrame<'gc, 'l> as *mut c_void;
    let exception_pointer = was_exception as *mut Option<WasException<'gc>> as *mut c_void;
    let interface = int_state.stack_jni_interface().jni_inner_mut();
    let jvm_state_save = interface.jvm_state;
    let native_frame_save = interface.native_frame;
    let was_exception_save = interface.was_exception;
    interface.jvm_state = jvm_ptr;
    interface.native_frame = int_state_ptr;
    interface.was_exception = exception_pointer;
    let mut as_ptr = interface as *const JNINativeInterfaceNamedReservedPointers;
    let as_ptr2 = (&mut as_ptr) as *mut *const JNINativeInterfaceNamedReservedPointers;
    let res = with_interface(as_ptr2);
    interface.jvm_state = jvm_state_save;
    interface.native_frame = native_frame_save;
    interface.was_exception = was_exception_save;
    res
}

pub unsafe fn get_state<'gc>(env: *mut JNIEnv) -> &'gc JVMState<'gc> {
    let env = env as *mut JNIEnvNamedReservedPointers;
    &*((**env).jvm_state as *const JVMState)
}

pub unsafe fn get_interpreter_state<'gc, 'k, 'any>(env: *mut JNIEnv) -> &'any mut NativeFrame<'gc, 'k> {
    let env = env as *mut JNIEnvNamedReservedPointers;
    &mut *((**env).native_frame as *mut NativeFrame<'gc, 'k>)
}

pub unsafe fn get_throw<'any, 'gc>(env: *mut JNIEnv) -> &'any mut Option<WasException<'gc>> {
    let env = env as *mut JNIEnvNamedReservedPointers;
    &mut *((**env).native_frame as *mut Option<WasException<'gc>>)
}
