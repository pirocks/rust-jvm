use jvmti_jni_bindings::{jboolean, jbyte, jchar, jdouble, jfloat, jint, jlong, JNIEnv, jobject, jshort};

use crate::object_access_volatile::{Java_sun_misc_Unsafe_getBooleanVolatile, Java_sun_misc_Unsafe_getByteVolatile, Java_sun_misc_Unsafe_getCharVolatile, Java_sun_misc_Unsafe_getDoubleVolatile, Java_sun_misc_Unsafe_getFloatVolatile, Java_sun_misc_Unsafe_getIntVolatile, Java_sun_misc_Unsafe_getLongVolatile, Java_sun_misc_Unsafe_getObjectVolatile, Java_sun_misc_Unsafe_getShortVolatile, Java_sun_misc_Unsafe_putBooleanVolatile, Java_sun_misc_Unsafe_putByteVolatile, Java_sun_misc_Unsafe_putCharVolatile, Java_sun_misc_Unsafe_putDoubleVolatile, Java_sun_misc_Unsafe_putFloatVolatile, Java_sun_misc_Unsafe_putIntVolatile, Java_sun_misc_Unsafe_putLongVolatile, Java_sun_misc_Unsafe_putObjectVolatile, Java_sun_misc_Unsafe_putShortVolatile};

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_getBoolean(env: *mut JNIEnv, the_unsafe: jobject, obj: jobject, offset: jlong) -> jboolean {
    Java_sun_misc_Unsafe_getBooleanVolatile(env, the_unsafe, obj, offset)
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_getObject(env: *mut JNIEnv, the_unsafe: jobject, obj: jobject, offset: jlong) -> jobject {
    Java_sun_misc_Unsafe_getObjectVolatile(env, the_unsafe, obj, offset)
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_getByte__Ljava_lang_Object_2J(env: *mut JNIEnv, the_unsafe: jobject, obj: jobject, offset: jlong) -> jbyte {
    Java_sun_misc_Unsafe_getByteVolatile(env, the_unsafe, obj, offset)
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_getShort__Ljava_lang_Object_2J(env: *mut JNIEnv, the_unsafe: jobject, obj: jobject, offset: jlong) -> jshort {
    Java_sun_misc_Unsafe_getShortVolatile(env, the_unsafe, obj, offset)
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_getChar__Ljava_lang_Object_2J(env: *mut JNIEnv, the_unsafe: jobject, obj: jobject, offset: jlong) -> jchar {
    Java_sun_misc_Unsafe_getCharVolatile(env, the_unsafe, obj, offset)
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_getInt__Ljava_lang_Object_2J(env: *mut JNIEnv, the_unsafe: jobject, obj: jobject, offset: jlong) -> jint {
    Java_sun_misc_Unsafe_getIntVolatile(env, the_unsafe, obj, offset)
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_getFloat__Ljava_lang_Object_2J(env: *mut JNIEnv, the_unsafe: jobject, obj: jobject, offset: jlong) -> jfloat {
    Java_sun_misc_Unsafe_getFloatVolatile(env, the_unsafe, obj, offset)
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_getDouble__Ljava_lang_Object_2J(env: *mut JNIEnv, the_unsafe: jobject, obj: jobject, offset: jlong) -> jdouble {
    Java_sun_misc_Unsafe_getDoubleVolatile(env, the_unsafe, obj, offset)
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_getLong__Ljava_lang_Object_2J(env: *mut JNIEnv, the_unsafe: jobject, obj: jobject, offset: jlong) -> jlong {
    Java_sun_misc_Unsafe_getLongVolatile(env, the_unsafe, obj, offset)
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_putOrderedObject(env: *mut JNIEnv, the_unsafe: jobject, obj: jobject, offset: jlong, to_put: jobject) {
    Java_sun_misc_Unsafe_putObjectVolatile(env, the_unsafe, obj, offset, to_put)
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_putOrderedInt(env: *mut JNIEnv, the_unsafe: jobject, obj: jobject, offset: jlong, val: jint) {
    Java_sun_misc_Unsafe_putIntVolatile(env, the_unsafe, obj, offset, val)
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_putObject(env: *mut JNIEnv, the_unsafe: jobject, obj: jobject, offset: jlong, to_put: jobject) {
    Java_sun_misc_Unsafe_putObjectVolatile(env, the_unsafe, obj, offset, to_put)
}
#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_putBoolean(env: *mut JNIEnv, the_unsafe: jobject, obj: jobject, offset: jlong, to_put: jboolean) {
    Java_sun_misc_Unsafe_putBooleanVolatile(env, the_unsafe, obj, offset, to_put)
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_putByte__Ljava_lang_Object_2JB(env: *mut JNIEnv, the_unsafe: jobject, obj: jobject, offset: jlong, val: jbyte) {
    Java_sun_misc_Unsafe_putByteVolatile(env, the_unsafe, obj, offset, val)
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_putShort__Ljava_lang_Object_2JS(env: *mut JNIEnv, the_unsafe: jobject, obj: jobject, offset: jlong, val: jshort) {
    Java_sun_misc_Unsafe_putShortVolatile(env, the_unsafe, obj, offset, val)
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_putChar__Ljava_lang_Object_2JC(env: *mut JNIEnv, the_unsafe: jobject, obj: jobject, offset: jlong, val: jchar) {
    Java_sun_misc_Unsafe_putCharVolatile(env, the_unsafe, obj, offset, val)
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_putInt__Ljava_lang_Object_2JI(env: *mut JNIEnv, the_unsafe: jobject, obj: jobject, offset: jlong, val: jint) {
    Java_sun_misc_Unsafe_putIntVolatile(env, the_unsafe, obj, offset, val)
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_putFloat__Ljava_lang_Object_2JF(env: *mut JNIEnv, the_unsafe: jobject, obj: jobject, offset: jlong, val: jfloat) {
    Java_sun_misc_Unsafe_putFloatVolatile(env, the_unsafe, obj, offset, val)
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_putDouble__Ljava_lang_Object_2JD(env: *mut JNIEnv, the_unsafe: jobject, obj: jobject, offset: jlong, val: jdouble) {
    Java_sun_misc_Unsafe_putDoubleVolatile(env, the_unsafe, obj, offset, val)
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_putLong__Ljava_lang_Object_2JJ(env: *mut JNIEnv, the_unsafe: jobject, obj: jobject, offset: jlong, long_: jlong) {
    Java_sun_misc_Unsafe_putLongVolatile(env, the_unsafe, obj, offset, long_)
}