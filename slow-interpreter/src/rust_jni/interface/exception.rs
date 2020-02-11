use jni_bindings::{jthrowable, JNIEnv};

pub unsafe extern "C" fn exception_occured(_env: *mut JNIEnv) -> jthrowable {
    //exceptions don't happen yet todo
    std::ptr::null_mut()
}


