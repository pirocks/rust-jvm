use std::os::raw::c_void;
use std::ptr::NonNull;

use jvmti_jni_bindings::{_jobject, jclass, JNIEnv, jobject};

use crate::{InterpreterStateGuard, JVMState};
use crate::class_objects::get_or_create_class_object;
use crate::java::lang::class::JClass;
use crate::java_values::{GcManagedObject};
use crate::new_java_values::{AllocatedObject, AllocatedObjectHandle, NewJavaValueHandle};
use crate::rust_jni::interface::local_frame::{new_local_ref_public_new};

pub unsafe extern "C" fn get_object_class(env: *mut JNIEnv, obj: jobject) -> jclass {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let unwrapped = from_object_new(jvm, obj).unwrap(); //todo handle npe
    let rc = unwrapped.as_allocated_obj().runtime_class(jvm);
    let class_object = get_or_create_class_object(jvm,rc.cpdtype(),int_state);
    /*let class_object = match unwrapped.deref() {
        Object::Array(a) => get_or_create_class_object(jvm, CPDType::Ref(CPRefType::Array(Box::new(a.elem_type.clone()))), int_state),
        Object::Object(o) => get_or_create_class_object(jvm, o.objinfo.class_pointer.view().type_(), int_state),
    }
        .unwrap(); //todo pass the error up*/

    new_local_ref_public_new(class_object.unwrap().into(), int_state) as jclass
}

pub unsafe fn get_state<'gc>(env: *mut JNIEnv) -> &'gc JVMState<'gc> {
    &(*((**env).reserved0 as *const JVMState))
}

pub unsafe fn get_interpreter_state<'k, 'l, 'interpreter_guard>(env: *mut JNIEnv) -> &'l mut InterpreterStateGuard<'l, 'interpreter_guard> {
    let jvm = get_state(env);
    jvm.get_int_state()
}

pub unsafe fn to_object<'gc>(obj: Option<GcManagedObject<'gc>>) -> jobject {
    match obj {
        None => std::ptr::null_mut(),
        Some(o) => {
            // o.self_check();
            let res = o.raw_ptr_usize() as *mut _jobject;
            res
        }
    }
}

pub unsafe fn to_object_new<'gc>(obj: Option<AllocatedObject<'gc, '_>>) -> jobject {
    match obj {
        None => std::ptr::null_mut(),
        Some(o) => {
            // o.self_check();
            let res = o.raw_ptr_usize() as *mut _jobject;
            res
        }
    }
}

pub unsafe fn from_object<'gc>(jvm: &'gc JVMState<'gc>, obj: jobject) -> Option<GcManagedObject<'gc>> {
    let option = NonNull::new(obj as *mut c_void)?;
    // if !jvm.gc.all_allocated_object.read().unwrap().contains(&option) {
    //     dbg!(option.as_ptr());
    //     dbg!(jvm.gc.all_allocated_object.read().unwrap());
    //     panic!()
    // }
    todo!()
    // Some(GcManagedObject::from_native(option, jvm))
}

pub unsafe fn from_object_new<'gc>(jvm: &'gc JVMState<'gc>, obj: jobject) -> Option<AllocatedObjectHandle> {
    let ptr = NonNull::new(obj as *mut c_void)?;
    let handle = jvm.gc.register_root_reentrant(jvm, ptr);
    Some(handle)
}

pub unsafe fn from_jclass<'gc>(jvm: &'gc JVMState<'gc>, obj: jclass) -> JClass<'gc> {//all jclasses have life of 'gc
    try_from_jclass(jvm, obj).unwrap()
    //todo handle npe
}

pub unsafe fn try_from_jclass<'gc>(jvm: &'gc JVMState<'gc>, obj: jclass) -> Option<JClass<'gc>> { //all jclasses have life of 'gc
    let possibly_null = from_object_new(jvm, obj);
    let not_null = possibly_null?;
    NewJavaValueHandle::Object(not_null).cast_class()
}