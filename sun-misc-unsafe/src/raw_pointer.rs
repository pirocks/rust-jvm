use std::intrinsics::{volatile_copy_memory};
use std::mem::transmute;
use std::ptr::null_mut;

use jvmti_jni_bindings::{jbyte, jchar, jfloat, jint, jlong, JNIEnv, jobject, jshort};
use crate::double_register_addressing::calc_address;


#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_putByte__JB(_env: *mut JNIEnv, _the_unsafe: jobject, address: jlong, byte_: jbyte) {
    let byte_addr: *mut jbyte = transmute(address);
    byte_addr.write(byte_);
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_putShort__JS(_env: *mut JNIEnv, _the_unsafe: jobject, address: jlong, short_: jshort) {
    let short_addr: *mut jshort = transmute(address);
    short_addr.write(short_);
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_putChar__JC(_env: *mut JNIEnv, _the_unsafe: jobject, address: jlong, char_: jchar) {
    let char_addr: *mut jchar = transmute(address);
    char_addr.write(char_)
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_putInt__JI(__env: *mut JNIEnv, _the_unsafe: jobject, address: jlong, int_: jint) {
    let int_addr: *mut jint = transmute(address);
    int_addr.write(int_)
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_putLong__JJ(_env: *mut JNIEnv, _the_unsafe: jobject, ptr: jlong, long_: jlong) {
    let long_ptr: *mut i64 = transmute(ptr);
    long_ptr.write(long_);
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_getByte__J(_env: *mut JNIEnv, _the_unsafe: jobject, ptr: jlong) -> jbyte {
    let byte_ptr: *mut jbyte = transmute(ptr);
    byte_ptr.read()
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_getShort__J(_env: *mut JNIEnv, _the_unsafe: jobject, ptr: jlong) -> jshort {
    let short_ptr: *mut jshort = transmute(ptr);
    short_ptr.read()
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_getChar__J(_env: *mut JNIEnv, _the_unsafe: jobject, ptr: jlong) -> jchar {
    let short_ptr: *mut jchar = transmute(ptr);
    short_ptr.read()
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_getInt__J(_env: *mut JNIEnv, _the_unsafe: jobject, ptr: jlong) -> jint {
    let int_ptr: *mut i32 = transmute(ptr);
    int_ptr.read()
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_getLong__J(_env: *mut JNIEnv, _the_unsafe: jobject, ptr: jlong) -> jlong {
    let long_ptr: *mut i64 = transmute(ptr);
    long_ptr.read()
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_getFloat__J(_env: *mut JNIEnv, _the_unsafe: jobject, ptr: jlong) -> jfloat {
    let float_ptr: *mut jfloat = transmute(ptr);
    float_ptr.read()
}

/**
* Sets all bytes in a given block of memory to a copy of another
* block.
*
* <p>This method determines each block's base address by means of two parameters,
* and so it provides (in effect) a <em>double-register</em> addressing mode,
* as discussed in {@link #getInt(Object,long)}.  When the object reference is null,
* the offset supplies an absolute base address.
*
* <p>The transfers are in coherent (atomic) units of a size determined
* by the address and length parameters.  If the effective addresses and
* length are all even modulo 8, the transfer takes place in 'long' units.
* If the effective addresses and length are (resp.) even modulo 4 or 2,
* the transfer takes place in units of 'int' or 'short'.
*
* @since 1.7
*/
#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_copyMemory(_env: *mut JNIEnv, _the_unsafe: jobject, src_obj: jobject, offset: jlong, dst_obj: jobject, address: jlong, len: jlong) {
    let src_address = calc_address(src_obj, offset) as *const u8;
    let dst_address = calc_address(dst_obj, address) as *mut u8;
    volatile_copy_memory(dst_address, src_address, len as usize);
    return;
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_allocateMemory(_env: *mut JNIEnv, _the_unsafe: jobject, len: jlong) -> jlong {
    let res: jlong = libc::malloc(len as usize) as i64;
    res
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_getAddress(_env: *mut JNIEnv, _the_unsafe: jobject, address: jlong) -> jlong {
    address
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_freeMemory(_env: *mut JNIEnv, _the_unsafe: jobject, ptr: jlong) {
    libc::free(transmute(ptr))
}


#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_setMemory(_env: *mut JNIEnv, _the_unsafe: jobject, o: jobject, offset: jlong, bytes: jlong, value: jbyte) {
    assert_eq!(o, null_mut());// todo handle npe?

    for i in offset..(offset + bytes) {
        let address = i as *mut jbyte;
        address.write(value)
    }
}
