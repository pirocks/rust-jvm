use std::ptr::null_mut;

use jvmti_jni_bindings::{jboolean, jclass, JNIEnv, jobject, jstring};

#[no_mangle]
unsafe extern "system" fn JVM_InitializeCompiler(_env: *mut JNIEnv, _compCls: jclass) {
    eprintln!("JVM_InitializeCompiler not supported");
}

#[no_mangle]
unsafe extern "system" fn JVM_IsSilentCompiler(_env: *mut JNIEnv, _compCls: jclass) -> jboolean {
    eprintln!("JVM_IsSilentCompiler not supported");
    u8::from(false)
}

#[no_mangle]
unsafe extern "system" fn JVM_CompileClass(_env: *mut JNIEnv, _compCls: jclass, _cls: jclass) -> jboolean {
    eprintln!("JVM_CompileClass not supported");
    u8::from(false)
}

#[no_mangle]
unsafe extern "system" fn JVM_CompileClasses(_env: *mut JNIEnv, _cls: jclass, _jname: jstring) -> jboolean {
    eprintln!("JVM_CompileClasses not supported");
    u8::from(false)
}

#[no_mangle]
unsafe extern "system" fn JVM_CompilerCommand(_env: *mut JNIEnv, _compCls: jclass, _arg: jobject) -> jobject {
    eprintln!("JVM_CompilerCommand not supported");
    null_mut()
}

#[no_mangle]
unsafe extern "system" fn JVM_EnableCompiler(_env: *mut JNIEnv, _compCls: jclass) {
    eprintln!("JVM_EnableCompiler not supported");
}

#[no_mangle]
unsafe extern "system" fn JVM_DisableCompiler(_env: *mut JNIEnv, _compCls: jclass) {
    eprintln!("JVM_DisableCompiler not supported");
}