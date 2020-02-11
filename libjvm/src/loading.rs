use jni_bindings::{jclass, jstring, jobject, JNIEnv, jint};

#[no_mangle]
unsafe extern "system" fn JVM_CurrentLoadedClass(env: *mut JNIEnv) -> jclass {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_CurrentClassLoader(env: *mut JNIEnv) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ClassLoaderDepth(env: *mut JNIEnv) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_LoadClass0(env: *mut JNIEnv, obj: jobject, currClass: jclass, currClassName: jstring) -> jclass {
    unimplemented!()
}


#[no_mangle]
unsafe extern "system" fn JVM_LatestUserDefinedLoader(env: *mut JNIEnv) -> jobject {
    unimplemented!()
}
#[no_mangle]
unsafe extern "system" fn JVM_GetClassLoader(env: *mut JNIEnv, cls: jclass) -> jobject {
    unimplemented!()
}
