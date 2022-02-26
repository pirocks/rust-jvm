use std::ffi::VaList;
use std::ptr::null_mut;

use jvmti_jni_bindings::{jclass, jmethodID, JNIEnv, jobject, jvalue};
use rust_jvm_common::compressed_classfile::names::CClassName;

use crate::instructions::invoke::special::invoke_special_impl;
use crate::interpreter_util::new_object;
use crate::method_table::from_jmethod_id;
use crate::rust_jni::interface::call::VarargProvider;
use crate::rust_jni::interface::local_frame::{new_local_ref_public, new_local_ref_public_new};
use crate::rust_jni::interface::{push_type_to_operand_stack, push_type_to_operand_stack_new};
use crate::rust_jni::native_util::{get_interpreter_state, get_state};

pub unsafe extern "C" fn new_object_v(env: *mut JNIEnv, _clazz: jclass, jmethod_id: jmethodID, mut args: VaList) -> jobject {
    new_object_impl(env, _clazz, jmethod_id, VarargProvider::VaList(&mut args))
}

pub unsafe extern "C" fn jni_new_object(env: *mut JNIEnv, _clazz: jclass, jmethod_id: jmethodID, mut l: ...) -> jobject {
    new_object_impl(env, _clazz, jmethod_id, VarargProvider::Dots(&mut l))
}

pub unsafe extern "C" fn new_object_a(env: *mut JNIEnv, clazz: jclass, method_id: jmethodID, args: *const jvalue) -> jobject {
    new_object_impl(env, clazz, method_id, VarargProvider::Array(args))
}

pub unsafe fn new_object_impl(env: *mut JNIEnv, _clazz: jclass, jmethod_id: jmethodID, mut l: VarargProvider) -> jobject {
    let method_id = from_jmethod_id(jmethod_id);
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let (class, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
    let classview = &class.view();
    let method = &classview.method_view_i(method_i);
    let _name = method.name();
    let parsed = method.desc();
    let obj = new_object(jvm, int_state, &class);
    let mut args = vec![];
    let mut args_handle = vec![];
    args.push(obj.new_java_value());
    for type_ in &parsed.arg_types {
        args_handle.push(push_type_to_operand_stack_new(jvm, int_state, type_, &mut l));
    }
    for arg in args_handle.iter(){
        args.push(arg.as_njv());
    }
    if let Err(_) = invoke_special_impl(jvm, int_state, &parsed, method_i, class.clone(), args) {
        return null_mut();
    };
    new_local_ref_public_new(obj.new_java_value().unwrap_object_alloc(), int_state)
}