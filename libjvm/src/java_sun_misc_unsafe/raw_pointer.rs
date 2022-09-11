use std::mem::transmute;
use std::ptr::null_mut;

use jvmti_jni_bindings::{jbyte, jint, jlong, JNIEnv, jobject};

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
unsafe extern "system" fn Java_sun_misc_Unsafe_copyMemory(env: *mut JNIEnv, the_unsafe: jobject, src_obj: jobject, offset: jlong, dst_obj: jobject, address: jlong, len: jlong) {
    todo!("update for new offset")
    /*let jvm = get_state(env);
    let nonnull = match from_object(jvm, src_obj) {
        Some(x) => x,
        None => return throw_npe(get_state(env), get_interpreter_state(env)),
    };
    let as_array = nonnull.unwrap_array(); //not defined for non-byte-array objects
    assert_eq!(as_array.elem_type, CPDType::ByteType);
    let array_mut = as_array;
    let src_slice_indices = offset..(offset + len);
    let mut src_buffer: Vec<i8> = vec![];
    for i in src_slice_indices {
        src_buffer.push(array_mut.get_i(jvm, i as i32).unwrap_byte());
    }
    assert_eq!(dst_obj, null_mut());
    libc::memcpy(transmute(address), src_buffer.as_ptr() as *const libc::c_void, len as usize);*/
}


#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_allocateMemory(env: *mut JNIEnv, the_unsafe: jobject, len: jlong) -> jlong {
    let res: i64 = libc::malloc(len as usize) as i64;
    res
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
