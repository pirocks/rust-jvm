use std::sync::{Arc, RwLockWriteGuard};
use crate::{JVMState, StackEntry};
use jvmti_jni_bindings::{jclass, JNIEnv, jobject, _jobject};

use std::ops::Deref;
use classfile_view::view::ptype_view::{ReferenceTypeView, PTypeView};
use crate::java_values::{Object, JavaValue};
use crate::class_objects::get_or_create_class_object;
use crate::java::lang::class::JClass;
use crate::threading::JavaThread;


pub unsafe extern "C" fn get_object_class(env: *mut JNIEnv, obj: jobject) -> jclass {
    let unwrapped = from_object(obj).unwrap();
    let jvm = get_state(env);
    let mut thread = get_thread(env);
    let mut frames = get_frames(&thread);
    let frame = get_frame(&mut frames);
    let class_object = match unwrapped.deref(){
        Object::Array(a) => {
            get_or_create_class_object(jvm, &PTypeView::Ref(ReferenceTypeView::Array(Box::new(a.elem_type.clone()))), frame, frame.class_pointer.loader(jvm).clone())
        },
        Object::Object(o) => {
            get_or_create_class_object(jvm, &PTypeView::Ref(ReferenceTypeView::Class(o.class_pointer.view().name())), frame, o.class_pointer.loader(jvm).clone())
        },
    };

    to_object(class_object.into()) as jclass
}

pub unsafe extern "C" fn get_frame<'l>(frames: &'l mut  RwLockWriteGuard<Vec<StackEntry>>) -> &'l mut StackEntry {
    frames.last_mut().unwrap()
}

pub fn get_frames(threads: &Arc<JavaThread>) -> RwLockWriteGuard<Vec<StackEntry>> {
    threads.get_frames_mut()
}

pub unsafe fn get_thread<'l>(env: *mut JNIEnv) -> Arc<JavaThread> {
    get_state(env).thread_state.get_current_thread()
}


pub unsafe extern "C" fn get_state(env: *mut JNIEnv) -> &'static JVMState {
    &(*((**env).reserved0 as *const JVMState))
}

pub unsafe extern "C" fn to_object(obj: Option<Arc<Object>>) -> jobject {
    match obj {
        None => std::ptr::null_mut(),
        Some(o) => Box::into_raw(Box::new(o)) as *mut _jobject,
    }
}

pub unsafe extern "C" fn from_object(obj: jobject) -> Option<Arc<Object>> {
    if obj == std::ptr::null_mut() {
        None
    } else {
        (obj as *mut Arc<Object>).as_ref().unwrap().clone().into()
    }
}

pub unsafe fn from_jclass(obj: jclass) -> JClass {
    let possibly_null = from_object(obj);
    if possibly_null.is_none(){
        panic!()
    }
    JavaValue::Object(possibly_null).cast_class()
}
