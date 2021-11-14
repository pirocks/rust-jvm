use std::ptr::null_mut;

use jvmti_jni_bindings::{jboolean, jclass, JNIEnv, jobject, jstring};

#[no_mangle]
unsafe extern "system" fn JVM_InitializeCompiler(env: *mut JNIEnv, compCls: jclass) {
    eprintln!("JVM_InitializeCompiler not supported");
}

#[no_mangle]
unsafe extern "system" fn JVM_IsSilentCompiler(env: *mut JNIEnv, compCls: jclass) -> jboolean {
    eprintln!("JVM_IsSilentCompiler not supported");
    u8::from(false)
}

#[no_mangle]
unsafe extern "system" fn JVM_CompileClass(env: *mut JNIEnv, compCls: jclass, cls: jclass) -> jboolean {
    eprintln!("JVM_CompileClass not supported");
    u8::from(false)
}

#[no_mangle]
unsafe extern "system" fn JVM_CompileClasses(env: *mut JNIEnv, cls: jclass, jname: jstring) -> jboolean {
    eprintln!("JVM_CompileClasses not supported");
    u8::from(false)
}

#[no_mangle]
unsafe extern "system" fn JVM_CompilerCommand(env: *mut JNIEnv, compCls: jclass, arg: jobject) -> jobject {
    eprintln!("JVM_CompilerCommand not supported");
    null_mut()
}

#[no_mangle]
unsafe extern "system" fn JVM_EnableCompiler(env: *mut JNIEnv, compCls: jclass) {
    eprintln!("JVM_EnableCompiler not supported");
}

#[no_mangle]
unsafe extern "system" fn JVM_DisableCompiler(env: *mut JNIEnv, compCls: jclass) {
    eprintln!("JVM_DisableCompiler not supported");
}