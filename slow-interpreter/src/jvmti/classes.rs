use jvmti_bindings::{jvmtiEnv, jclass, jint, jvmtiError, JVMTI_CLASS_STATUS_INITIALIZED, jvmtiError_JVMTI_ERROR_NONE};
use crate::jvmti::get_state;
use std::mem::transmute;
use classfile_view::view::ptype_view::{ReferenceTypeView, PTypeView};
use crate::class_objects::get_or_create_class_object;
use crate::rust_jni::native_util::to_object;
use std::ops::Deref;

pub unsafe extern "C" fn get_class_status(env: *mut jvmtiEnv, _klass: jclass, status_ptr: *mut jint) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm,"GetClassStatus");
    status_ptr.write(JVMTI_CLASS_STATUS_INITIALIZED as i32);
    //todo actually implement this
//todo handle primitive classes
    jvm.tracing.trace_jdwp_function_exit(jvm,"GetClassStatus");
    jvmtiError_JVMTI_ERROR_NONE
}

pub unsafe extern "C" fn get_loaded_classes(env: *mut jvmtiEnv, class_count_ptr: *mut jint, classes_ptr: *mut *mut jclass) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm,"GetLoadedClasses");
    let frame = jvm.get_current_frame();
    let mut res_vec = vec![];
//todo what about int.class and other primitive classes
    jvm.initialized_classes.read().unwrap().iter().for_each(|(_, runtime_class)| {
        let name = runtime_class.class_view.name();
        let class_object = get_or_create_class_object(jvm, &PTypeView::Ref(ReferenceTypeView::Class(name)), frame.deref(), runtime_class.loader.clone());
        res_vec.push(to_object(class_object.into()))
    });
    class_count_ptr.write(res_vec.len() as i32);
    classes_ptr.write(transmute(Vec::leak(res_vec).as_mut_ptr())); //todo leaking
    jvm.tracing.trace_jdwp_function_exit(jvm,"GetLoadedClasses");
    jvmtiError_JVMTI_ERROR_NONE
}
