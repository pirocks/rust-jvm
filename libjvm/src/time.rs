use jvmti_jni_bindings::{jlong, jclass, JNIEnv, lchmod};
use std::time::Instant;
use slow_interpreter::rust_jni::native_util::get_state;

#[no_mangle]
unsafe extern "system" fn JVM_CurrentTimeMillis(env: *mut JNIEnv, ignored: jclass) -> jlong {
    let now = Instant::now();
    let jvm = get_state(env);
    now.duration_since(jvm.start_instant).as_millis() as jlong //todo dup
}

#[no_mangle]
unsafe extern "system" fn JVM_NanoTime(env: *mut JNIEnv, ignored: jclass) -> jlong {
    let now = Instant::now();
    let jvm = get_state(env);
    now.duration_since(jvm.start_instant).as_nanos() as jlong
}
