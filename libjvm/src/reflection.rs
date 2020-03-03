use std::borrow::Borrow;
use slow_interpreter::rust_jni::native_util::{get_state, get_frame, to_object, from_object};
use slow_interpreter::interpreter_util::{push_new_object, run_constructor};
use jni_bindings::{jobject, jobjectArray, JNIEnv, jclass};
use utils::string_obj_to_string;
use slow_interpreter::rust_jni::interface::util::class_object_to_runtime_class;

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
    let clazz = class_object_to_runtime_class(temp_4.unwrap_normal_object(), state, &frame);
    let mut signature = string_obj_to_string(signature_str_obj.unwrap_object());
    push_new_object(frame.clone(), &clazz);
    let obj = frame.pop();
    let mut full_args = vec![obj.clone()];
    full_args.extend(args.iter().cloned());
    run_constructor(state, frame, clazz, full_args, signature);
    to_object(obj.unwrap_object())
}

