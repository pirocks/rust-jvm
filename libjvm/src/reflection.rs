use std::borrow::Borrow;
use slow_interpreter::rust_jni::native_util::{get_state, get_frame, to_object, from_object};
use slow_interpreter::interpreter_util::{push_new_object, run_constructor};
use jni_bindings::{jobject, jobjectArray, JNIEnv, jclass};

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
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_NewInstanceFromConstructor(env: *mut JNIEnv, c: jobject, args0: jobjectArray) -> jobject {
//    assert_ne!(args0, std::ptr::null_mut());
    let args = if args0 == std::ptr::null_mut() {
        vec![]
    } else {
        let temp_1 = from_object(args0).unwrap().clone();
        let array_temp = temp_1.unwrap_array().borrow();
        let elems_refcell = array_temp.elems.borrow();
        elems_refcell.clone()
    };
    let constructor_obj = from_object(c).unwrap();
    let constructor_obj_fields = constructor_obj.unwrap_normal_object().fields.borrow();
    let signature_str_obj = constructor_obj_fields.get("signature").unwrap();
    let temp_4 = constructor_obj_fields.get("clazz").unwrap().unwrap_object().unwrap();
    let temp_3 = temp_4.unwrap_normal_object().object_class_object_pointer.borrow();
    let clazz = temp_3.as_ref().unwrap().clone();
    let temp_2 = signature_str_obj.unwrap_object().unwrap().unwrap_normal_object().fields.borrow().get("value").unwrap().unwrap_object().unwrap();
    let sig_chars = &temp_2.unwrap_array().borrow().elems;
    let mut signature = String::new();
    for char_ in sig_chars.borrow().iter() {
        signature.push(char_.unwrap_char())
    }
    let state = get_state(env);
    let frame = get_frame(env);
    push_new_object(frame.clone(), &clazz);
    let obj = frame.pop();
    let mut full_args = vec![obj.clone()];
    full_args.extend(args.iter().cloned());
    run_constructor(state, frame, clazz, full_args, signature);
    to_object(obj.unwrap_object())
}

