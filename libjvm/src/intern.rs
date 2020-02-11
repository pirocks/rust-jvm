use slow_interpreter::rust_jni::string::intern_impl;
use jni_bindings::{jstring, JNIEnv};

#[no_mangle]
unsafe extern "system" fn JVM_InternString(env: *mut JNIEnv, str_unsafe: jstring) -> jstring {
    intern_impl(str_unsafe)
}
