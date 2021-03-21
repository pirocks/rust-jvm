use std::ffi::{c_void, CStr};
use std::iter::once;
use std::os::raw::c_char;
use std::ptr::null_mut;
use std::sync::Arc;

use jvmti_jni_bindings::{jboolean, jchar, JNI_TRUE, JNIEnv, jobject, jsize, jstring};
use sketch_jvm_version_of_utf8::JVMString;

use crate::instructions::ldc::create_string_on_stack;
use crate::interpreter::WasException;
use crate::interpreter_state::InterpreterStateGuard;
use crate::java::lang::string::JString;
use crate::java_values::{ExceptionReturn, JavaValue, Object};
use crate::jvm_state::JVMState;
use crate::rust_jni::interface::local_frame::new_local_ref_public;
use crate::rust_jni::native_util::{from_object, get_interpreter_state, get_state, to_object};
use crate::utils::{throw_npe, throw_npe_res};

pub unsafe extern "C" fn get_string_utfchars(env: *mut JNIEnv,
                                             str: jstring,
                                             is_copy: *mut jboolean) -> *const c_char {
    get_rust_str(env, str, |rust_str| {
        let mut buf = JVMString::from_regular_string(rust_str.as_str()).buf.clone();
        buf.push(0);//null terminator
        let jvm = get_state(env);
        let mut res = null_mut();
        jvm.native_interface_allocations.allocate_and_write_vec(buf, null_mut(), &mut res as *mut *mut u8);
        if !is_copy.is_null() {
            is_copy.write(JNI_TRUE as u8);
        }
        res as *const c_char
    })
}

pub unsafe extern "C" fn release_string_chars(env: *mut JNIEnv, _str: jstring, chars: *const jchar) {
    let jvm = get_state(env);
    jvm.native_interface_allocations.free(chars as *mut c_void);
}


pub unsafe extern "C" fn new_string_utf(env: *mut JNIEnv, utf: *const ::std::os::raw::c_char) -> jstring {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let str = CStr::from_ptr(utf);
    new_local_ref_public(match JString::from_rust(jvm, int_state, str.to_str().unwrap().to_string()) {
        Ok(jstring) => jstring,
        Err(WasException {}) => return null_mut()
    }.object().into(), int_state)
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
    if let Err(WasException {}) = create_string_on_stack(jvm, int_state, owned_str) {
        return null_mut();
    };
    let string = int_state.pop_current_operand_stack().unwrap_object();
    assert!(!string.is_none());
    new_local_ref_public(string, int_state)
}


pub unsafe fn intern_impl_unsafe(jvm: &JVMState, int_state: &mut InterpreterStateGuard, str_unsafe: jstring) -> Result<jstring, WasException> {
    let str_obj = match from_object(str_unsafe) {
        Some(x) => x,
        None => return throw_npe_res(jvm, int_state),
    };
    Ok(to_object(intern_safe(jvm, str_obj).object().into()))
}

pub fn intern_safe(jvm: &JVMState, str_obj: Arc<Object>) -> JString {
    let char_array_ptr = match str_obj.clone().lookup_field("value").unwrap_object() {
        None => {
            eprintln!("Weird malformed string encountered. Not interning.");
            return JavaValue::Object(str_obj.into()).cast_string().unwrap();//fallback to not interning weird strings like this. not sure if compatible with hotspot but idk what else to do. perhaps throwing an exception would be better idk?
        }
        Some(char_array_ptr) => char_array_ptr
    };
    let char_array = char_array_ptr.unwrap_array().mut_array();
    let mut native_string_bytes = Vec::with_capacity(char_array.len());
    for char_ in &*char_array {
        native_string_bytes.push(char_.unwrap_char());
    }
    let mut guard = jvm.string_internment.write().unwrap();
    match guard.strings.get(&native_string_bytes) {
        None => {
            guard.strings.insert(native_string_bytes, str_obj.clone());
            JavaValue::Object(str_obj.into()).cast_string().unwrap()
        }
        Some(res) => {
            JavaValue::Object(res.clone().into()).cast_string().unwrap()
        }
    }
}


pub unsafe extern "C" fn get_string_utflength(env: *mut JNIEnv, str: jstring) -> jsize {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);

    let str_obj = match from_object(str) {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state);
        }
    };
    let jstring = JavaValue::Object(str_obj.into()).cast_string().unwrap();
    let rust_str = jstring.to_rust_string();
    JVMString::from_regular_string(rust_str.as_str()).buf.len() as i32
}


pub unsafe extern "C" fn get_string_utfregion(env: *mut JNIEnv, str: jstring, start: jsize, len: jsize, buf: *mut ::std::os::raw::c_char) {
    get_rust_str(env, str, |rust_str| {
        let chars = rust_str.chars().skip(start as usize).take(len as usize);
        let new_str = chars.collect::<String>();
        if new_str.chars().count() != len as usize || rust_str.chars().count() < start as usize {
            todo!("string out of bounds exception");
        }
        let sketch_string = JVMString::from_regular_string(new_str.as_str());
        for (i, val) in sketch_string.buf.iter().chain(once(&0u8)).enumerate() {
            buf.offset(i as isize).write(*val as i8);
        }
    });
}

unsafe fn get_rust_str<T: ExceptionReturn>(env: *mut JNIEnv, str: jobject, and_then: impl Fn(String) -> T) -> T {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let str_obj = match from_object(str) {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state);
        }
    };
    let rust_str = JavaValue::Object(str_obj.into()).cast_string().unwrap().to_rust_string();
    and_then(rust_str)
}


pub unsafe extern "C" fn new_string(env: *mut JNIEnv, unicode: *const jchar, len: jsize) -> jstring {
    let mut str = String::with_capacity(len as usize);
    for i in 0..len {
        str.push(unicode.offset(i as isize).read() as u8 as char)
    }
    new_string_with_string(env, str)
}

pub unsafe extern "C" fn get_string_region(env: *mut JNIEnv, str: jstring, start: jsize, len: jsize, buf: *mut jchar) {
    get_rust_str(env, str, |rust_str| {
        for (i, char) in rust_str.chars().skip(start as usize).take(len as usize).enumerate() {//todo bounds check
            buf.offset(i as isize).write(char as jchar);
        }
    })
}


pub unsafe extern "C" fn release_string_utfchars(env: *mut JNIEnv, _str: jstring, chars: *const c_char) {
    let jvm = get_state(env);
    jvm.native_interface_allocations.free(chars as *mut c_void)
}
