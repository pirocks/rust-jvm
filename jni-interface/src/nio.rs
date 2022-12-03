use std::ffi::c_void;
use jvmti_jni_bindings::{jlong, JNIEnv, jobject};
use slow_interpreter::rust_jni::jni_utils::{get_interpreter_state, get_state, new_local_ref_public_new};
use slow_interpreter::stdlib::java::NewAsObjectOrJavaValue;
use slow_interpreter::stdlib::java::nio::direct_byte_buffer::DirectByteBuffer;

pub unsafe extern "C" fn new_direct_byte_buffer(env: *mut JNIEnv, address: *mut c_void, capacity: jlong) -> jobject{
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let res = match DirectByteBuffer::new(jvm, int_state, address as jlong, match capacity.try_into() {
        Ok(x) => x,
        Err(_) => todo!("big byte buffers?"),
    }) {
        Ok(x) => x,
        Err(_) => todo!(),
    };
    new_local_ref_public_new(Some(res.full_object_ref()),int_state)
}




