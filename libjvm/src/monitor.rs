use jni_bindings::{jobject, JNIEnv, jlong};

#[no_mangle]
unsafe extern "system" fn JVM_MonitorWait(env: *mut JNIEnv, obj: jobject, ms: jlong) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_MonitorNotify(env: *mut JNIEnv, obj: jobject) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_MonitorNotifyAll(env: *mut JNIEnv, obj: jobject) {
    //todo unimpl for now, since we don't support mutlithreading anyway
}

