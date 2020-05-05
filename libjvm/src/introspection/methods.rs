use jvmti_jni_bindings::{JNIEnv, jobject, jobjectArray, jclass, jint};
use slow_interpreter::rust_jni::native_util::{get_frame, get_state, from_jclass};
use slow_interpreter::rust_jni::value_conversion::native_to_runtime_class;
use num_cpus::get;

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodParameters(env: *mut JNIEnv, method: jobject) -> jobjectArray {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetEnclosingMethodInfo(env: *mut JNIEnv, ofClass: jclass) -> jobjectArray {
    let state = get_state(env);
    let frame = get_frame(env);
    let em = from_jclass(ofClass).as_runtime_class().view().enclosing_method_view();
    match em {
        None => std::ptr::null_mut(),
        Some(_) => unimplemented!(),
    }
}



#[no_mangle]
unsafe extern "system" fn JVM_GetClassMethodsCount(env: *mut JNIEnv, cb: jclass) -> jint {
    unimplemented!()
}