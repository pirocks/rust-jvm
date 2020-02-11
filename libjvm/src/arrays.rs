use jni_bindings::{jobject, jintArray, jclass, JNIEnv, jint, jvalue};

#[no_mangle]
unsafe extern "system" fn JVM_AllocateNewArray(env: *mut JNIEnv, obj: jobject, currClass: jclass, length: jint) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetArrayLength(env: *mut JNIEnv, arr: jobject) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetArrayElement(env: *mut JNIEnv, arr: jobject, index: jint) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetPrimitiveArrayElement(env: *mut JNIEnv, arr: jobject, index: jint, wCode: jint) -> jvalue {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_SetArrayElement(env: *mut JNIEnv, arr: jobject, index: jint, val: jobject) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_SetPrimitiveArrayElement(env: *mut JNIEnv, arr: jobject, index: jint, v: jvalue, vCode: ::std::os::raw::c_uchar) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_NewArray(env: *mut JNIEnv, eltClass: jclass, length: jint) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_NewMultiArray(env: *mut JNIEnv, eltClass: jclass, dim: jintArray) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ArrayCopy(env: *mut JNIEnv, ignored: jclass, src: jobject, src_pos: jint, dst: jobject, dst_pos: jint, length: jint) {
    unimplemented!()
}
