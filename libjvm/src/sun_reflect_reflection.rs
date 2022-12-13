use jvmti_jni_bindings::{jclass, JNIEnv, JVM_CALLER_DEPTH};
use crate::introspection::JVM_GetCallerClass;

#[no_mangle]
unsafe extern "system" fn Java_sun_reflect_Reflection_getCallerClass(env: *mut JNIEnv, _cb: jclass) -> jclass {
    JVM_GetCallerClass(env, JVM_CALLER_DEPTH)
}

