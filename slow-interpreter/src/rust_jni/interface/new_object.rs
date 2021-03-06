use std::ffi::VaList;
use std::ptr::null_mut;

use classfile_view::view::ClassView;
use jvmti_jni_bindings::{jclass, jmethodID, JNIEnv, jobject, jvalue};

use crate::instructions::invoke::special::invoke_special_impl;
use crate::interpreter::WasException;
use crate::interpreter_util::push_new_object;
use crate::method_table::from_jmethod_id;
use crate::rust_jni::interface::call::VarargProvider;
use crate::rust_jni::interface::local_frame::new_local_ref_public;
use crate::rust_jni::interface::push_type_to_operand_stack;
use crate::rust_jni::native_util::{get_interpreter_state, get_state};

pub unsafe extern "C" fn new_object_v(env: *mut JNIEnv, _clazz: jclass, jmethod_id: jmethodID, mut args: VaList) -> jobject {
    new_object_impl(env, _clazz, jmethod_id, VarargProvider::VaList(&mut args))
}

pub unsafe extern "C" fn new_object(env: *mut JNIEnv, _clazz: jclass, jmethod_id: jmethodID, mut l: ...) -> jobject {
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
    let method = &classview.method_view_i(method_i as usize);
    let _name = method.name();
    let parsed = method.desc();
    push_new_object(jvm, int_state, &class);
    let obj = int_state.pop_current_operand_stack();
    int_state.push_current_operand_stack(obj.clone());
    for type_ in &parsed.parameter_types {
        push_type_to_operand_stack(int_state, type_, &mut l)
    }
    if let Err(_) = invoke_special_impl(
        jvm,
        int_state,
        &parsed,
        method_i as usize,
        class.clone(),
    ) {
        return null_mut()
    };
    new_local_ref_public(obj.unwrap_object(), int_state)
}

