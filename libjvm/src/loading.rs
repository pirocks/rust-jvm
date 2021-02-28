use std::ptr::null_mut;

use jvmti_jni_bindings::{jclass, jint, JNIEnv, jobject, jstring};
use slow_interpreter::java::lang::class_loader::ClassLoader;
use slow_interpreter::rust_jni::native_util::{get_interpreter_state, get_state, to_object};

#[no_mangle]
unsafe extern "system" fn JVM_CurrentLoadedClass(env: *mut JNIEnv) -> jclass {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_CurrentClassLoader(env: *mut JNIEnv) -> jobject {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let loader_name = int_state.current_frame().loader();
    match jvm.get_loader_obj(loader_name) {
        None => null_mut(),
        Some(loader) => to_object(loader.object().into())
    }
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
