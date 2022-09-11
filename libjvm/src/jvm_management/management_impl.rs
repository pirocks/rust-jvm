use std::ptr::null_mut;


use jvmti_jni_bindings::{jint, jlong};
use slow_interpreter::rust_jni::jni_interface::jni::{get_interpreter_state, get_state};
use slow_interpreter::rust_jni::jni_interface::local_frame::new_local_ref_public_new;

