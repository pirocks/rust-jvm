use jvmti_jni_bindings::{JNIEnv, jobject};

#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_loadFence(_env: *mut JNIEnv, _the_unsafe: jobject){}

#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_storeFence(_env: *mut JNIEnv, _the_unsafe: jobject){}

#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_fullFence(_env: *mut JNIEnv, _the_unsafe: jobject){
    //todo new compiler will need to take these into account
}
