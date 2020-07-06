use jvmti_jni_bindings::{JNIEnv, jobject};

use crate::rust_jni::native_util::from_object;

pub unsafe extern "C" fn new_local_ref(_env: *mut JNIEnv, ref_: jobject) -> jobject {
    //todo blocking on actually having gc
    std::mem::forget(from_object(ref_).unwrap());
    ref_
}
