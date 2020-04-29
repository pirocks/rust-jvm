use jvmti_jni_bindings::jint;

#[no_mangle]
unsafe extern "system" fn JVM_RawMonitorCreate() -> *mut ::std::os::raw::c_void {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_RawMonitorDestroy(mon: *mut ::std::os::raw::c_void) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_RawMonitorEnter(mon: *mut ::std::os::raw::c_void) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_RawMonitorExit(mon: *mut ::std::os::raw::c_void) {
    unimplemented!()
}

