use std::alloc::Layout;
use std::collections::HashMap;
use std::ffi::{c_void, CStr};
use std::mem::{size_of, transmute};
use std::os::raw::c_char;
use std::sync::Arc;

use jvmti_jni_bindings::{jboolean, jchar, JNI_TRUE, JNIEnv, jsize, jstring};

use crate::instructions::ldc::create_string_on_stack;
use crate::java::lang::string::JString;
use crate::java_values::{JavaValue, Object};
use crate::rust_jni::interface::local_frame::new_local_ref_public;
use crate::rust_jni::native_util::{from_object, get_interpreter_state, get_state, to_object};

//todo shouldn't this be handled by a registered native
pub unsafe extern "C" fn get_string_utfchars(_env: *mut JNIEnv,
                                             name: jstring,
                                             is_copy: *mut jboolean) -> *const c_char {
    //todo this could be replaced with string_obj_to_string, though prob wise to have some kind of streaming impl or something
    let str_obj_o = from_object(name).unwrap();//todo handle npe
    let possibly_uninit = str_obj_o.lookup_field("value").unwrap_object();
    let char_array: Vec<JavaValue> = match possibly_uninit {
        None => {
            "<invalid string>".chars().map(|c| JavaValue::Char(c as u16)).collect::<Vec<JavaValue>>()
        }
        Some(string_chars_o) => {
            let unwrapped = string_chars_o.unwrap_array().mut_array();
            unwrapped.clone()
        }
    };
    let chars_layout = Layout::from_size_align((char_array.len() + 1) * size_of::<c_char>(), size_of::<c_char>()).unwrap();
    let res = std::alloc::alloc(chars_layout) as *mut c_char;
    char_array.iter().enumerate().for_each(|(i, j)| {
        let cur = j.unwrap_char() as u8;
        res.add(i).write(transmute(cur))
    });
    res.add(char_array.len()).write(0);//null terminate
    if !is_copy.is_null() {
        is_copy.write(JNI_TRUE as u8);
    }
    // dbg!(get_state(_env).get_current_thread());
    res
}

pub unsafe extern "C" fn release_string_chars(env: *mut JNIEnv, _str: jstring, chars: *const jchar) {
    let jvm = get_state(env);
    jvm.native_interface_allocations.free(chars as *mut c_void);
}


pub unsafe extern "C" fn new_string_utf(env: *mut JNIEnv, utf: *const ::std::os::raw::c_char) -> jstring {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let str = CStr::from_ptr(utf);
    // dbg!(int_state.current_frame().local_vars());
    // dbg!(int_state.current_frame().operand_stack());
    new_local_ref_public(JString::from_rust(jvm, int_state, str.to_str().unwrap().to_string()).object().into(), int_state)
}

pub unsafe fn new_string_with_len(env: *mut JNIEnv, utf: *const ::std::os::raw::c_char, len: usize) -> jstring {
    let mut owned_str = String::with_capacity(len);
    for i in 0..len {
        owned_str.push(utf.add(i).read() as u8 as char);
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
    new_local_ref_public(string, int_state)
}


pub static mut STRING_INTERNMENT: Option<HashMap<Vec<u16>, Arc<Object>>> = None;

pub unsafe fn intern_impl(str_unsafe: jstring) -> jstring {
    //todo fix this entire function
    match &STRING_INTERNMENT {
        None => { STRING_INTERNMENT = Some(HashMap::new()) }
        Some(_) => {}
    };
    let str_obj = from_object(str_unsafe);
    let char_array_ptr = str_obj.clone().unwrap().lookup_field("value").unwrap_object().unwrap();//todo handle npe
    let char_array = char_array_ptr.unwrap_array().mut_array();
    let mut native_string_bytes = Vec::with_capacity(char_array.len());
    for char_ in &*char_array {
        native_string_bytes.push(char_.unwrap_char());
    }
    if STRING_INTERNMENT.as_ref().unwrap().contains_key(&native_string_bytes) {
        let res = STRING_INTERNMENT.as_ref().unwrap().get(&native_string_bytes).unwrap().clone();
        to_object(res.into())
    } else {
        STRING_INTERNMENT.as_mut().unwrap().insert(native_string_bytes, str_obj.as_ref().unwrap().clone());
        to_object(str_obj)
    }
}


pub unsafe extern "C" fn get_string_utflength(_env: *mut JNIEnv, str: jstring) -> jsize {
    let str_obj = from_object(str).unwrap();//todo handle npe
    //todo use length function.
    let char_object = str_obj.lookup_field("value").unwrap_object().unwrap();//todo handle npe
    let chars = char_object.unwrap_array();
    let borrowed_elems = chars.mut_array();
    borrowed_elems.len() as i32
}


pub unsafe extern "C" fn get_string_utfregion(_env: *mut JNIEnv, str: jstring, start: jsize, len: jsize, buf: *mut ::std::os::raw::c_char) {
    let str_obj = from_object(str).unwrap();//todo handle npe
    //todo maybe use string_obj_to_string in future.
    let char_object = str_obj.lookup_field("value").unwrap_object().unwrap();//todo handle npe
    let chars = char_object.unwrap_array();
    let borrowed_elems = chars.mut_array();
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
    let temp = from_object(str).unwrap().lookup_field("value").unwrap_object().unwrap();//todo handle npe
    let char_array = &temp.unwrap_array().mut_array();
    let mut str_ = Vec::new();
    for char_ in char_array.iter() {
        str_.push(char_.unwrap_char())
    }
    for i in 0..len {
        buf.offset(i as isize).write(str_[(start + i) as usize] as jchar);
    }
}


pub unsafe extern "C" fn release_string_utfchars(_env: *mut JNIEnv, _str: jstring, chars: *const c_char) {
    let len = libc::strlen(chars);
    let chars_layout = Layout::from_size_align((len + 1) * size_of::<c_char>(), size_of::<c_char>()).unwrap();
    std::alloc::dealloc(chars as *mut u8, chars_layout);
}
