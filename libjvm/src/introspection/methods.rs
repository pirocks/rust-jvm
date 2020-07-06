use num_cpus::get;

use jvmti_jni_bindings::{jclass, jint, JNIEnv, jobject, jobjectArray};
use slow_interpreter::rust_jni::native_util::{from_jclass, get_state};
use slow_interpreter::rust_jni::value_conversion::native_to_runtime_class;

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodParameters(env: *mut JNIEnv, method: jobject) -> jobjectArray {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetEnclosingMethodInfo(env: *mut JNIEnv, ofClass: jclass) -> jobjectArray {
    if from_jclass(ofClass).as_type().is_primitive() {
        return std::ptr::null_mut();
    }
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