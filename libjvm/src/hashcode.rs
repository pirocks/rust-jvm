use jvmti_jni_bindings::{jint, JNIEnv, jobject};

use slow_interpreter::rust_jni::native_util::{from_object_new};
use slow_interpreter::rust_jni::jni_utils::{get_state};

#[no_mangle]
unsafe extern "system" fn JVM_IHashCode(env: *mut JNIEnv, obj: jobject) -> jint {
    let jvm = get_state(env);
    let object = from_object_new(jvm, obj);
    let _64bit: u64 = match object {
        Some(x) => x,
        None => return 0,
    }.as_allocated_obj().raw_ptr_usize() as u64;
    let hashcode = (_64bit >> 32) as u32 ^ _64bit as u32;
    hashcode as jint
    //don't change without also changing intrinsics setup.
}