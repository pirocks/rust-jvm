use std::sync::Arc;
use runtime_common::java_values::Object;
use crate::get_or_create_class_object;
use jni_bindings::{jclass, JNIEnv, jobject, _jobject};
use rust_jvm_common::classnames::class_name;
use runtime_common::{StackEntry, InterpreterState};
use std::rc::Rc;

pub unsafe extern "C" fn get_object_class(env: *mut JNIEnv, obj: jobject) -> jclass {
    let obj: Arc<Object> = from_object(obj).unwrap();//todo double free hazard
    let state = get_state(env);
    let frame = get_frame(env);
    let class_object = get_or_create_class_object(state, &class_name(&obj.class_pointer.classfile), frame, obj.class_pointer.loader.clone());
    to_object(class_object.into()) as jclass
}

pub unsafe extern "C" fn get_frame(env: *mut JNIEnv) -> Rc<StackEntry> {
    let res = ((**env).reserved1 as *mut Rc<StackEntry>).as_ref().unwrap();// ptr::as_ref
    res.clone()
}

pub unsafe extern "C" fn get_state<'l>(env: *mut JNIEnv) -> &'l mut InterpreterState {
    &mut (*((**env).reserved0 as *mut InterpreterState))
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