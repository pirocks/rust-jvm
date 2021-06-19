use std::ops::Deref;
use std::sync::Arc;

use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use jvmti_jni_bindings::{_jobject, jclass, JNIEnv, jobject};

use crate::{InterpreterStateGuard, JVMState};
use crate::class_objects::get_or_create_class_object;
use crate::java::lang::class::JClass;
use crate::java_values::{JavaValue, Object};
use crate::rust_jni::interface::local_frame::new_local_ref_public;

pub unsafe extern "C" fn get_object_class(env: *mut JNIEnv, obj: jobject) -> jclass {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let unwrapped = from_object(obj).unwrap();//todo handle npe
    let class_object = match unwrapped.deref() {
        Object::Array(a) => {
            get_or_create_class_object(jvm, PTypeView::Ref(ReferenceTypeView::Array(Box::new(a.elem_type.clone()))), int_state)
        }
        Object::Object(o) => {
            get_or_create_class_object(jvm, o.objinfo.class_pointer.view().type_(), int_state)
        }
    }.unwrap();//todo pass the error up

    new_local_ref_public(class_object.into(), int_state) as jclass
}


pub unsafe fn get_state<'gc_life>(env: *mut JNIEnv) -> &'gc_life JVMState<'gc_life> {
    &(*((**env).reserved0 as *const JVMState))
}

pub unsafe fn get_interpreter_state<'k, 'l>(env: *mut JNIEnv) -> &'l mut InterpreterStateGuard<'l, 'k> {
    let jvm = get_state(env);
    jvm.get_int_state()
}


pub unsafe fn to_object<'gc_life>(obj: Option<Arc<Object<'gc_life>>>) -> jobject {
    match obj {
        None => std::ptr::null_mut(),
        Some(o) => Box::into_raw(Box::new(o)) as *mut _jobject,
    }
}

pub unsafe fn from_object<'gc_life>(obj: jobject) -> Option<Arc<Object<'gc_life>>> {
    if obj.is_null() {
        None
    } else {
        (obj as *mut Arc<Object<'gc_life>>).as_ref().unwrap().clone().into()
    }
}

pub unsafe fn from_jclass<'gc_life>(obj: jclass) -> JClass<'gc_life> {
    try_from_jclass(obj).unwrap()//todo handle npe
}

pub unsafe fn try_from_jclass<'gc_life>(obj: jclass) -> Option<JClass<'gc_life>> {
    let possibly_null = from_object(obj);
    possibly_null.as_ref()?;
    JavaValue::Object(todo!()/*possibly_null*/).cast_class().into()
}
