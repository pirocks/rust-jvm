use jni_bindings::{JNIEnv, jclass, jobject, jint, jobjectArray, jfloat, jlong, jdouble, jstring};

#[no_mangle]
unsafe extern "system" fn JVM_GetClassConstantPool(env: *mut JNIEnv, cls: jclass) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetSize(env: *mut JNIEnv, unused: jobject, jcpool: jobject) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetClassAt(env: *mut JNIEnv, unused: jobject, jcpool: jobject, index: jint) -> jclass {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetClassAtIfLoaded(env: *mut JNIEnv, unused: jobject, jcpool: jobject, index: jint) -> jclass {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetMethodAt(env: *mut JNIEnv, unused: jobject, jcpool: jobject, index: jint) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetMethodAtIfLoaded(env: *mut JNIEnv, unused: jobject, jcpool: jobject, index: jint) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetFieldAt(env: *mut JNIEnv, unused: jobject, jcpool: jobject, index: jint) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetFieldAtIfLoaded(env: *mut JNIEnv, unused: jobject, jcpool: jobject, index: jint) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetMemberRefInfoAt(env: *mut JNIEnv, unused: jobject, jcpool: jobject, index: jint) -> jobjectArray {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetIntAt(env: *mut JNIEnv, unused: jobject, jcpool: jobject, index: jint) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetLongAt(env: *mut JNIEnv, unused: jobject, jcpool: jobject, index: jint) -> jlong {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetFloatAt(env: *mut JNIEnv, unused: jobject, jcpool: jobject, index: jint) -> jfloat {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetDoubleAt(env: *mut JNIEnv, unused: jobject, jcpool: jobject, index: jint) -> jdouble {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetStringAt(env: *mut JNIEnv, unused: jobject, jcpool: jobject, index: jint) -> jstring {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetUTF8At(env: *mut JNIEnv, unused: jobject, jcpool: jobject, index: jint) -> jstring {
    unimplemented!()
}

