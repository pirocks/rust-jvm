use jvmti_jni_bindings::{jstring, JNIEnv};
use slow_interpreter::rust_jni::interface::string::intern_impl;

#[no_mangle]
unsafe extern "system" fn JVM_InternString(env: *mut JNIEnv, str_unsafe: jstring) -> jstring {
    intern_impl(str_unsafe)
}
