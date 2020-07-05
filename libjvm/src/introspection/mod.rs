use jvmti_jni_bindings::{jobjectArray, jclass, JNIEnv, jobject, jint, jstring, jbyteArray, jboolean, JVM_ExceptionTableEntryType, jvmtiCapabilities};
use slow_interpreter::rust_jni::native_util::{to_object, get_state, get_frame, from_object, from_jclass};
use std::sync::Arc;
use std::cell::RefCell;
use rust_jvm_common::ptype::{PType, ReferenceType};
use rust_jvm_common::classnames::{class_name, ClassName};
use slow_interpreter::interpreter_util::{run_constructor, push_new_object, check_inited_class};
use slow_interpreter::instructions::ldc::{create_string_on_stack, load_class_constant_by_type};

use rust_jvm_common::classfile::{ACC_PUBLIC, ACC_ABSTRACT};
use std::ops::Deref;
use std::ffi::CStr;
use slow_interpreter::rust_jni::interface::util::class_object_to_runtime_class;
use slow_interpreter::rust_jni::interface::string::new_string_with_string;


use libjvm_utils::ptype_to_class_object;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use std::borrow::Borrow;
use slow_interpreter::rust_jni::get_all_methods;
use classfile_view::view::HasAccessFlags;
use classfile_view::view::method_view::MethodView;
use slow_interpreter::class_objects::get_or_create_class_object;
use slow_interpreter::java_values::JavaValue;
use slow_interpreter::get_state_thread_frame;
use slow_interpreter::rust_jni::native_util::{ get_frames, get_thread};
use slow_interpreter::threading::JavaThread;

pub mod constant_pool;
pub mod is_x;
pub mod index;
pub mod method_annotations;

#[no_mangle]
unsafe extern "system" fn JVM_GetClassInterfaces(env: *mut JNIEnv, cls: jclass) -> jobjectArray {
    unimplemented!()
}


#[no_mangle]
unsafe extern "system" fn JVM_GetClassSigners(env: *mut JNIEnv, cls: jclass) -> jobjectArray {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetProtectionDomain(env: *mut JNIEnv, cls: jclass) -> jobject {
    unimplemented!()
}


#[no_mangle]
unsafe extern "system" fn JVM_GetComponentType(env: *mut JNIEnv, cls: jclass) -> jclass {
    let object = from_object(cls);
    let temp = JavaValue::Object(object).cast_class().as_type();
    let object_class = temp.unwrap_ref_type();
    to_object(ptype_to_class_object(jvm, frame, &object_class.unwrap_array().to_ptype()))
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassModifiers(env: *mut JNIEnv, cls: jclass) -> jint {
    let jclass = from_jclass(cls);
    let type_ = jclass.as_type();
    if type_.is_primitive() {
        // is primitive
        // essentially an abstract class of the non-primitive version
        //todo find a better way to do this
        let obj = from_object(cls);
        let type_ = JavaValue::Object(obj).cast_class().as_type();
        let name = type_.unwrap_type_to_name().unwrap();
        let class_for_access_flags = check_inited_class(jvm, &name.into(), frame.class_pointer.loader(jvm).clone());
        (class_for_access_flags.view().access_flags() | ACC_ABSTRACT) as jint
    } else {
        jclass.as_runtime_class().view().access_flags() as jint
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_GetDeclaredClasses(env: *mut JNIEnv, ofClass: jclass) -> jobjectArray {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetDeclaringClass(env: *mut JNIEnv, ofClass: jclass) -> jclass {
    //todo unimplemented for now
    std::ptr::null_mut()
    //unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassSignature(env: *mut JNIEnv, cls: jclass) -> jstring {
    unimplemented!()
}

pub mod get_methods;

#[no_mangle]
unsafe extern "system" fn JVM_GetClassAccessFlags(env: *mut JNIEnv, cls: jclass) -> jint {
    from_jclass(cls).as_runtime_class().view().access_flags() as i32
}


#[no_mangle]
unsafe extern "system" fn JVM_ClassDepth(env: *mut JNIEnv, name: jstring) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassContext(env: *mut JNIEnv) -> jobjectArray {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassNameUTF(env: *mut JNIEnv, cb: jclass) -> *const ::std::os::raw::c_char {
    unimplemented!()
}

pub mod fields;
pub mod methods;


#[no_mangle]
pub unsafe extern "system" fn JVM_GetCallerClass(env: *mut JNIEnv, depth: ::std::os::raw::c_int) -> jclass {
    /*todo, so this is needed for booting but it is what could best be described as an advanced feature.
    Therefore it only sorta works*/
    let jvm = get_state(env);
    let thread = get_thread(env);
    let mut frames = get_frames(&thread);
    let len = frames.len();
    let type_ = PTypeView::Ref(ReferenceTypeView::Class(frames[len - 2].class_pointer.view().name()));
    let frame = get_frame(&mut frames);
    load_class_constant_by_type(jvm, frame, &type_);
    let jclass = frame.pop().unwrap_object();
    to_object(jclass)
}


#[no_mangle]
unsafe extern "system" fn JVM_IsSameClassPackage(env: *mut JNIEnv, class1: jclass, class2: jclass) -> jboolean {
    unimplemented!()
}


#[no_mangle]
unsafe extern "system" fn JVM_FindClassFromCaller(
    env: *mut JNIEnv,
    c_name: *const ::std::os::raw::c_char,
    init: jboolean,
    loader: jobject,
    caller: jclass,
) -> jclass {
    let name = CStr::from_ptr(&*c_name).to_str().unwrap().to_string();
    to_object(Some(get_or_create_class_object(
        jvm,
        &PTypeView::Ref(ReferenceTypeView::Class(ClassName::Str(name))),
        frame,
        frame.class_pointer.loader(jvm).clone(),
    )))
}


#[no_mangle]
unsafe extern "system" fn JVM_GetClassName(env: *mut JNIEnv, cls: jclass) -> jstring {
    let obj = from_jclass(cls).as_runtime_class();
    let full_name = &obj.view().name().get_referred_name().replace("/", ".");//todo need a standard way of doing this
    new_string_with_string(env, full_name.to_string())
}

