use jni_bindings::{jlong, jclass, JNIEnv, lchmod};
use std::time::Instant;
use slow_interpreter::rust_jni::native_util::get_state;

#[no_mangle]
unsafe extern "system" fn JVM_CurrentTimeMillis(env: *mut JNIEnv, ignored: jclass) -> jlong {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_NanoTime(env: *mut JNIEnv, ignored: jclass) -> jlong {
    let now = Instant::now();
    let state = get_state(env);
    now.duration_since(state.start_instant).as_nanos() as jlong
}
