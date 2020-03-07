use jni_bindings::{JNIEnv, jobject, jobjectArray, jclass};
use slow_interpreter::rust_jni::native_util::{get_frame, get_state};
use slow_interpreter::rust_jni::value_conversion::native_to_runtime_class;

use slow_interpreter::rust_jni::interface::util::runtime_class_from_object;
use num_cpus::get;

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodParameters(env: *mut JNIEnv, method: jobject) -> jobjectArray {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetEnclosingMethodInfo(env: *mut JNIEnv, ofClass: jclass) -> jobjectArray {
    let state = get_state(env);
    let frame = get_frame(env);
    frame.print_stack_trace();
    let em = runtime_class_from_object(ofClass, state, &frame).and_then(|x| x.class_view.enclosing_method_view());
    match em {
        None => std::ptr::null_mut(),
        Some(_) => unimplemented!(),
    }
}



#[no_mangle]
unsafe extern "system" fn JVM_GetClassMethodsCount(env: *mut JNIEnv, cb: jclass) -> jint {
    unimplemented!()
}