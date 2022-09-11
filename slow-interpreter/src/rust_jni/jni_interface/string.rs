use std::ffi::{c_void, CStr};
use std::iter::once;
use std::os::raw::c_char;
use std::ptr::null_mut;

use wtf8::{CodePoint, Wtf8Buf};

use jvmti_jni_bindings::{jboolean, jchar, JNI_TRUE, JNIEnv, jobject, jsize, jstring};
use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName};
use sketch_jvm_version_of_utf8::JVMString;

use crate::class_loading::assert_loaded_class;

use crate::stdlib::java::lang::string::JString;
use crate::java_values::{ExceptionReturn, JavaValue};
use crate::jvm_state::JVMState;
use crate::new_java_values::{NewJavaValueHandle};
use crate::{AllocatedHandle, JavaValueCommon, NewAsObjectOrJavaValue, PushableFrame, WasException};
use crate::rust_jni::jni_interface::{get_interpreter_state, get_state};
use crate::rust_jni::jni_interface::local_frame::{new_local_ref_public_new};
use crate::rust_jni::native_util::{from_object_new, to_object_new};
use crate::utils::{throw_npe, throw_npe_res};

pub unsafe extern "C" fn get_string_utfchars(env: *mut JNIEnv, str: jstring, is_copy: *mut jboolean) -> *const c_char {
    get_rust_str(env, str, |rust_str| {
        let mut buf = JVMString::from_regular_string(rust_str.as_str()).buf.clone();
        buf.push(0); //null terminator
        let jvm = get_state(env);
        let mut res = null_mut();
        jvm.native.native_interface_allocations.allocate_and_write_vec(buf, null_mut(), &mut res as *mut *mut u8);
        if !is_copy.is_null() {
            is_copy.write(JNI_TRUE as u8);
        }
        res as *const c_char
    })
}

pub unsafe extern "C" fn release_string_chars(env: *mut JNIEnv, _str: jstring, chars: *const jchar) {
    let jvm = get_state(env);
    jvm.native.native_interface_allocations.free(chars as *mut c_void);
}

pub unsafe extern "C" fn new_string_utf(env: *mut JNIEnv, utf: *const c_char) -> jstring {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let str = CStr::from_ptr(utf);
    let res = new_local_ref_public_new(
        match JString::from_rust(jvm, int_state, Wtf8Buf::from_string(str.to_str().unwrap().to_string())) {
            Ok(jstring) => jstring,
            Err(WasException { exception_obj }) => {
                todo!();
                return null_mut();
            }
        }.intern(jvm, int_state).unwrap()
            .object().as_allocated_obj()
            .into(),
        int_state
    );
    res
}

pub unsafe fn new_string_with_len(env: *mut JNIEnv, utf: *const c_char, len: usize) -> jstring {
    let mut owned_str = Wtf8Buf::with_capacity(len);
    for i in 0..len {
        //todo this is probably wrong
        owned_str.push(CodePoint::from_char(utf.add(i).read() as u8 as char));
    }
    new_string_with_string(env, owned_str)
}

pub unsafe fn new_string_with_string(env: *mut JNIEnv, owned_str: Wtf8Buf) -> jstring {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    match JString::from_rust(jvm, int_state, owned_str).unwrap().intern(jvm, int_state) {
        Err(WasException { exception_obj }) => {
            todo!();
            null_mut()
        }
        Ok(res) => {
            new_local_ref_public_new(res.new_java_value_handle().as_njv().unwrap_object_alloc(), int_state)
        }
    }
}

pub unsafe fn intern_impl_unsafe<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, str_unsafe: jstring) -> Result<jstring, WasException<'gc>> {
    let str_obj = match from_object_new(jvm, str_unsafe) {
        Some(x) => x,
        None => return throw_npe_res(jvm, int_state),
    };
    Ok(to_object_new(intern_safe(jvm, str_obj).object().as_allocated_obj().into()))//todo should this be local ref?
}

pub fn intern_safe<'gc>(jvm: &'gc JVMState<'gc>, str_obj: AllocatedHandle<'gc>) -> JString<'gc> {
    let string_class = assert_loaded_class(jvm, CClassName::string().into());
    let char_array_ptr = match str_obj.unwrap_normal_object_ref().get_var(jvm, &string_class, FieldName::field_value()).unwrap_object() {
        None => {
            eprintln!("Weird malformed string encountered. Not interning.");
            return JavaValue::Object(todo!() /*str_obj.into()*/).cast_string().unwrap();
            //fallback to not interning weird strings like this. not sure if compatible with hotspot but idk what else to do. perhaps throwing an exception would be better idk?
        }
        Some(char_array_ptr) => char_array_ptr,
    };
    let char_array = char_array_ptr.unwrap_array();
    let mut native_string_bytes = Vec::with_capacity(char_array.len() as usize);
    for char_ in char_array.array_iterator() {
        native_string_bytes.push(char_.as_njv().unwrap_char_strict());
    }
    let mut guard = jvm.string_internment.write().unwrap();
    match guard.strings.get(&native_string_bytes) {
        None => {
            guard.strings.insert(native_string_bytes, str_obj.duplicate_discouraged());
            NewJavaValueHandle::Object(str_obj.into()).cast_string().unwrap()
        }
        Some(res) => NewJavaValueHandle::Object(res.duplicate_discouraged()).cast_string().unwrap(),
    }
}

pub unsafe extern "C" fn get_string_utflength(env: *mut JNIEnv, str: jstring) -> jsize {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);

    let str_obj = match from_object_new(jvm, str) {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state);
        }
    };
    let jstring = NewJavaValueHandle::Object(str_obj.into()).cast_string().unwrap();
    let rust_str = jstring.to_rust_string(jvm);
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
    let str_obj = match from_object_new(jvm, str) {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state);
        }
    };
    let rust_str = NewJavaValueHandle::Object(str_obj).cast_string().unwrap().to_rust_string(jvm);
    and_then(rust_str)
}

pub unsafe extern "C" fn new_string(env: *mut JNIEnv, unicode: *const jchar, len: jsize) -> jstring {
    let mut str = Wtf8Buf::with_capacity(len as usize);
    for i in 0..len {
        str.push(CodePoint::from_char(unicode.offset(i as isize).read() as u8 as char)) // todo handle unicode properly.
    }
    new_string_with_string(env, str)
}

pub unsafe extern "C" fn get_string_region(env: *mut JNIEnv, str: jstring, start: jsize, len: jsize, buf: *mut jchar) {
    get_rust_str(env, str, |rust_str| {
        for (i, char) in rust_str.chars().skip(start as usize).take(len as usize).enumerate() {
            //todo bounds check
            buf.offset(i as isize).write(char as jchar);
        }
    })
}

pub unsafe extern "C" fn release_string_utfchars(env: *mut JNIEnv, _str: jstring, chars: *const c_char) {
    let jvm = get_state(env);
    jvm.native.native_interface_allocations.free(chars as *mut c_void)
}