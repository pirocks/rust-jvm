use std::borrow::Borrow;
use slow_interpreter::rust_jni::native_util::{get_state, get_frame, to_object, from_object};
use slow_interpreter::interpreter_util::{push_new_object, run_constructor, check_inited_class};
use jni_bindings::{jobject, jobjectArray, JNIEnv, jclass};
use slow_interpreter::rust_jni::interface::util::class_object_to_runtime_class;
use std::ops::Deref;
use slow_interpreter::instructions::invoke::native::mhn_temp::run_static_or_virtual;
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
    let frame = get_frame(env);
    let state = get_state(env);
    //todo need to convert lots of these to unwrap_or_throw
    dbg!(args0);
    dbg!(method);
    dbg!(obj);
    assert_eq!(obj, std::ptr::null_mut());//non-static methods not supported atm.
    let method_obj = from_object(method).unwrap();
    let args_not_null = from_object(args0).unwrap();
    let args_refcell = args_not_null.unwrap_array().elems.borrow();
    let args = args_refcell.deref();
    let method_name = string_obj_to_string(method_obj.lookup_field("name").unwrap_object());
    let signature = string_obj_to_string(method_obj.lookup_field("signature").unwrap_object());
    let clazz_java_val = method_obj.lookup_field("clazz");
    let target_class_refcell_borrow = clazz_java_val.unwrap_normal_object().class_object_ptype.borrow();
    let target_class = target_class_refcell_borrow.as_ref().unwrap();
    if target_class.is_primitive() || target_class.is_array() {
        unimplemented!()
    }
    let target_class_name = target_class.unwrap_class_type();
    let target_runtime_class = check_inited_class(state,&target_class_name,frame.clone().into(),frame.class_pointer.loader.clone());

    //todo this arg array setup is almost certainly wrong.
    for arg in args {
        dbg!(arg);
        frame.push(arg.clone());
    }

    run_static_or_virtual(state,&frame,&target_runtime_class,method_name,signature);
    to_object(frame.pop().unwrap_object())
}

#[no_mangle]
unsafe extern "system" fn JVM_NewInstanceFromConstructor(env: *mut JNIEnv, c: jobject, args0: jobjectArray) -> jobject {
    let args = if args0 == std::ptr::null_mut() {
        vec![]
    } else {
        let temp_1 = from_object(args0).unwrap().clone();
        let array_temp = temp_1.unwrap_array().borrow();
        let elems_refcell = array_temp.elems.borrow();
        elems_refcell.clone()
    };
    let constructor_obj = from_object(c).unwrap();
    let signature_str_obj = constructor_obj.lookup_field("signature");
    let temp_4 = constructor_obj.lookup_field("clazz").unwrap_object_nonnull();
    let state = get_state(env);
    let frame = get_frame(env);
    let clazz = class_object_to_runtime_class(temp_4.unwrap_normal_object(), state, &frame).unwrap();
    let mut signature = string_obj_to_string(signature_str_obj.unwrap_object());
    push_new_object(state,frame.clone(), &clazz);
    let obj = frame.pop();
    let mut full_args = vec![obj.clone()];
    full_args.extend(args.iter().cloned());
    run_constructor(state, frame, clazz, full_args, signature);
    to_object(obj.unwrap_object())
}

