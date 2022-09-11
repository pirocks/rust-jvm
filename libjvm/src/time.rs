use std::time::Instant;

use jvmti_jni_bindings::{jclass, jlong, JNIEnv, lchmod};
use slow_interpreter::rust_jni::jni_interface::jni::get_state;

#[no_mangle]
unsafe extern "system" fn JVM_CurrentTimeMillis(env: *mut JNIEnv, ignored: jclass) -> jlong {
    let now = Instant::now();
    let jvm = get_state(env);
    now.duration_since(jvm.start_instant).as_millis() as jlong
}

#[no_mangle]
unsafe extern "system" fn JVM_NanoTime(env: *mut JNIEnv, ignored: jclass) -> jlong {
    let now = Instant::now();
    let jvm = get_state(env);
    now.duration_since(jvm.start_instant).as_nanos() as jlong
}