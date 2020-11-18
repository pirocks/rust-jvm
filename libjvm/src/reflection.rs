use std::borrow::Borrow;
use std::ops::Deref;

use jvmti_jni_bindings::{jclass, JNIEnv, jobject, jobjectArray};
use slow_interpreter::instructions::invoke::native::mhn_temp::run_static_or_virtual;
use slow_interpreter::interpreter_util::{check_inited_class, push_new_object, run_constructor};
use slow_interpreter::rust_jni::interface::local_frame::new_local_ref_public;
use slow_interpreter::rust_jni::interface::util::class_object_to_runtime_class;
use slow_interpreter::rust_jni::native_util::{from_object, get_interpreter_state, get_state, to_object};
use slow_interpreter::utils::string_obj_to_string;

#[no_mangle]
unsafe extern "system" fn JVM_AllocateNewObject(env: *mut JNIEnv, obj: jobject, currClass: jclass, initClass: jclass) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_SetClassSigners(env: *mut JNIEnv, cls: jclass, signers: jobjectArray) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_InvokeMethod(env: *mut JNIEnv, method: jobject, obj: jobject, args0: jobjectArray) -> jobject {
    //todo need to convert lots of these to unwrap_or_throw
    // dbg!(args0);
    // dbg!(method);
    // dbg!(obj);
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    assert_eq!(obj, std::ptr::null_mut());//non-static methods not supported atm.
    let method_obj = from_object(method).unwrap();
    let args_not_null = from_object(args0).unwrap();
    let args_refcell = args_not_null.unwrap_array().elems.borrow();
    let args = args_refcell.deref();
    let method_name = string_obj_to_string(method_obj.lookup_field("name").unwrap_object());
    let signature = string_obj_to_string(method_obj.lookup_field("signature").unwrap_object());
    let clazz_java_val = method_obj.lookup_field("clazz");
    let target_class_refcell_borrow = clazz_java_val.cast_class().as_type();
    let target_class = target_class_refcell_borrow;
    if target_class.is_primitive() || target_class.is_array() {
        unimplemented!()
    }
    let target_class_name = target_class.unwrap_class_type();
    let target_runtime_class = check_inited_class(jvm, int_state, &target_class_name.into(), int_state.current_loader(jvm).clone()).unwrap();

    //todo this arg array setup is almost certainly wrong.
    for arg in args {
        dbg!(arg);
        int_state.push_current_operand_stack(arg.clone());
    }

    run_static_or_virtual(jvm, int_state, &target_runtime_class, method_name, signature);
    new_local_ref_public(int_state.pop_current_operand_stack().unwrap_object(), int_state)
}

#[no_mangle]
unsafe extern "system" fn JVM_NewInstanceFromConstructor(env: *mut JNIEnv, c: jobject, args0: jobjectArray) -> jobject {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let args = if args0.is_null() {
        vec![]
    } else {
        let temp_1 = from_object(args0).unwrap();
        let array_temp = temp_1.unwrap_array().borrow();
        let elems_refcell = array_temp.elems.borrow();
        elems_refcell.clone()
    };
    let constructor_obj = from_object(c).unwrap();
    let signature_str_obj = constructor_obj.lookup_field("signature");
    let temp_4 = constructor_obj.lookup_field("clazz");
    let clazz = class_object_to_runtime_class(&temp_4.cast_class(), jvm, int_state).unwrap();
    let mut signature = string_obj_to_string(signature_str_obj.unwrap_object());
    push_new_object(jvm, int_state, &clazz, None);
    let obj = int_state.pop_current_operand_stack();
    let mut full_args = vec![obj.clone()];
    full_args.extend(args.iter().cloned());
    run_constructor(jvm, int_state, clazz, full_args, signature);
    new_local_ref_public(obj.unwrap_object(), int_state)
}

