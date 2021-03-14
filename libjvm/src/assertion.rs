use jvmti_jni_bindings::{jboolean, jclass, JNIEnv, jobject};
use slow_interpreter::rust_jni::native_util::get_state;

#[no_mangle]
unsafe extern "system" fn JVM_DesiredAssertionStatus(env: *mut JNIEnv, _unused: jclass, _cls: jclass) -> jboolean {
    let jvm = get_state(env);
    u8::from(jvm.assertions_enabled)
}

#[no_mangle]
unsafe extern "system" fn JVM_AssertionStatusDirectives(env: *mut JNIEnv, _unused: jclass) -> jobject {
    unimplemented!()
}
