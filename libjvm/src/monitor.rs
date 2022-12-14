use std::ptr::null_mut;
use std::time::Duration;

use libc::c_void;

use jvmti_jni_bindings::{jlong, JNIEnv, jobject};

use slow_interpreter::rust_jni::jni_utils::{get_interpreter_state, get_state};

#[no_mangle]
unsafe extern "system" fn JVM_MonitorWait(env: *mut JNIEnv, obj: jobject, ms: jlong) {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    assert_ne!(obj, null_mut());
    let monitor = jvm.monitor_for(obj as *const c_void);
    let duration = if ms == 0 { None } else { Some(Duration::from_millis(ms as u64)) };
    match monitor.wait(jvm, int_state, duration) {
        Ok(x) => x,
        Err(_) => todo!(),
    };
}

#[no_mangle]
unsafe extern "system" fn JVM_MonitorNotify(env: *mut JNIEnv, obj: jobject) {
    let jvm = get_state(env);
    assert_ne!(obj, null_mut());
    let monitor = jvm.monitor_for(obj as *const c_void);
    monitor.notify(jvm).expect("todo");
}

#[no_mangle]
unsafe extern "system" fn JVM_MonitorNotifyAll(env: *mut JNIEnv, obj: jobject) {
    let jvm = get_state(env);
    assert_ne!(obj, null_mut());
    let monitor = jvm.monitor_for(obj as *const c_void);
    monitor.notify_all(jvm).expect("todo");
}