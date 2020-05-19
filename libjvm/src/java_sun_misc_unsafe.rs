use jvmti_jni_bindings::{JNIEnv, jclass, jobject, jint, JVM_CALLER_DEPTH, jlong, jboolean, jbyte};
use crate::introspection::JVM_GetCallerClass;
use slow_interpreter::rust_jni::native_util::from_object;
use std::ptr::null_mut;
use std::intrinsics::transmute;
use classfile_view::view::ptype_view::PTypeView;
use std::ops::Deref;

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

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_copyMemory(
    env: *mut JNIEnv,
    the_unsafe: jobject,
    src_obj: jobject,
    offset: jlong,
    dst_obj: jobject,
    address: jlong,
    len: jlong,
) {
    let nonnull = from_object(src_obj).unwrap();
    let as_array = nonnull.unwrap_array();//not defined for non-byte-array objects
    assert_eq!(as_array.elem_type, PTypeView::ByteType);
    let array_mut = as_array.elems.borrow_mut();
    let src_slice = &array_mut.deref()[offset as usize..((offset + len) as usize)];
    let mut src_buffer: Vec<i8> = vec![];
    for src_elem in src_slice {
        src_buffer.push(src_elem.unwrap_byte());
    }
    assert_eq!(dst_obj, null_mut());
    libc::memcpy(transmute(address), src_buffer.as_ptr() as *const libc::c_void, len as usize);
}


#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_putByte__JB(env: *mut JNIEnv,
                                                           the_unsafe: jobject,
                                                           address: jlong,
                                                           byte_: jbyte,
) {
    let byte_addr: *mut jbyte = transmute(address);
    byte_addr.write(byte_);
}