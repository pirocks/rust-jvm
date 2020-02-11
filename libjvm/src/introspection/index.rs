use jni_bindings::{JNIEnv, jclass, jint};

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxExceptionsCount(env: *mut JNIEnv, cb: jclass, method_index: jint) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxByteCode(env: *mut JNIEnv, cb: jclass, method_index: jint, code: *mut ::std::os::raw::c_uchar) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxByteCodeLength(env: *mut JNIEnv, cb: jclass, method_index: jint) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxExceptionTableLength(env: *mut JNIEnv, cb: jclass, index: ::std::os::raw::c_int) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetFieldIxModifiers(env: *mut JNIEnv, cb: jclass, index: ::std::os::raw::c_int) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxModifiers(env: *mut JNIEnv, cb: jclass, index: ::std::os::raw::c_int) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxLocalsCount(env: *mut JNIEnv, cb: jclass, index: ::std::os::raw::c_int) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxArgsSize(env: *mut JNIEnv, cb: jclass, index: ::std::os::raw::c_int) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxMaxStack(env: *mut JNIEnv, cb: jclass, index: ::std::os::raw::c_int) -> jint {
    unimplemented!()
}


#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxNameUTF(env: *mut JNIEnv, cb: jclass, index: jint) -> *const ::std::os::raw::c_char {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxSignatureUTF(env: *mut JNIEnv, cb: jclass, index: jint) -> *const ::std::os::raw::c_char {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxExceptionTableEntry(
    env: *mut JNIEnv,
    cb: jclass,
    method_index: jint,
    entry_index: jint,
    entry: *mut JVM_ExceptionTableEntryType,
) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxExceptionIndexes(
    env: *mut JNIEnv,
    cb: jclass,
    method_index: jint,
    exceptions: *mut ::std::os::raw::c_ushort,
) {
    unimplemented!()
}