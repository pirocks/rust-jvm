use std::ffi::CStr;
use std::ptr::null_mut;

use itertools::Itertools;
use wtf8::Wtf8Buf;

use jvmti_jni_bindings::{JNIEnv, jobjectArray, jstring};
use slow_interpreter::interpreter::WasException;
use slow_interpreter::java::lang::string::JString;
use slow_interpreter::java_values::JavaValue;
use slow_interpreter::native_allocation::AllocationType::CString;
use slow_interpreter::rust_jni::interface::local_frame::new_local_ref_public;
use slow_interpreter::rust_jni::native_util::{from_object, get_interpreter_state, get_state};

#[no_mangle]
unsafe extern "system" fn JVM_GetSystemPackage(env: *mut JNIEnv, name: jstring) -> jstring {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let class_name_string = JavaValue::Object(from_object(jvm, name)).cast_string().unwrap().to_rust_string(jvm);
    dbg!(&class_name_string);
    let mut elements = class_name_string.split(|char_| char_ == '.' || char_ == '/').collect_vec();
    elements.pop();
    let res_string = elements.iter().join(".");
    dbg!(&res_string);
    let jstring = match JString::from_rust(jvm, int_state, Wtf8Buf::from_string(res_string)) {
        Ok(jstring) => jstring,
        Err(WasException {}) => {
            return null_mut()
        }
    };
    new_local_ref_public(jstring.object().into(), int_state)
}

#[no_mangle]
unsafe extern "system" fn JVM_GetSystemPackages(env: *mut JNIEnv) -> jobjectArray {
    unimplemented!()
}
