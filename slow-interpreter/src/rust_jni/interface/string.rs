use std::alloc::Layout;
use std::cell::Ref;
use std::collections::HashMap;
use std::mem::{size_of, transmute};
use std::os::raw::c_char;
use std::sync::Arc;

use jvmti_jni_bindings::{jboolean, jchar, JNI_TRUE, JNIEnv, jsize, jstring};

use crate::instructions::ldc::create_string_on_stack;
use crate::java_values::{JavaValue, Object};
use crate::rust_jni::native_util::{from_object, get_interpreter_state, get_state, to_object};

//todo shouldn't this be handled by a registered native
pub unsafe extern "C" fn get_string_utfchars(_env: *mut JNIEnv,
                                             name: jstring,
                                             is_copy: *mut jboolean) -> *const c_char {
    //todo this could be replaced with string_obj_to_string, though prob wise to have some kind of streaming impl or something
    let str_obj_o = from_object(name).unwrap();
    let string_chars_o = str_obj_o.lookup_field("value").unwrap_object().unwrap();
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
        is_copy.write(JNI_TRUE as u8);
    }
    // dbg!(get_state(_env).get_current_thread());
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
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    // let frame = int_state.current_frame_mut();
    create_string_on_stack(jvm, int_state, owned_str);
    let string = int_state.pop_current_operand_stack().unwrap_object();
    assert!(!string.is_none());
    to_object(string)
}


pub static mut STRING_INTERNMENT: Option<HashMap<String, Arc<Object>>> = None;

pub unsafe extern "system" fn intern_impl(str_unsafe: jstring) -> jstring {
    match &STRING_INTERNMENT {
        None => { STRING_INTERNMENT = Some(HashMap::new()) }
        Some(_) => {}
    };
    let str_obj = from_object(str_unsafe);
    let char_array_ptr = str_obj.clone().unwrap().lookup_field("value").unwrap_object().unwrap();
    let char_array = char_array_ptr.unwrap_array().elems.borrow();
    let mut native_string = String::with_capacity(char_array.len());
    for char_ in &*char_array {
        native_string.push(char_.unwrap_char() as u8 as char);
    }
    if STRING_INTERNMENT.as_ref().unwrap().contains_key(&native_string) {
        let res = STRING_INTERNMENT.as_ref().unwrap().get(&native_string).unwrap().clone();
        to_object(res.into())
    } else {
        STRING_INTERNMENT.as_mut().unwrap().insert(native_string, str_obj.as_ref().unwrap().clone());
        to_object(str_obj)
    }
}


pub unsafe extern "C" fn get_string_utflength(_env: *mut JNIEnv, str: jstring) -> jsize {
    let str_obj = from_object(str).unwrap();
    //todo use length function.
    let char_object = str_obj.lookup_field("value").unwrap_object().unwrap();
    let chars = char_object.unwrap_array();
    let borrowed_elems = chars.elems.borrow();
    borrowed_elems.len() as i32
}


pub unsafe extern "C" fn get_string_utfregion(_env: *mut JNIEnv, str: jstring, start: jsize, len: jsize, buf: *mut ::std::os::raw::c_char) {
    let str_obj = from_object(str).unwrap();
    //todo maybe use string_obj_to_string in future.
    let char_object = str_obj.lookup_field("value").unwrap_object().unwrap();
    let chars = char_object.unwrap_array();
    let borrowed_elems = chars.elems.borrow();
    for i in 0..len {
        let char_ = (&borrowed_elems[(start + i) as usize]).unwrap_char() as i8 as u8 as char;
        buf.offset(i as isize).write(char_ as i8);
    }
    buf.offset((len) as isize).write('\0' as i8);
}


pub unsafe extern "C" fn new_string(env: *mut JNIEnv, unicode: *const jchar, len: jsize) -> jstring {
    let mut str = String::with_capacity(len as usize);
    for i in 0..len {
        str.push(unicode.offset(i as isize).read() as u8 as char)
    }
    let res = new_string_with_string(env, str);
    assert_ne!(res, std::ptr::null_mut());
    res
}

pub unsafe extern "C" fn get_string_region(_env: *mut JNIEnv, str: jstring, start: jsize, len: jsize, buf: *mut jchar) {
    let temp = from_object(str).unwrap().lookup_field("value").unwrap_object().unwrap();
    let char_array = &temp.unwrap_array().elems.borrow();
    let mut str_ = Vec::new();
    for char_ in char_array.iter() {
        str_.push(char_.unwrap_char())
    }
    for i in 0..len {
        buf.offset(i as isize).write(str_[(start + i) as usize] as jchar);
    }
}

