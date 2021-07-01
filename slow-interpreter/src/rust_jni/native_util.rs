use std::mem;
use std::ops::Deref;
use std::ptr::NonNull;

use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use jvmti_jni_bindings::{_jobject, jclass, JNIEnv, jobject};

use crate::{InterpreterStateGuard, JVMState};
use crate::class_objects::get_or_create_class_object;
use crate::java::lang::class::JClass;
use crate::java_values::{GcManagedObject, JavaValue, Object};
use crate::rust_jni::interface::local_frame::new_local_ref_public;

pub unsafe extern "C" fn get_object_class(env: *mut JNIEnv, obj: jobject) -> jclass {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let unwrapped = from_object(jvm, obj).unwrap();//todo handle npe
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

pub unsafe fn get_interpreter_state<'k, 'l, 'interpreter_guard>(env: *mut JNIEnv) -> &'l mut InterpreterStateGuard<'l, 'interpreter_guard> {
    let jvm = get_state(env);
    jvm.get_int_state()
}

pub unsafe fn to_object<'gc_life>(obj: Option<GcManagedObject<'gc_life>>) -> jobject {
    match obj {
        None => std::ptr::null_mut(),
        Some(o) => {
            o.self_check();
            let res = o.raw_ptr_usize() as *mut _jobject;
            res
        }
    }
}

pub unsafe fn from_object<'gc_life>(jvm: &'gc_life JVMState<'gc_life>, obj: jobject) -> Option<GcManagedObject<'gc_life>> {
    let option = NonNull::new(obj as *mut Object<'gc_life>)?;
    assert!(jvm.gc.all_allocated_object.read().unwrap().contains(&option));
    Some(GcManagedObject::from_native(option, &jvm.gc))
}

pub unsafe fn from_jclass<'gc_life>(jvm: &'gc_life JVMState<'gc_life>, obj: jclass) -> JClass<'gc_life> {
    try_from_jclass(jvm, obj).unwrap()//todo handle npe
}

pub unsafe fn try_from_jclass<'gc_life>(jvm: &'gc_life JVMState<'gc_life>, obj: jclass) -> Option<JClass<'gc_life>> {
    let possibly_null = from_object(jvm, obj);
    possibly_null.as_ref()?;
    JavaValue::Object(possibly_null).cast_class().into()
}
