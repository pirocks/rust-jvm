use std::borrow::Borrow;
use std::cell::RefCell;
use std::ffi::CStr;
use std::ops::Deref;
use std::sync::Arc;

use classfile_view::view::HasAccessFlags;
use classfile_view::view::method_view::MethodView;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use jvmti_jni_bindings::{jboolean, jbyteArray, jclass, jint, JNIEnv, jobject, jobjectArray, jstring, JVM_ExceptionTableEntryType, jvmtiCapabilities};
use libjvm_utils::ptype_to_class_object;
use rust_jvm_common::classfile::{ACC_ABSTRACT, ACC_PUBLIC};
use rust_jvm_common::classnames::{class_name, ClassName};
use rust_jvm_common::ptype::{PType, ReferenceType};
use slow_interpreter::class_objects::get_or_create_class_object;
use slow_interpreter::instructions::ldc::{create_string_on_stack, load_class_constant_by_type};
use slow_interpreter::interpreter_util::{check_inited_class, push_new_object, run_constructor};
use slow_interpreter::java_values::{ArrayObject, JavaValue};
use slow_interpreter::java_values::Object::Array;
use slow_interpreter::rust_jni::get_all_methods;
use slow_interpreter::rust_jni::interface::local_frame::new_local_ref_public;
use slow_interpreter::rust_jni::interface::string::new_string_with_string;
use slow_interpreter::rust_jni::interface::util::class_object_to_runtime_class;
use slow_interpreter::rust_jni::native_util::{from_jclass, from_object, get_interpreter_state, get_state, to_object};
use slow_interpreter::threading::JavaThread;
use slow_interpreter::threading::monitors::Monitor;

pub mod constant_pool;
pub mod is_x;
pub mod index;
pub mod method_annotations;

#[no_mangle]
unsafe extern "system" fn JVM_GetClassInterfaces(env: *mut JNIEnv, cls: jclass) -> jobjectArray {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let interface_vec = from_jclass(cls).as_runtime_class().view().interfaces().map(|interface| {
        let class_obj = get_or_create_class_object(jvm, &ClassName::Str(interface.interface_name()).into(), int_state, int_state.current_loader(jvm));
        JavaValue::Object(Some(class_obj))
    }).collect::<Vec<_>>();
    //todo helper function for this:
    let res = Some(Arc::new(Array(ArrayObject {
        elems: RefCell::new(interface_vec),
        elem_type: ClassName::class().into(),
        monitor: jvm.thread_state.new_monitor("".to_string()),
    })));
    new_local_ref_public(res, int_state)
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
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let object = from_object(cls);
    let temp = JavaValue::Object(object).cast_class().as_type();
    let object_class = temp.unwrap_ref_type();
    new_local_ref_public(ptype_to_class_object(jvm, int_state, &object_class.unwrap_array().to_ptype()), int_state)
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassModifiers(env: *mut JNIEnv, cls: jclass) -> jint {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let jclass = from_jclass(cls);
    let type_ = jclass.as_type();
    if type_.is_primitive() {
        // is primitive
        // essentially an abstract class of the non-primitive version
        //todo find a better way to do this
        let obj = from_object(cls);
        let type_ = JavaValue::Object(obj).cast_class().as_type();
        let name = type_.unwrap_type_to_name().unwrap();
        let class_for_access_flags = check_inited_class(jvm, int_state, &name.into(), int_state.current_loader(jvm).clone());
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
    let res = from_jclass(cls).as_runtime_class().view().access_flags() as i32;
    res
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
    let int_state = get_interpreter_state(env);
    let type_ = PTypeView::Ref(ReferenceTypeView::Class(int_state.previous_previous_frame().class_pointer().view().name()));
    load_class_constant_by_type(jvm, int_state, &type_);
    let jclass = int_state.pop_current_operand_stack().unwrap_object();
    new_local_ref_public(jclass, int_state)
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
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let name = CStr::from_ptr(&*c_name).to_str().unwrap().to_string();
    new_local_ref_public(Some(get_or_create_class_object(
        jvm,
        &PTypeView::Ref(ReferenceTypeView::Class(ClassName::Str(name))),
        int_state,
        int_state.current_loader(jvm).clone(),
    )), int_state)
}


#[no_mangle]
unsafe extern "system" fn JVM_GetClassName(env: *mut JNIEnv, cls: jclass) -> jstring {
    let obj = from_jclass(cls).as_runtime_class();
    let full_name = &obj.view().name().get_referred_name().replace("/", ".");//todo need a standard way of doing this
    new_string_with_string(env, full_name.to_string())
}

