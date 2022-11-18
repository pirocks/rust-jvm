use std::mem::transmute;

use jvmti_jni_bindings::{jint, JNIEnv, jobject};

use slow_interpreter::rust_jni::native_util::{from_object, from_object_new};
use slow_interpreter::utils::throw_npe;
use slow_interpreter::rust_jni::jni_utils::{get_interpreter_state, get_state, get_throw};

#[no_mangle]
unsafe extern "system" fn JVM_IHashCode(env: *mut JNIEnv, obj: jobject) -> jint {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let object = from_object_new(jvm, obj);
    if object.is_none() {
        return throw_npe(jvm, int_state,get_throw(env));
    }
    let _64bit: u64 = object.unwrap().as_allocated_obj().raw_ptr_usize() as u64;
    let hashcode = ((_64bit >> 32) as u32 ^ _64bit as u32);
    hashcode as jint
    //todo don't change without also changing intrinsics setup.
}