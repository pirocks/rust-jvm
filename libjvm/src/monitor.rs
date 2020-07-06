use jvmti_jni_bindings::{jlong, JNIEnv, jobject};
use slow_interpreter::jvmti::get_jvmti_interface;
use slow_interpreter::rust_jni::native_util::{from_object, get_interpreter_state, get_state};

#[no_mangle]
unsafe extern "system" fn JVM_MonitorWait(env: *mut JNIEnv, obj: jobject, ms: jlong) {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    // int_state.print_stack_trace();
    from_object(obj).unwrap().monitor().wait(ms, jvm);
}

#[no_mangle]
unsafe extern "system" fn JVM_MonitorNotify(env: *mut JNIEnv, obj: jobject) {
    let jvm = get_state(env);
    from_object(obj).unwrap().monitor().notify(jvm);
}

#[no_mangle]
unsafe extern "system" fn JVM_MonitorNotifyAll(env: *mut JNIEnv, obj: jobject) {
    let jvm = get_state(env);
    from_object(obj).unwrap().monitor().notify_all(jvm);
}

