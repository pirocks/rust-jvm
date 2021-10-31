#![feature(box_syntax)]
#![feature(entry_insert)]
#![feature(with_options)]
#![feature(in_band_lifetimes)]
//#![feature(asm)]

#![allow(non_snake_case)]
#![allow(unused)]

extern crate libc;
extern crate nix;
extern crate num_cpus;
extern crate regex;

use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::ffi::{c_void, CStr};
use std::intrinsics::transmute;
use std::ops::Deref;
use std::os::raw::{c_char, c_int};
use std::str::from_utf8;
use std::thread::Thread;

use jvmti_jni_bindings::{__va_list_tag, FILE, getc, JavaVM, jboolean, jbyte, jbyteArray, jclass, jdouble, jfloat, jint, jintArray, jlong, jmethodID, JNI_VERSION_1_8, JNIEnv, jobject, jobjectArray, jsize, jstring, jvalue, JVM_CALLER_DEPTH, JVM_ExceptionTableEntryType, jvm_version_info, sockaddr, vsnprintf};
use rust_jvm_common::classfile::{ACC_INTERFACE, ACC_PUBLIC};
use rust_jvm_common::classnames::{class_name, ClassName};
use rust_jvm_common::ptype::PType;
use slow_interpreter::instructions::ldc::{create_string_on_stack, load_class_constant_by_type};
use slow_interpreter::interpreter_util::{new_object, run_constructor};
use slow_interpreter::rust_jni::native_util::{from_object, get_state, to_object};
use slow_interpreter::rust_jni::value_conversion::{native_to_runtime_class, runtime_class_to_native};

use crate::introspection::JVM_GetCallerClass;

//so in theory I need something like this:
//    asm!(".symver JVM_GetEnclosingMethodInfo JVM_GetEnclosingMethodInfo@@SUNWprivate_1.1");
//but in reality I don't?

pub mod hashcode;
pub mod monitor;
pub mod time;
pub mod intern;
pub mod clone;
pub mod properties;
pub mod memory;
pub mod library;
pub mod stacktrace;
pub mod compiler;
pub mod thread;
pub mod arrays;
pub mod resolve_class;
pub mod find_class;
pub mod define_class;
pub mod get_resource;
pub mod jio;
pub mod loading;
pub mod packages;
pub mod gc;
pub mod trace;
pub mod jvm_management;
pub mod signals;
pub mod assertion;
pub mod introspection;
pub mod reflection;
pub mod access_control;
pub mod dtrace;
pub mod io;
pub mod socket;
pub mod raw_monitor;
pub mod java_sun_misc_unsafe;
pub mod util;

#[no_mangle]
unsafe extern "system" fn JVM_GetTemporaryDirectory(env: *mut JNIEnv) -> jstring {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ReleaseUTF(utf: *const ::std::os::raw::c_char) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetLastErrorString(buf: *mut ::std::os::raw::c_char, len: ::std::os::raw::c_int) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_CopySwapMemory(
    env: *mut JNIEnv,
    srcObj: jobject,
    srcOffset: jlong,
    dstObj: jobject,
    dstOffset: jlong,
    size: jlong,
    elemSize: jlong,
) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_KnownToNotExist(
    env: *mut JNIEnv,
    loader: jobject,
    classname: *const ::std::os::raw::c_char,
) -> jboolean {
    unimplemented!()
}


