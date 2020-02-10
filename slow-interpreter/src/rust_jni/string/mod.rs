use jni_bindings::{JNIEnv, jstring, jboolean, jchar};
use std::os::raw::c_char;
use runtime_common::java_values::{JavaValue, Object};
use std::cell::Ref;
use std::alloc::Layout;
use std::mem::{size_of, transmute};
use crate::rust_jni::native_util::{from_object, get_state, get_frame, to_object};
use crate::instructions::ldc::create_string_on_stack;
use std::collections::HashMap;
use std::sync::Arc;

//todo shouldn't this be handled by a registered native
pub unsafe extern "C" fn get_string_utfchars(_env: *mut JNIEnv,
                                             name: jstring,
                                             is_copy: *mut jboolean) -> *const c_char {
    let str_obj_o = from_object(name).unwrap();
    let str_obj= str_obj_o.unwrap_normal_object();
    let string_chars_o = str_obj.fields.borrow().get("value").unwrap().clone().unwrap_object().unwrap();
    let unwrapped = string_chars_o.unwrap_array().elems.borrow();
    let char_array: &Ref<Vec<JavaValue>> = &unwrapped;
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
    new_string_with_len(env, utf, len)
}

pub unsafe fn new_string_with_len(env: *mut JNIEnv, utf: *const ::std::os::raw::c_char, len: usize) -> jstring {
    let mut owned_str = String::with_capacity(len);
    for i in 0..len {
        owned_str.push(utf.offset(i as isize).read() as u8 as char);
    }
    new_string_with_string(env, owned_str)
}

pub unsafe fn new_string_with_string(env: *mut JNIEnv, owned_str: String) -> jstring {
    let state = get_state(env);
    let frame = get_frame(env);
    create_string_on_stack(state, &frame, owned_str);
    let string = frame.pop().unwrap_object();
    assert!(!string.is_none());
    to_object(string)
}


pub static mut STRING_INTERNMENT_CAMP: Option<HashMap<String, Arc<Object>>> = None;

pub unsafe extern "system" fn intern_impl(str_unsafe: jstring) -> jstring{
    match &STRING_INTERNMENT_CAMP {
        None => { STRING_INTERNMENT_CAMP = Some(HashMap::new()) }
        Some(_) => {}
    };
    let str_obj = from_object(str_unsafe);
    let char_array_ptr = str_obj.clone().unwrap().unwrap_normal_object().fields.borrow().get("value").unwrap().unwrap_object().unwrap();
    let char_array = char_array_ptr.unwrap_array().elems.borrow();
    let mut native_string = String::with_capacity(char_array.len());
    for char_ in &*char_array {
        native_string.push(char_.unwrap_char());
    }
    if STRING_INTERNMENT_CAMP.as_ref().unwrap().contains_key(&native_string) {
        let res = STRING_INTERNMENT_CAMP.as_ref().unwrap().get(&native_string).unwrap().clone();
        to_object(res.into())
    } else {
        STRING_INTERNMENT_CAMP.as_mut().unwrap().insert(native_string, str_obj.as_ref().unwrap().clone());
        to_object(str_obj)
    }
}
