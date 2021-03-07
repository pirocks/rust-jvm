use jvmti_jni_bindings::{JNIEnv, jstring};
use slow_interpreter::rust_jni::interface::string::intern_impl_unsafe;
use slow_interpreter::rust_jni::native_util::{get_interpreter_state, get_state};

#[no_mangle]
unsafe extern "system" fn JVM_InternString(env: *mut JNIEnv, str_unsafe: jstring) -> jstring {
    let jvm = get_state(env);
    intern_impl_unsafe(jvm, str_unsafe)
}
