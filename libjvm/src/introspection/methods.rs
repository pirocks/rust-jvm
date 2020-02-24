use jni_bindings::{JNIEnv, jobject, jobjectArray, jclass};
use slow_interpreter::rust_jni::native_util::get_frame;
use slow_interpreter::rust_jni::value_conversion::native_to_runtime_class;

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodParameters(env: *mut JNIEnv, method: jobject) -> jobjectArray {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetEnclosingMethodInfo(env: *mut JNIEnv, ofClass: jclass) -> jobjectArray {
    let frame = get_frame(env);
    frame.print_stack_trace();
    native_to_runtime_class(ofClass).class_view
        //EnclosingMethod attribute.
    unimplemented!()
}
