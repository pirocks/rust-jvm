use std::ffi::c_void;
use jvmti_jni_bindings::{jboolean, jbyte, jchar, jdouble, jfloat, jint, jlong, JNIEnv, jobject, jshort};
use crate::double_register_addressing::calc_address;

#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putByteVolatile(_env: *mut JNIEnv, _the_unsafe: jobject, obj: jobject, offset: jlong, val: jbyte) {
    calc_address(obj, offset).cast::<jbyte>().write(val)
}

#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putBooleanVolatile(_env: *mut JNIEnv, _the_unsafe: jobject, obj: jobject, offset: jlong, val: jboolean) {
    calc_address(obj, offset).cast::<jboolean>().write(val)
}

#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putShortVolatile(_env: *mut JNIEnv, _the_unsafe: jobject, obj: jobject, offset: jlong, val: jshort) {
    calc_address(obj, offset).cast::<jshort>().write(val)
}

#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putCharVolatile(_env: *mut JNIEnv, _the_unsafe: jobject, obj: jobject, offset: jlong, val: jchar) {
    calc_address(obj, offset).cast::<jchar>().write(val)
}

#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putIntVolatile(_env: *mut JNIEnv, _the_unsafe: jobject, obj: jobject, offset: jlong, val: jint) {
    obj.cast::<c_void>().offset(offset as isize).cast::<jint>().write(val)
}

#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putLongVolatile(_env: *mut JNIEnv, _the_unsafe: jobject, obj: jobject, offset: jlong, val: jlong) {
    calc_address(obj, offset).cast::<jlong>().write(val)
}

#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putFloatVolatile(_env: *mut JNIEnv, _the_unsafe: jobject, obj: jobject, offset: jlong, val: jfloat) {
    calc_address(obj, offset).cast::<jfloat>().write(val)
}

#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putDoubleVolatile(_env: *mut JNIEnv, _the_unsafe: jobject, obj: jobject, offset: jlong, val: jdouble) {
    calc_address(obj, offset).cast::<jdouble>().write(val)
}

#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_putObjectVolatile(_env: *mut JNIEnv, _the_unsafe: jobject, obj: jobject, offset: jlong, to_put: jobject) {
    calc_address(obj, offset).cast::<jobject>().write(to_put)
}

#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getBooleanVolatile(_env: *mut JNIEnv, _the_unsafe: jobject, obj: jobject, offset: jlong) -> jboolean {
    calc_address(obj, offset).cast::<jboolean>().read()
}

#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getByteVolatile(_env: *mut JNIEnv, _the_unsafe: jobject, obj: jobject, offset: jlong) -> jbyte {
    calc_address(obj, offset).cast::<jbyte>().read()
}

#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getCharVolatile(_env: *mut JNIEnv, _the_unsafe: jobject, obj: jobject, offset: jlong) -> jchar {
    calc_address(obj, offset).cast::<jchar>().read()
}

#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getShortVolatile(_env: *mut JNIEnv, _the_unsafe: jobject, obj: jobject, offset: jlong) -> jshort {
    calc_address(obj, offset).cast::<jshort>().read()
}

#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getIntVolatile(_env: *mut JNIEnv, _the_unsafe: jobject, obj: jobject, offset: jlong) -> jint {
    calc_address(obj, offset).cast::<jint>().read()
}

#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getFloatVolatile(_env: *mut JNIEnv, _the_unsafe: jobject, obj: jobject, offset: jlong) -> jfloat {
    calc_address(obj, offset).cast::<jfloat>().read()
}

#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getDoubleVolatile(_env: *mut JNIEnv, _the_unsafe: jobject, obj: jobject, offset: jlong) -> jdouble {
    calc_address(obj, offset).cast::<jdouble>().read()
}

#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getLongVolatile(_env: *mut JNIEnv, _the_unsafe: jobject, obj: jobject, offset: jlong) -> jlong {
    calc_address(obj, offset).cast::<jlong>().read()
}

#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_getObjectVolatile(_env: *mut JNIEnv, _the_unsafe: jobject, obj: jobject, offset: jlong) -> jobject {
    calc_address(obj, offset).cast::<jobject>().read()
}