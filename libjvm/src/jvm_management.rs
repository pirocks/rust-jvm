use jni_bindings::{jboolean, jint, jobject, JNIEnv, jvm_version_info};

#[no_mangle]
unsafe extern "system" fn JVM_GetInterfaceVersion() -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_OnExit(func: ::std::option::Option<unsafe extern "C" fn()>) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_Exit(code: jint) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_Halt(code: jint) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ActiveProcessorCount() -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_IsSupportedJNIVersion(version: jint) -> jboolean {
    //todo for now we support everything?
    true as jboolean
}


#[no_mangle]
unsafe extern "system" fn JVM_GetManagement(version: jint) -> *mut ::std::os::raw::c_void {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_InitAgentProperties(env: *mut JNIEnv, agent_props: jobject) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetVersionInfo(env: *mut JNIEnv, info: *mut jvm_version_info, info_size: usize) {
    (*info).jvm_version = 8;//todo what should I put here?
}


#[no_mangle]
unsafe extern "system" fn JVM_SupportsCX8() -> jboolean {
    false as jboolean//todo this is actually something that might be easy to support.
}

