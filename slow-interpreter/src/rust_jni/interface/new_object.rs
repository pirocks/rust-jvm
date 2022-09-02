use std::ffi::VaList;
use std::ptr::null_mut;

use jvmti_jni_bindings::{jclass, jmethodID, JNIEnv, jobject, jvalue};

use crate::instructions::invoke::special::invoke_special_impl;
use crate::interpreter_util::new_object;
use crate::{JavaValueCommon, JVMState, pushable_frame_todo};
use method_table::from_jmethod_id;
use crate::better_java_stack::opaque_frame::OpaqueFrame;
use crate::rust_jni::interface::call::VarargProvider;
use crate::rust_jni::interface::local_frame::{new_local_ref_public_new};
use crate::rust_jni::interface::{get_interpreter_state, get_state, push_type_to_operand_stack_new};

pub unsafe extern "C" fn new_object_v(env: *mut JNIEnv, _clazz: jclass, jmethod_id: jmethodID, mut args: VaList) -> jobject {
    new_object_impl(env, _clazz, jmethod_id, VarargProvider::VaList(&mut args))
}

pub unsafe extern "C" fn jni_new_object(env: *mut JNIEnv, _clazz: jclass, jmethod_id: jmethodID, mut l: ...) -> jobject {
    new_object_impl(env, _clazz, jmethod_id, VarargProvider::Dots(&mut l))
}

pub unsafe extern "C" fn new_object_a(env: *mut JNIEnv, clazz: jclass, method_id: jmethodID, args: *const jvalue) -> jobject {
    new_object_impl(env, clazz, method_id, VarargProvider::Array(args))
}

pub unsafe fn new_object_impl<'gc, 'l>(env: *mut JNIEnv, _clazz: jclass, jmethod_id: jmethodID, mut l: VarargProvider) -> jobject {
    let method_id = from_jmethod_id(jmethod_id);
    let jvm: &'gc JVMState<'gc> = get_state(env);
    let int_state = get_interpreter_state(env);
    let (class, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
    let classview = &class.view();
    let method = &classview.method_view_i(method_i);
    let _name = method.name();
    let parsed = method.desc();
    let mut temp : OpaqueFrame<'gc, '_> = todo!();
    let obj = new_object(jvm, /*int_state*/&mut temp, &class);
    let mut args = vec![];
    let mut args_handle = vec![];
    args.push(obj.new_java_value());
    for type_ in &parsed.arg_types {
        args_handle.push(push_type_to_operand_stack_new(jvm, todo!()/*int_state*/, type_, &mut l));
    }
    for arg in args_handle.iter(){
        args.push(arg.as_njv());
    }
    if let Err(_) = invoke_special_impl(jvm, pushable_frame_todo()/*int_state*/, &parsed, method_i, class.clone(), args) {
        return null_mut();
    };
    new_local_ref_public_new(obj.new_java_value().unwrap_object_alloc(), todo!()/*int_state*/)
}