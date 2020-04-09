use std::sync::Arc;
use crate::{JVMState, StackEntry};
use jni_bindings::{jclass, JNIEnv, jobject, _jobject};
use std::rc::Rc;
use std::ops::Deref;
use classfile_view::view::ptype_view::{ReferenceTypeView, PTypeView};
use crate::java_values::Object;
use crate::class_objects::get_or_create_class_object;


pub unsafe extern "C" fn get_object_class(env: *mut JNIEnv, obj: jobject) -> jclass {
    let unwrapped = from_object(obj).unwrap();
    let state = get_state(env);
    let frame = get_frame(env);
//    let obj= unwrapped.unwrap_normal_object();//todo double free hazard
    let class_object = match unwrapped.deref(){
        Object::Array(a) => {
            get_or_create_class_object(state, &PTypeView::Ref(ReferenceTypeView::Array(Box::new(a.elem_type.clone()))), frame.clone(), frame.class_pointer.loader.clone())
        },
        Object::Object(o) => {
            get_or_create_class_object(state, &PTypeView::Ref(ReferenceTypeView::Class(o.class_pointer.class_view.name())), frame, o.class_pointer.loader.clone())
        },
    };

    to_object(class_object.into()) as jclass
}

pub unsafe extern "C" fn get_frame(env: *mut JNIEnv) -> Rc<StackEntry> {
    get_state(env).get_current_thread().call_stack.clone()
}

pub unsafe extern "C" fn get_state<'l>(env: *mut JNIEnv) -> &'l JVMState/*<'l>*/ {
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
