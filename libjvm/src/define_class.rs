use std::ffi::{c_char, CStr};
use std::fs::File;
use std::io::{Cursor, Write};
use std::ptr::null_mut;
use std::sync::Arc;

use classfile_parser::parse_class_file;
use classfile_view::view::ClassBackedView;
use jni_interface::define_class_safe;
use jvmti_jni_bindings::{jbyte, jclass, JNIEnv, jobject, jsize};
use rust_jvm_common::loading::LoaderName;
use slow_interpreter::better_java_stack::frames::HasFrame;
use slow_interpreter::exceptions::WasException;
use slow_interpreter::java_values::JavaValue;
use slow_interpreter::new_java_values::allocated_objects::AllocatedHandle;


use slow_interpreter::rust_jni::jni_utils::{get_interpreter_state, get_state};
use slow_interpreter::rust_jni::native_util::{from_object, from_object_new, to_object, to_object_new};

#[no_mangle]
unsafe extern "system" fn JVM_DefineClass(env: *mut JNIEnv, name: *const ::std::os::raw::c_char, loader: jobject, buf: *const jbyte, len: jsize, pd: jobject) -> jclass {
    JVM_DefineClassWithSource(env, name, loader, buf, len, pd, null_mut())
}

//todo handle source
//todo what is pd
#[no_mangle]
unsafe extern "system" fn JVM_DefineClassWithSource(env: *mut JNIEnv, name: *const c_char, loader: jobject, buf: *const jbyte, len: jsize, _pd: jobject, _source: *const c_char) -> jclass {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let name_string = CStr::from_ptr(name).to_str().unwrap(); //todo handle bad utf8, with to lossy or something
    let loader_name = match from_object_new(jvm, loader) {
        None => LoaderName::BootstrapLoader,
        Some(loader_obj) => loader_obj.cast_class_loader().to_jvm_loader(jvm)
    };
    let slice = std::slice::from_raw_parts(buf as *const u8, len as usize);
    if jvm.config.store_generated_classes {
        File::create("withsource.class").unwrap().write_all(slice).unwrap();
    }
    let parsed = Arc::new(match parse_class_file(&mut Cursor::new(slice)) {
        Ok(x) => x,
        Err(err) => {
            int_state.debug_print_stack_trace(jvm);
            dbg!(err);
            todo!()
        },
    });
    to_object_new(
        match define_class_safe(jvm, int_state, parsed.clone(), loader_name, ClassBackedView::from(parsed, &jvm.string_pool)) {
            Ok(res) => res,
            Err(WasException { exception_obj }) => {
                todo!();
                return null_mut();
            }
        }
            .unwrap_object().unwrap().as_allocated_obj().into(),
    )
}