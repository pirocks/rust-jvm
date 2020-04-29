use jvmti_jni_bindings::{JNIEnv, jclass, jobject, jint, JVM_CALLER_DEPTH, jlong, jboolean};
use crate::introspection::JVM_GetCallerClass;
#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_registerNatives(
    env: *mut JNIEnv,
    cb: jclass) -> () {
    //todo for now register nothing, register later as needed.
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_arrayBaseOffset(env: *mut JNIEnv,
                                                               obj: jobject,
                                                               cb: jclass) -> jint {
    0//unimplemented but can't return nothing.
    //essentially the amount at the beginning of the array which is reserved
}


#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_arrayIndexScale(env: *mut JNIEnv,
                                                               obj: jobject,
                                                               cb: jclass) -> jint {
    1//todo unimplemented but can't return nothing, and need to return a power of 2,1 counts as a power of two. This essentially reprs the size of an elem in java arrays
}


#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_addressSize(env: *mut JNIEnv,
                                                           obj: jobject) -> jint {
    64//officially speaking unimplemented but can't return nothing, and should maybe return something reasonable todo
}

#[no_mangle]
unsafe extern "system" fn Java_sun_reflect_Reflection_getCallerClass(env: *mut JNIEnv,
                                                                     cb: jclass) -> jclass {
    return JVM_GetCallerClass(env, JVM_CALLER_DEPTH);
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_compareAndSwapObject(
    env: *mut JNIEnv,
    the_unsafe: jobject,
    obj: jobject,
    offset: jlong,
    obj1: jobject,
    obj2: jobject,
) -> jboolean {
//    if mangled == "Java_sun_misc_Unsafe_compareAndSwapObject".to_string() {
//        //todo do nothing for now and see what happens
//        Some(JavaValue::Boolean(true))
//    }
//    unimplemented!()
    true as jboolean
}
