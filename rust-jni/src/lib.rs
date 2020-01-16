extern crate libloading;
extern crate jni;
extern crate libc;

use libloading::Library;
use libloading::Symbol;
use std::sync::Arc;
use rust_jvm_common::unified_types::ParsedType;
use runtime_common::runtime_class::RuntimeClass;
use runtime_common::java_values::JavaValue;
use jni::sys::jdouble;

pub mod mangling {
    use std::sync::Arc;
    use runtime_common::runtime_class::RuntimeClass;

    pub fn mangle(classfile: Arc<RuntimeClass>, method_i: usize) -> String {
        unimplemented!()
    }
}


pub trait JNIContext {
    fn call(&self, classfile: Arc<RuntimeClass>, method_i: usize, args: Vec<JavaValue>, return_type: ParsedType)-> JavaValue;
}

pub struct LibJavaLoading {
    pub lib: Library
}

impl JNIContext for LibJavaLoading {
    fn call(&self, classfile: Arc<RuntimeClass>, method_i: usize, args: Vec<JavaValue>, return_type: ParsedType) -> JavaValue{
        let mangled = mangling::mangle(classfile,method_i);
        unsafe {
            let symbol: Symbol<unsafe extern fn(env: *const jni::JNIEnv, ...) -> jdouble> = self.lib.get(mangled.as_bytes()).unwrap();
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