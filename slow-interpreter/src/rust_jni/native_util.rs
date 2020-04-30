use std::sync::Arc;
use crate::{JVMState, StackEntry};
use jvmti_jni_bindings::{jclass, JNIEnv, jobject, _jobject};

use std::ops::Deref;
use classfile_view::view::ptype_view::{ReferenceTypeView, PTypeView};
use crate::java_values::Object;
use crate::class_objects::get_or_create_class_object;
use std::rc::Rc;


pub unsafe extern "C" fn get_object_class(env: *mut JNIEnv, obj: jobject) -> jclass {
    let unwrapped = from_object(obj).unwrap();
    let jvm = get_state(env);
    let frame_temp = get_frame(env);
    let frame = frame_temp.deref();
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

pub unsafe extern "C" fn get_frame(env: *mut JNIEnv) -> Rc<StackEntry> {
    get_state(env).get_current_frame()
}

pub unsafe extern "C" fn get_state<'l>(env: *mut JNIEnv) -> &'l JVMState {
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
