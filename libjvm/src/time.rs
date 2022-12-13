use std::time::{Instant, SystemTime, UNIX_EPOCH};

use jvmti_jni_bindings::{jclass, jlong, JNIEnv, lchmod};
use slow_interpreter::rust_jni::jni_utils::{get_interpreter_state, get_state};

#[no_mangle]
unsafe extern "system" fn JVM_CurrentTimeMillis(_env: *mut JNIEnv, _ignored: jclass) -> jlong {
    let unix_epoch = UNIX_EPOCH;
    SystemTime::now().duration_since(unix_epoch).unwrap().as_millis() as jlong
}

#[no_mangle]
unsafe extern "system" fn JVM_NanoTime(env: *mut JNIEnv, _ignored: jclass) -> jlong {
    let now = Instant::now();
    let jvm = get_state(env);
    now.duration_since(jvm.start_instant).as_nanos() as jlong
}