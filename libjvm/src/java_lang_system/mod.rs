use std::borrow::Borrow;

use jvmti_jni_bindings::{jint, JNIEnv, jobject};
use slow_interpreter::rust_jni::native_util::{from_object, get_interpreter_state, get_state};
use slow_interpreter::utils::throw_npe;

#[no_mangle]
unsafe extern "system" fn Java_lang_system_arraycopy(
    env: *mut JNIEnv,
    src: jobject,
    srcpos: jint,
    dst: jobject,
    destpos: jint,
    length: jint,
) {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let src_o = from_object(src);
    let src = match src_o.as_ref() {
        Some(x) => x,
        None => return throw_npe(jvm, int_state),
    }.unwrap_array();
    let dest_o = from_object(dst);
    let dest = match dest_o.as_ref() {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state)
        },
    }.unwrap_array();
    if srcpos < 0
        || destpos < 0
        || length < 0
        || srcpos + length > src.mut_array().len() as i32
        || destpos + length > dest.mut_array().len() as i32 {
        unimplemented!()
    }
    let mut to_copy = vec![];
    for i in 0..(length as usize) {
        let borrowed = src.mut_array();
        let temp = (borrowed)[srcpos as usize + i].borrow().clone();
        to_copy.push(temp);
    }
    for i in 0..(length as usize) {
        let borrowed = dest.mut_array();
        borrowed[destpos as usize + i] = to_copy[i].clone();
    }
}
