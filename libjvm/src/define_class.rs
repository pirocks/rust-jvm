use std::ffi::CStr;
use std::fs::File;
use std::io::{Cursor, Write};
use std::ptr::null_mut;
use std::sync::Arc;

use classfile_parser::parse_class_file;
use classfile_view::view::ClassBackedView;
use jvmti_jni_bindings::{jbyte, jclass, JNIEnv, jobject, jsize};
use slow_interpreter::interpreter::WasException;
use slow_interpreter::java_values::JavaValue;
use slow_interpreter::rust_jni::interface::define_class_safe;
use slow_interpreter::rust_jni::native_util::{from_object, get_interpreter_state, get_state, to_object};

#[no_mangle]
unsafe extern "system" fn JVM_DefineClass(env: *mut JNIEnv, name: *const ::std::os::raw::c_char, loader: jobject, buf: *const jbyte, len: jsize, pd: jobject) -> jclass {
    JVM_DefineClassWithSource(env, name, loader, buf, len, pd, null_mut())
}

//todo handle source
//todo what is pd
#[no_mangle]
unsafe extern "system" fn JVM_DefineClassWithSource(env: *mut JNIEnv, name: *const ::std::os::raw::c_char, loader: jobject, buf: *const jbyte, len: jsize, _pd: jobject, _source: *const ::std::os::raw::c_char) -> jclass {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let name_string = CStr::from_ptr(name).to_str().unwrap(); //todo handle bad utf8, with to lossy or something
    let loader_name = JavaValue::Object(from_object(jvm, loader)).cast_class_loader().to_jvm_loader(jvm);
    let slice = std::slice::from_raw_parts(buf as *const u8, len as usize);
    if jvm.store_generated_classes { File::create("withsource").unwrap().write_all(slice).unwrap(); }
    let parsed = Arc::new(parse_class_file(&mut Cursor::new(slice)).expect("todo handle invalid"));
    to_object(match define_class_safe(jvm, int_state, parsed.clone(), loader_name, ClassBackedView::from(parsed, &jvm.string_pool)) {
        Ok(res) => res,
        Err(_) => todo!()
    }.unwrap_object())
}
