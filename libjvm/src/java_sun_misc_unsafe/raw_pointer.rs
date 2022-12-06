use std::intrinsics::{size_of, volatile_copy_memory};
use std::mem::transmute;
use std::ptr::null_mut;
use itertools::repeat_n;
use libc::{c_void, time};

use jvmti_jni_bindings::{jbyte, jchar, jfloat, jint, jlong, JNIEnv, jobject, jshort};

use slow_interpreter::better_java_stack::frames::HasFrame;
use slow_interpreter::new_java_values::java_value_common::JavaValueCommon;

use slow_interpreter::rust_jni::native_util::{from_object, from_object_new};
use slow_interpreter::utils::throw_npe;
use slow_interpreter::rust_jni::jni_utils::{get_interpreter_state, get_state};

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_putByte__JB(env: *mut JNIEnv, the_unsafe: jobject, address: jlong, byte_: jbyte) {
    let byte_addr: *mut jbyte = transmute(address);
    byte_addr.write(byte_);
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_putInt__JI(env: *mut JNIEnv, the_unsafe: jobject, address: jlong, int_: jint) {
    let int_addr: *mut jint = transmute(address);
    int_addr.write(int_)
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_putChar__JC(env: *mut JNIEnv, the_unsafe: jobject, address: jlong, int_: jchar) {
    let int_addr: *mut jchar = transmute(address);
    int_addr.write(int_)
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_putShort__JC(env: *mut JNIEnv, the_unsafe: jobject, address: jlong, int_: jshort) {
    let int_addr: *mut jshort = transmute(address);
    int_addr.write(int_)
}


#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_putLong__JJ(env: *mut JNIEnv, the_unsafe: jobject, ptr: jlong, val: jlong) {
    let ptr: *mut i64 = transmute(ptr);
    ptr.write(val);
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_getByte__J(env: *mut JNIEnv, the_unsafe: jobject, ptr: jlong) -> jbyte {
    let ptr: *mut i8 = transmute(ptr);
    ptr.read()
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_getInt__J(env: *mut JNIEnv, the_unsafe: jobject, ptr: jlong) -> i32 {
    let ptr: *mut i32 = transmute(ptr);
    ptr.read()
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_getLong__J(env: *mut JNIEnv, the_unsafe: jobject, ptr: jlong) -> i64 {
    let ptr: *mut i64 = transmute(ptr);
    ptr.read()
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_getShort__J(env: *mut JNIEnv, the_unsafe: jobject, ptr: jlong) -> jshort {
    let ptr: *mut jshort = transmute(ptr);
    ptr.read()
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_getFloat__J(env: *mut JNIEnv, the_unsafe: jobject, ptr: jlong) -> jfloat {
    let ptr: *mut jfloat = transmute(ptr);
    ptr.read()
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
unsafe extern "system" fn Java_sun_misc_Unsafe_copyMemory(env: *mut JNIEnv, the_unsafe: jobject, src_obj: jobject, offset: jlong, dst_obj: jobject, address: jlong, len: jlong) {
    let jvm = get_state(env);
    let src_address = if src_obj == null_mut() {
        offset as *const i8
    } else {
        (src_obj as *const i8).offset(offset as isize)
    };
    let dst_address = if dst_obj == null_mut() {
        address as *mut i8
    } else {
        //todo have an address calulation function
        (dst_obj as *mut i8).offset(address as isize)
    };
    volatile_copy_memory(dst_address as *mut u8, src_address as *const u8, len as usize);
    return;
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_allocateMemory(env: *mut JNIEnv, the_unsafe: jobject, len: jlong) -> jlong {
    let res: i64 = libc::malloc(len as usize) as i64;
    res
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_getAddress(env: *mut JNIEnv, the_unsafe: jobject, address: jlong) -> jlong {
    address
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_freeMemory(env: *mut JNIEnv, the_unsafe: jobject, ptr: jlong) {
    libc::free(transmute(ptr))
}


#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_setMemory(env: *mut JNIEnv, the_unsafe: jobject, o: jobject, offset: jlong, bytes: jlong, value: jbyte) {
    assert_eq!(o, null_mut());

    for i in offset..(offset + bytes) {
        let address = i as *mut jbyte;
        address.write(value)
    }
}
