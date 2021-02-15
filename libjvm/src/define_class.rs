use std::ffi::CStr;
use std::fs::File;
use std::io::{Cursor, Write};
use std::sync::Arc;

use classfile_parser::parse_class_file;
use classfile_view::view::ClassView;
use jvmti_jni_bindings::{jbyte, jclass, JNIEnv, jobject, jsize};
use slow_interpreter::java_values::JavaValue;
use slow_interpreter::rust_jni::native_util::{from_object, get_interpreter_state, get_state, to_object};

use crate::java_sun_misc_unsafe::defineAnonymousClass::define_class;

#[no_mangle]
unsafe extern "system" fn JVM_DefineClass(env: *mut JNIEnv, name: *const ::std::os::raw::c_char, loader: jobject, buf: *const jbyte, len: jsize, pd: jobject) -> jclass {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_DefineClassWithSource(env: *mut JNIEnv, name: *const ::std::os::raw::c_char, loader: jobject, buf: *const jbyte, len: jsize, pd: jobject, source: *const ::std::os::raw::c_char) -> jclass {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let name_string = CStr::from_ptr(name).to_str().unwrap();
    let loader_name = JavaValue::Object(from_object(loader)).cast_class_loader().to_jvm_loader(jvm);
    let slice = std::slice::from_raw_parts(buf as *const u8, len as usize);
    File::create("withsource").unwrap().write_all(slice).unwrap();
    let parsed = Arc::new(parse_class_file(&mut Cursor::new(slice)).expect("todo handle invalid"));
    to_object(define_class(jvm, int_state, parsed.clone(), loader_name, ClassView::from(parsed)).unwrap_object())
}
