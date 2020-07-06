use jvmti_jni_bindings::{jboolean, jclass, JNIEnv, jobject, jstring};

#[no_mangle]
unsafe extern "system" fn JVM_InitializeCompiler(env: *mut JNIEnv, compCls: jclass) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_IsSilentCompiler(env: *mut JNIEnv, compCls: jclass) -> jboolean {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_CompileClass(env: *mut JNIEnv, compCls: jclass, cls: jclass) -> jboolean {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_CompileClasses(env: *mut JNIEnv, cls: jclass, jname: jstring) -> jboolean {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_CompilerCommand(env: *mut JNIEnv, compCls: jclass, arg: jobject) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_EnableCompiler(env: *mut JNIEnv, compCls: jclass) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_DisableCompiler(env: *mut JNIEnv, compCls: jclass) {
    unimplemented!()
}

