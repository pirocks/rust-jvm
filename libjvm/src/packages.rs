use std::ffi::CStr;
use std::ptr::null_mut;

use itertools::Itertools;
use wtf8::Wtf8Buf;

use jvmti_jni_bindings::{JNIEnv, jobjectArray, jstring};
use slow_interpreter::exceptions::WasException;
use slow_interpreter::java::lang::string::JString;
use slow_interpreter::java::NewAsObjectOrJavaValue;
use slow_interpreter::java_values::JavaValue;
use slow_interpreter::native_allocation::AllocationType::CString;
use slow_interpreter::new_java_values::NewJavaValueHandle;
use slow_interpreter::rust_jni::interface::jni::{get_interpreter_state, get_state};
use slow_interpreter::rust_jni::interface::local_frame::{new_local_ref_public, new_local_ref_public_new};
use slow_interpreter::rust_jni::native_util::{from_object, from_object_new};
use slow_interpreter::utils::pushable_frame_todo;

#[no_mangle]
unsafe extern "system" fn JVM_GetSystemPackage(env: *mut JNIEnv, name: jstring) -> jstring {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let class_name_string = NewJavaValueHandle::Object(from_object_new(jvm, name).unwrap()).cast_string().unwrap().to_rust_string(jvm);
    let mut elements = class_name_string.split(|char_| char_ == '.' || char_ == '/').collect_vec();
    elements.pop();
    let res_string = elements.iter().join(".");
    let jstring = match JString::from_rust(jvm, int_state, Wtf8Buf::from_string(res_string)) {
        Ok(jstring) => jstring,
        Err(WasException { exception_obj }) => {
            todo!();
            return null_mut();
        }
    };
    new_local_ref_public_new(jstring.full_object().as_allocated_obj().into(), int_state)
}

#[no_mangle]
unsafe extern "system" fn JVM_GetSystemPackages(env: *mut JNIEnv) -> jobjectArray {
    unimplemented!()
}