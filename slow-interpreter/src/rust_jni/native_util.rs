use std::sync::Arc;
use runtime_common::java_values::Object;
use crate::get_or_create_class_object;
use jni_bindings::{jclass, JNIEnv, jobject, _jobject};
use rust_jvm_common::classnames::class_name;
use runtime_common::{CallStackEntry, InterpreterState};
use std::rc::Rc;

pub unsafe extern "C" fn get_object_class(env: *mut JNIEnv, obj: jobject) -> jclass {
    let obj: Arc<Object> = get_object(obj);//todo double free hazard
    let state = get_state(env);
    let frame = get_frame(env);
    let class_object = get_or_create_class_object(state, &class_name(&obj.class_pointer.classfile), frame, obj.class_pointer.loader.clone());
    to_object(class_object) as jclass
}

pub unsafe extern "C" fn get_frame(env: *mut JNIEnv) -> Rc<CallStackEntry> {
    let res = ((**env).reserved1 as *mut Rc<CallStackEntry>).as_ref().unwrap();// ptr::as_ref
    res.clone()
}

pub unsafe extern "C" fn get_state<'l>(env: *mut JNIEnv) -> &'l mut InterpreterState {
    &mut (*((**env).reserved0 as *mut InterpreterState))
}

pub unsafe extern "C" fn to_object(obj: Arc<Object>) -> jobject {
    Box::into_raw(Box::new(obj)) as *mut _jobject
}

pub unsafe extern "C" fn get_object(obj: jobject) -> Arc<Object> {
    (obj as *mut Arc<Object>).as_ref().unwrap().clone()
}
