//#![feature(asm)]

#![allow(non_snake_case)]
#![allow(unused)]

extern crate log;
extern crate simple_logger;
extern crate regex;
extern crate num_cpus;

use std::str::from_utf8;
use std::borrow::Borrow;
use runtime_common::{InterpreterState, StackEntry};
use rust_jvm_common::classnames::{ClassName, class_name};
use slow_interpreter::{get_or_create_class_object, array_of_type_class};
use std::rc::Rc;
use std::intrinsics::transmute;
use slow_interpreter::rust_jni::native_util::{get_state, get_frame, to_object, from_object};
use jni_bindings::{JNIEnv, jclass, jstring, jobject, jlong, jint, jboolean, jobjectArray, jvalue, jbyte, jsize, jbyteArray, jfloat, jdouble, jmethodID, sockaddr, jintArray, jvm_version_info, getc, __va_list_tag, FILE, JVM_ExceptionTableEntryType, vsnprintf, JVM_CALLER_DEPTH, JavaVM, JNI_VERSION_1_8};
use log::trace;
use slow_interpreter::interpreter_util::{check_inited_class, push_new_object, run_function, run_constructor};
use slow_interpreter::instructions::ldc::{load_class_constant_by_name, create_string_on_stack};
use slow_interpreter::instructions::invoke::{invoke_virtual_method_i, invoke_special, actually_virtual};
use rust_jvm_common::unified_types::{PType};
use runtime_common::java_values::{JavaValue, Object, ArrayObject};
use slow_interpreter::rust_jni::value_conversion::{native_to_runtime_class, runtime_class_to_native};
use std::sync::Arc;
use std::cell::RefCell;
use runtime_common::runtime_class::RuntimeClass;
use std::thread::Thread;
use std::ffi::{CStr, c_void};
use std::ops::Deref;
use std::collections::HashMap;
use std::collections::hash_map::RandomState;
use rust_jvm_common::classfile::{ACC_INTERFACE, ACC_PUBLIC};
use crate::introspection::JVM_GetCallerClass;
use std::os::raw::{c_int, c_char};
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



//this ends required symbols
//The following symbols are not needed for linking

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_registerNatives(
    env: *mut JNIEnv,
    cb: jclass) -> () {
    //todo for now register nothing, register later as needed.
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_arrayBaseOffset(env: *mut JNIEnv,
                                                               obj: jobject,
                                                               cb: jclass) -> jint {
    -1//unimplemented but can't return nothing.
}


#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_arrayIndexScale(env: *mut JNIEnv,
                                                               obj: jobject,
                                                               cb: jclass) -> jint {
    2//todo unimplemented but can't return nothing, and need to return a power of 2
}


#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_addressSize(env: *mut JNIEnv,
                                                           obj: jobject) -> jint {
    64//officially speaking unimplemented but can't return nothing, and should maybe return something reasonable todo
}

#[no_mangle]
unsafe extern "system" fn Java_sun_reflect_Reflection_getCallerClass(env: *mut JNIEnv,
                                                                     cb: jclass) -> jclass
{
    return JVM_GetCallerClass(env, JVM_CALLER_DEPTH);
}
