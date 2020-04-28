use jvmti_bindings::{jvmtiEnv, jclass, jint, jvmtiError, JVMTI_CLASS_STATUS_INITIALIZED, jvmtiError_JVMTI_ERROR_NONE, JVMTI_CLASS_STATUS_ARRAY, JVMTI_CLASS_STATUS_PREPARED, JVMTI_CLASS_STATUS_VERIFIED, JVMTI_CLASS_STATUS_PRIMITIVE};
use crate::jvmti::get_state;
use std::mem::transmute;
use classfile_view::view::ptype_view::{ReferenceTypeView, PTypeView};
use crate::class_objects::get_or_create_class_object;
use crate::rust_jni::native_util::{to_object, from_object};
use std::ops::Deref;

pub unsafe extern "C" fn get_class_status(env: *mut jvmtiEnv, klass: jclass, status_ptr: *mut jint) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm,"GetClassStatus");
    let class = from_object(transmute(klass)).unwrap();//todo handle null
    let res = match class.unwrap_normal_object().class_object_ptype.borrow().as_ref() {
        None => {
            0
        },
        Some(type_) => {
            let mut status = 0;
            status |= JVMTI_CLASS_STATUS_PREPARED as i32;
            status |= JVMTI_CLASS_STATUS_VERIFIED as i32;
            status |= JVMTI_CLASS_STATUS_INITIALIZED as i32;//todo so technically this isn't correct, b/c we don't check static intializer completeness
            match type_ {
                PTypeView::Ref(ref_) => {
                    match ref_{
                        ReferenceTypeView::Class(_) => {},
                        ReferenceTypeView::Array(array) => {
                            status |= JVMTI_CLASS_STATUS_ARRAY as i32;
                        },
                    }
                },
                _ => {status |= JVMTI_CLASS_STATUS_PRIMITIVE as i32;},
            };
            status
        },
    };
    status_ptr.write(res);


    //    JVMTI_CLASS_STATUS_VERIFIED	1	Class bytecodes have been verified
    //     JVMTI_CLASS_STATUS_PREPARED	2	Class preparation is complete
    //     JVMTI_CLASS_STATUS_INITIALIZED	4	Class initialization is complete. Static initializer has been run.
    //     JVMTI_CLASS_STATUS_ERROR	8	Error during initialization makes class unusable
    //     JVMTI_CLASS_STATUS_ARRAY	16	Class is an array. If set, all other bits are zero.
    //     JVMTI_CLASS_STATUS_PRIMITIVE	32	Class is a primitive class (for example, java.lang.Integer.TYPE). If set, all other bits are zero.
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