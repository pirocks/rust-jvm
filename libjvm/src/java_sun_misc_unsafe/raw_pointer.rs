use std::intrinsics::{size_of, volatile_copy_memory};
use std::mem::transmute;
use std::ptr::null_mut;
use libc::c_void;
use gc_memory_layout_common::layout::ArrayMemoryLayout;

use jvmti_jni_bindings::{jbyte, jint, jlong, JNIEnv, jobject};

use rust_jvm_common::NativeJavaValue;
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
    get_interpreter_state(env).debug_print_stack_trace(jvm);
    let src_address = if src_obj == null_mut() {
        offset as *const i8
    } else {
        todo!()
    };
    let dst_address = if dst_obj == null_mut() {
        todo!()
        /*address as *mut i8*/
    } else {
        //todo have an address calulation function
        (dst_obj as *mut i8).offset(address as isize)
    };
    for i in 0..len {
        dbg!(src_address.offset(i as isize).read());
    }
    assert!(len > 0);
    //todo this needs a better more general impl
    // volatile_copy_memory(dst_address, src_address, len as usize)
    for i in 0..len{
        let temp = src_address.offset(i as isize).read();
        dst_address.offset((i as usize * size_of::<NativeJavaValue>() as usize) as isize).write(temp);
    }

    // let nonnull = match from_object_new(jvm, src_obj) {
    //     Some(x) => x,
    //     None => {
    //         dbg!(offset as *mut c_void);
    //         todo!()}/*return throw_npe(jvm, get_interpreter_state(env))*/,
    // };
    // let as_array = nonnull.unwrap_array(); //not defined for non-byte-array objects
    // assert_eq!(as_array.elem_cpdtype(), CPDType::ByteType);
    // let array_mut = as_array;
    // let src_slice_indices = offset..(offset + len);
    // let mut src_buffer: Vec<i8> = vec![];
    // for i in src_slice_indices {
    //     src_buffer.push(array_mut.get_i(i as usize).unwrap_byte_strict());
    // }
    // assert_eq!(dst_obj, null_mut());
    // libc::memcpy(transmute(address), src_buffer.as_ptr() as *const libc::c_void, len as usize);
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
