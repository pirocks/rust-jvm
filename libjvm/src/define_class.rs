use jvmti_jni_bindings::{jbyte, jclass, JNIEnv, jobject, jsize};

#[no_mangle]
unsafe extern "system" fn JVM_DefineClass(env: *mut JNIEnv, name: *const ::std::os::raw::c_char, loader: jobject, buf: *const jbyte, len: jsize, pd: jobject) -> jclass {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_DefineClassWithSource(env: *mut JNIEnv, name: *const ::std::os::raw::c_char, loader: jobject, buf: *const jbyte, len: jsize, pd: jobject, source: *const ::std::os::raw::c_char) -> jclass {
    unimplemented!()
}
