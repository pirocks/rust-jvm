#![feature(core_intrinsics)]
#![allow(non_snake_case)]
#![allow(unreachable_code)]
extern crate libc;
extern crate nix;
extern crate num_cpus;
extern crate regex;

use std::cmp::min;
use std::ffi::{c_void, CStr};
use std::os::raw::{c_char, c_int};
use libc::{size_t};
use nix::NixPath;

use jvmti_jni_bindings::{jboolean, jint, jlong, JNIEnv, jobject, jstring};
use slow_interpreter::better_java_stack::frames::HasFrame;
use slow_interpreter::rust_jni::jni_utils::{get_interpreter_state, get_state};


//so in theory I need something like this:
//    asm!(".symver JVM_GetEnclosingMethodInfo JVM_GetEnclosingMethodInfo@@SUNWprivate_1.1");
//but in reality I don't?

pub mod access_control;
pub mod arrays;
pub mod assertion;
pub mod clone;
pub mod compiler;
pub mod define_class;
pub mod dtrace;
pub mod find_class;
pub mod gc;
pub mod get_resource;
pub mod hashcode;
pub mod intern;
pub mod introspection;
pub mod io;
pub mod jio;
pub mod jvm_management;
pub mod library;
pub mod loading;
pub mod memory;
pub mod monitor;
pub mod packages;
pub mod properties;
pub mod raw_monitor;
pub mod reflection;
pub mod resolve_class;
pub mod signals;
pub mod socket;
pub mod stacktrace;
pub mod thread;
pub mod time;
pub mod trace;
pub mod util;
pub mod sun_reflect_reflection;
pub mod ensure_deps_used;

#[no_mangle]
unsafe extern "system" fn JVM_GetTemporaryDirectory(_env: *mut JNIEnv) -> jstring {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ReleaseUTF(_utf: *const c_char) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetLastErrorString(buf: *mut c_char, len: c_int) -> jint {
    let error_string= CStr::from_ptr(libc::strerror(nix::errno::errno()));
    let output_len = min(error_string.len(), (len - 1) as usize);
    libc::strncpy(buf, error_string.as_ptr(), output_len as size_t);
    buf.add(output_len).write(0);
    output_len as jint
}

#[no_mangle]
unsafe extern "system" fn JVM_CopySwapMemory(env: *mut JNIEnv, srcObj: jobject, srcOffset: jlong, dstObj: jobject, dstOffset: jlong, _size: jlong, _elemSize: jlong) {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    int_state.debug_print_stack_trace(jvm);
    let _ = srcObj.cast::<c_void>().offset(srcOffset as isize);
    let _ = dstObj.cast::<c_void>().offset(dstOffset as isize);
    todo!()
}

#[no_mangle]
unsafe extern "system" fn JVM_KnownToNotExist(_env: *mut JNIEnv, _loader: jobject, _classname: *const c_char) -> jboolean {
    unimplemented!()
}

pub mod real_main;