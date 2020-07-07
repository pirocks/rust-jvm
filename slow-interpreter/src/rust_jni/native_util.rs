use std::ops::Deref;
use std::sync::Arc;

use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use jvmti_jni_bindings::{_jobject, jclass, JNIEnv, jobject};

use crate::{InterpreterStateGuard, JVMState};
use crate::class_objects::get_or_create_class_object;
use crate::java::lang::class::JClass;
use crate::java_values::{JavaValue, Object};

pub unsafe extern "C" fn get_object_class(env: *mut JNIEnv, obj: jobject) -> jclass {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let unwrapped = from_object(obj).unwrap();
    let class_object = match unwrapped.deref() {
        Object::Array(a) => {
            get_or_create_class_object(jvm, &PTypeView::Ref(ReferenceTypeView::Array(Box::new(a.elem_type.clone()))), int_state, int_state.current_loader(jvm))
        }
        Object::Object(o) => {
            get_or_create_class_object(jvm, &PTypeView::Ref(ReferenceTypeView::Class(o.class_pointer.view().name())), int_state, int_state.current_loader(jvm))
        }
    };

    to_object(class_object.into()) as jclass
}


pub unsafe fn get_state(env: *mut JNIEnv) -> &'static JVMState {
    &(*((**env).reserved0 as *const JVMState))
}

pub unsafe fn get_interpreter_state<'l>(env: *mut JNIEnv) -> &'l mut InterpreterStateGuard<'l> {
    get_state(env).get_int_state_guard()
}

pub unsafe fn to_object(obj: Option<Arc<Object>>) -> jobject {
    match obj {
        None => std::ptr::null_mut(),
        Some(o) => Box::into_raw(Box::new(o)) as *mut _jobject,
    }
}

pub unsafe fn from_object(obj: jobject) -> Option<Arc<Object>> {
    if obj == std::ptr::null_mut() {
        None
    } else {
        (obj as *mut Arc<Object>).as_ref().unwrap().clone().into()
    }
}

pub unsafe fn from_jclass(obj: jclass) -> JClass {
    let possibly_null = from_object(obj);
    if possibly_null.is_none() {
        panic!()
    }
    JavaValue::Object(possibly_null).cast_class()
}
