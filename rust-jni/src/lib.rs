extern crate libloading;
extern crate jni;
extern crate libc;

use libloading::Library;
use libloading::Symbol;
use std::sync::Arc;
use rust_jvm_common::unified_types::ParsedType;
use runtime_common::runtime_class::RuntimeClass;
use runtime_common::java_values::JavaValue;
use jni::sys::*;
use std::ffi::c_void;

pub mod mangling;


pub trait JNIContext {
    fn call(&self, classfile: Arc<RuntimeClass>, method_i: usize, args: Vec<JavaValue>, return_type: ParsedType) -> JavaValue;
}

pub struct LibJavaLoading {
    pub lib: Library
}

impl JNIContext for LibJavaLoading {
    fn call(&self, classfile: Arc<RuntimeClass>, method_i: usize, args: Vec<JavaValue>, return_type: ParsedType) -> JavaValue {
        let mangled = mangling::mangle(classfile, method_i);
        unsafe {
            match return_type {
                /*ParsedType::ByteType => {
                    let symbol: Symbol<unsafe extern fn(env: *const jni::JNIEnv, ...) -> jbyte> = self.lib.get(mangled.as_bytes()).unwrap();
                }
                ParsedType::CharType => {
                    let symbol: Symbol<unsafe extern fn(env: *const jni::JNIEnv, ...) -> jchar> = self.lib.get(mangled.as_bytes()).unwrap();
                }
                ParsedType::DoubleType => {
                    let symbol: Symbol<unsafe extern fn(env: *const jni::JNIEnv, ...) -> jdouble> = self.lib.get(mangled.as_bytes()).unwrap();
                }
                ParsedType::FloatType => {
                    let symbol: Symbol<unsafe extern fn(env: *const jni::JNIEnv, ...) -> jfloat> = self.lib.get(mangled.as_bytes()).unwrap();
                }
                ParsedType::IntType => {
                    let symbol: Symbol<unsafe extern fn(env: *const jni::JNIEnv, ...) -> jint> = self.lib.get(mangled.as_bytes()).unwrap();
                }
                ParsedType::LongType => {
                    let symbol: Symbol<unsafe extern fn(env: *const jni::JNIEnv, ...) -> jlong> = self.lib.get(mangled.as_bytes()).unwrap();
                }
                ParsedType::ShortType => {
                    let symbol: Symbol<unsafe extern fn(env: *const jni::JNIEnv, ...) -> jshort> = self.lib.get(mangled.as_bytes()).unwrap();
                }
                ParsedType::BooleanType => {
                    let symbol: Symbol<unsafe extern fn(env: *const jni::JNIEnv, ...) -> jboolean> = self.lib.get(mangled.as_bytes()).unwrap();
                }
                ParsedType::Class(_) => {
                    let symbol: Symbol<unsafe extern fn(env: *const jni::JNIEnv, ...) -> jobject> = self.lib.get(mangled.as_bytes()).unwrap();
                }
                ParsedType::ArrayReferenceType(_) => {
                    //todo handle multiple array type
                    let symbol: Symbol<unsafe extern fn(env: *const jni::JNIEnv, ...) -> jarray> = self.lib.get(mangled.as_bytes()).unwrap();
                }*/
                ParsedType::VoidType => {
                    let symbol: Symbol<unsafe extern fn(env: *const jni::JNIEnv, ...) -> c_void > = self.lib.get(mangled.as_bytes()).unwrap();

                }
                _ => {}
            }
        }


        unimplemented!()
    }
}

//#![allow(non_upper_case_globals)]
//#![allow(non_camel_case_types)]
//#![allow(non_snake_case)]
//
//include!(concat!(env!("OUT_DIR"), "/bindings.rs"));


//#[no_mangle]
//pub extern "C" struct JENV() {
////    ...
//}