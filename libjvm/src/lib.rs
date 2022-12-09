#![feature(box_syntax)]
#![feature(core_intrinsics)]
#![feature(once_cell)]
//#![feature(asm)]
#![allow(non_snake_case)]
#![allow(unused)]

extern crate libc;
extern crate nix;
extern crate num_cpus;
extern crate regex;

use std::borrow::Borrow;
use std::cell::RefCell;
use std::cmp::min;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::ffi::{c_void, CStr};
use std::intrinsics::transmute;
use std::ops::Deref;
use std::os::raw::{c_char, c_int};
use std::str::from_utf8;
use std::thread::Thread;
use libc::{size_t, strcpy};
use nix::NixPath;

use jvmti_jni_bindings::{__va_list_tag, FILE, getc, JavaVM, jboolean, jbyte, jbyteArray, jclass, jdouble, jfloat, jint, jintArray, jlong, jmethodID, JNI_VERSION_1_8, JNIEnv, jobject, jobjectArray, jsize, jstring, jvalue, JVM_CALLER_DEPTH, JVM_ExceptionTableEntryType, jvm_version_info, sockaddr, vsnprintf};
use rust_jvm_common::classfile::{ACC_INTERFACE, ACC_PUBLIC};
use rust_jvm_common::classnames::{class_name, ClassName};
use rust_jvm_common::ptype::PType;
use slow_interpreter::better_java_stack::frames::HasFrame;
use slow_interpreter::interpreter::common::ldc::{create_string_on_stack, load_class_constant_by_type};
use slow_interpreter::interpreter_util::{new_object, run_constructor};
use slow_interpreter::rust_jni::jni_utils::{get_interpreter_state, get_state};
use slow_interpreter::rust_jni::native_util::{from_object, to_object};
use slow_interpreter::rust_jni::value_conversion::{native_to_runtime_class, runtime_class_to_native};

use crate::introspection::JVM_GetCallerClass;

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
unsafe extern "system" fn JVM_GetTemporaryDirectory(env: *mut JNIEnv) -> jstring {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ReleaseUTF(utf: *const c_char) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetLastErrorString(buf: *mut c_char, len: c_int) -> jint {
    let error_string= CStr::from_ptr(libc::strerror(nix::errno::errno()));
    let output_len = min(error_string.len(), (len - 1) as usize);
    libc::strncpy(buf, error_string.as_ptr(), output_len as size_t);
    buf.offset(output_len as isize).write(0);
    output_len as jint
}

#[no_mangle]
unsafe extern "system" fn JVM_CopySwapMemory(env: *mut JNIEnv, srcObj: jobject, srcOffset: jlong, dstObj: jobject, dstOffset: jlong, size: jlong, elemSize: jlong) {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    int_state.debug_print_stack_trace(jvm);
    srcObj.cast::<c_void>().offset(srcOffset as isize);
    dstObj.cast::<c_void>().offset(dstOffset as isize);
    todo!()
}

#[no_mangle]
unsafe extern "system" fn JVM_KnownToNotExist(env: *mut JNIEnv, loader: jobject, classname: *const c_char) -> jboolean {
    unimplemented!()
}

pub mod real_main;