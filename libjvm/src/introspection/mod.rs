use jni_bindings::{jobjectArray, jclass, JNIEnv, jobject, jint, jstring, jbyteArray, jboolean, JVM_ExceptionTableEntryType};
use slow_interpreter::rust_jni::native_util::{to_object, get_state, get_frame, from_object};
use std::sync::Arc;
use runtime_common::java_values::{Object, ArrayObject, JavaValue};
use std::cell::RefCell;
use rust_jvm_common::ptype::{PType, ReferenceType};
use rust_jvm_common::classnames::{class_name, ClassName};
use slow_interpreter::interpreter_util::{run_constructor, push_new_object, check_inited_class};
use slow_interpreter::instructions::ldc::{create_string_on_stack, load_class_constant_by_type};
use runtime_common::{StackEntry, InterpreterState};
use std::rc::Rc;
use slow_interpreter::{array_of_type_class, get_or_create_class_object};
use rust_jvm_common::classfile::{ACC_PUBLIC, ACC_ABSTRACT};
use std::ops::Deref;
use std::ffi::CStr;
use slow_interpreter::rust_jni::interface::util::{runtime_class_from_object, class_object_to_runtime_class};
use slow_interpreter::rust_jni::interface::string::new_string_with_string;


use libjvm_utils::ptype_to_class_object;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use classfile_view::view::descriptor_parser::parse_method_descriptor;
use std::borrow::Borrow;
use slow_interpreter::rust_jni::get_all_methods;
use classfile_view::view::HasAccessFlags;
use classfile_view::view::method_view::MethodView;
use classfile_view::view::descriptor_parser::Descriptor::Method;
use runtime_common::runtime_class::RuntimeClass;

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
    let state = get_state(env);
    let frame = get_frame(env);
    let object_non_null = from_object(cls).unwrap().clone();
    let temp = object_non_null.unwrap_normal_object().class_object_ptype.borrow();
    let object_class = temp.as_ref().unwrap().unwrap_ref_type();
    to_object(ptype_to_class_object(state, &frame, &object_class.unwrap_array().to_ptype()))
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassModifiers(env: *mut JNIEnv, cls: jclass) -> jint {
    let frame = get_frame(env);
    let state = get_state(env);

    match runtime_class_from_object(cls, state, &frame) {
        None => {
            // is primitive
            // essentially an abstract class of the non-primitive version
            //todo find a better way to do this
            let obj = from_object(cls).unwrap();
            let type_ = obj.unwrap_normal_object().class_object_to_ptype();
            let name = type_.unwrap_type_to_name().unwrap();//if type_ == PTypeView::IntType {
                // ClassName::int()
            // }else {
            //     dbg!(type_);
            //     unimplemented!()
            // };
            let class_for_access_flags = check_inited_class(state, &name, frame.clone().into(), frame.class_pointer.loader.clone());
            (class_for_access_flags.class_view.access_flags() | ACC_ABSTRACT) as jint
        }
        Some(rc) => {
            rc.class_view.access_flags() as jint
        }
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
    runtime_class_from_object(cls, get_state(env), &get_frame(env)).unwrap().classfile.access_flags as i32
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
    let frame = get_frame(env);
    let state = get_state(env);

    load_class_constant_by_type(state, &frame, &PTypeView::Ref(ReferenceTypeView::Class(frame.last_call_stack.as_ref().unwrap().class_pointer.class_view.name())));
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
    let state = get_state(env);
    let frame = get_frame(env);

    let name = CStr::from_ptr(&*c_name).to_str().unwrap().to_string();
    to_object(Some(get_or_create_class_object(state, &PTypeView::Ref(ReferenceTypeView::Class(ClassName::Str(name))), frame.clone(), frame.class_pointer.loader.clone())))
}


#[no_mangle]
unsafe extern "system" fn JVM_GetClassName(env: *mut JNIEnv, cls: jclass) -> jstring {
    let obj = runtime_class_from_object(cls, get_state(env), &get_frame(env)).unwrap();
    let full_name = class_name(&obj.classfile).get_referred_name().replace("/", ".");
    new_string_with_string(env, full_name)
}

