use jni_bindings::{JNIEnv, jstring, jboolean, jchar};
use std::os::raw::c_char;
use runtime_common::java_values::{JavaValue, Object};
use std::cell::{Ref, RefCell};
use std::sync::Arc;
use std::alloc::Layout;
use std::mem::{size_of, transmute};
use crate::rust_jni::native_util::{from_object, get_state, get_frame, to_object};
use crate::instructions::ldc::create_string_on_stack;

//todo shouldn't this be handled by a registered native
pub unsafe extern "C" fn get_string_utfchars(_env: *mut JNIEnv,
                                         name: jstring,
                                         is_copy: *mut jboolean) -> *const c_char {
    let str_obj: Arc<Object> = from_object(name).unwrap();
    let unwrapped = str_obj.fields.borrow().get("value").unwrap().clone().unwrap_array();
    let refcell: &RefCell<Vec<JavaValue>> = &unwrapped;
    let char_array: &Ref<Vec<JavaValue>> = &refcell.borrow();
    let chars_layout = Layout::from_size_align((char_array.len() + 1) * size_of::<c_char>(), size_of::<c_char>()).unwrap();
    let res = std::alloc::alloc(chars_layout) as *mut c_char;
    char_array.iter().enumerate().for_each(|(i, j)| {
        let cur = j.unwrap_char() as u8;
        res.offset(i as isize).write(transmute(cur))
    });
    res.offset(char_array.len() as isize).write(0);//null terminate
    if is_copy != std::ptr::null_mut() {
        unimplemented!()
    }
    return res;
}

pub unsafe extern "C" fn release_string_chars(_env: *mut JNIEnv, _str: jstring, _chars: *const jchar) {
    unimplemented!()
}


pub unsafe extern "C" fn new_string_utf(env: *mut JNIEnv, utf: *const ::std::os::raw::c_char) -> jstring {
    let len = libc::strlen(utf);
    let mut owned_str = String::with_capacity(len);
    for i in 0..len {
        owned_str.push(utf.offset(i as isize).read() as u8 as char);
    }
    let state = get_state(env);
    let frame = get_frame(env);
    create_string_on_stack(state,&frame,owned_str);
    let string = frame.pop().unwrap_object();
    to_object(string.into())
}