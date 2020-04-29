use jvmti_jni_bindings::{jvmtiEnv, jclass, jint, jvmtiError, JVMTI_CLASS_STATUS_INITIALIZED, jvmtiError_JVMTI_ERROR_NONE, JVMTI_CLASS_STATUS_ARRAY, JVMTI_CLASS_STATUS_PREPARED, JVMTI_CLASS_STATUS_VERIFIED, JVMTI_CLASS_STATUS_PRIMITIVE, jmethodID};
use crate::jvmti::get_state;
use std::mem::transmute;
use classfile_view::view::ptype_view::{ReferenceTypeView, PTypeView};
use crate::class_objects::get_or_create_class_object;
use crate::rust_jni::native_util::{to_object, from_object};
use std::ops::Deref;
use std::ffi::{CString, c_void};
use crate::interpreter_util::check_inited_class;
use crate::rust_jni::MethodId;
use std::mem::size_of;

pub unsafe extern "C" fn get_class_status(env: *mut jvmtiEnv, klass: jclass, status_ptr: *mut jint) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm,"GetClassStatus");
    let class = from_object(transmute(klass)).unwrap();//todo handle null
    let res = match class.unwrap_normal_object().class_object_ptype.as_ref() {
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



pub unsafe extern "C" fn get_class_signature(env: *mut jvmtiEnv, klass: jclass, signature_ptr: *mut *mut ::std::os::raw::c_char, generic_ptr: *mut *mut ::std::os::raw::c_char) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm, "GetClassSignature");
    let notnull_class = from_object(transmute(klass)).unwrap();
    let class_object_ptype = notnull_class.unwrap_normal_object().class_object_ptype.clone();
    let type_ = class_object_ptype.as_ref().unwrap();
    if !signature_ptr.is_null() {
        let jvm_repr = CString::new(type_.jvm_representation()).unwrap();
        let jvm_repr_ptr = jvm_repr.into_raw();
        let allocated_jvm_repr = libc::malloc(libc::strlen(jvm_repr_ptr) + 1) as *mut ::std::os::raw::c_char;
        signature_ptr.write(allocated_jvm_repr);
        libc::strcpy(allocated_jvm_repr, jvm_repr_ptr);
    }
    if !generic_ptr.is_null() {
        let java_repr = CString::new(type_.java_source_representation()).unwrap();
        let java_repr_ptr = java_repr.into_raw();
        let allocated_java_repr = libc::malloc(libc::strlen(java_repr_ptr) + 1) as *mut ::std::os::raw::c_char;
        generic_ptr.write(allocated_java_repr);
        libc::strcpy(allocated_java_repr, java_repr_ptr);
    }
    jvm.tracing.trace_jdwp_function_exit(jvm, "GetClassSignature");
    jvmtiError_JVMTI_ERROR_NONE
}


pub unsafe extern "C" fn get_class_methods(env: *mut jvmtiEnv, klass: jclass, method_count_ptr: *mut jint, methods_ptr: *mut *mut jmethodID) -> jvmtiError{
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm, "GetClassMethods");
    let class_object_wrapped = from_object(transmute(klass)).unwrap();
    let class = class_object_wrapped.unwrap_normal_object();
    let class_ptype_guard = class.class_object_ptype.clone();
    let class_name = class_ptype_guard.as_ref().unwrap().unwrap_class_type();
    let loaded_class = check_inited_class(jvm,&class_name,jvm.get_current_frame().deref().class_pointer.loader.clone());
    method_count_ptr.write(loaded_class.class_view.num_methods() as i32);
    //todo use Layout instead of whatever this is.
    *methods_ptr = libc::malloc((size_of::<*mut c_void>())*(*method_count_ptr as usize)) as *mut *mut jvmti_jni_bindings::_jmethodID;
    loaded_class.class_view.methods().enumerate().for_each(|(i,mv)|{
        methods_ptr
            .read()
            .offset(i as isize)
            .write( Box::leak(box MethodId { class: loaded_class.clone(), method_i: mv.method_i() }) as *mut MethodId as jmethodID)
    });
    jvm.tracing.trace_jdwp_function_exit(jvm, "GetClassMethods");
    jvmtiError_JVMTI_ERROR_NONE
}
