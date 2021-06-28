use std::time::Duration;

use jvmti_jni_bindings::{jlong, JNIEnv, jobject};
use slow_interpreter::jvmti::get_jvmti_interface;
use slow_interpreter::rust_jni::native_util::{from_object, get_interpreter_state, get_state};

#[no_mangle]
unsafe extern "system" fn JVM_MonitorWait(env: *mut JNIEnv, obj: jobject, ms: jlong) {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let monitor_obj = from_object(jvm, obj).expect("null monitor?");
    let monitor_to_wait = monitor_obj.monitor();
    monitor_to_wait.wait(jvm, int_state, Some(Duration::from_millis(ms as u64)));
}

#[no_mangle]
unsafe extern "system" fn JVM_MonitorNotify(env: *mut JNIEnv, obj: jobject) {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    from_object(jvm, obj).expect("null monitor?").monitor().notify(jvm);
}

#[no_mangle]
unsafe extern "system" fn JVM_MonitorNotifyAll(env: *mut JNIEnv, obj: jobject) {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    from_object(jvm, obj).expect("null monitor?").monitor().notify_all(jvm);
}

