extern crate libloading;
extern crate jni;
extern crate libc;

use libloading::Library;
use libloading::Symbol;
use std::sync::Arc;
use rust_jvm_common::unified_types::ParsedType;

pub mod mangling {
    use std::sync::Arc;
    use crate::runtime_class::RuntimeClass;

    pub fn mangle(classfile: Arc<RuntimeClass>, method_i: usize) -> String {
        unimplemented!()
    }
}


pub trait JNIContext {
    fn call(&self, classfile: Arc<RuntimeClass>, method_i: usize, args: List<JavaValue>, return_type: ParsedType);
}

pub struct LibJavaLoading {
    pub lib: Library
}

impl JNIContext for LibJavaLoading {
    fn call(&self, classfile: Arc<RuntimeClass>, method_i: usize, args: List<JavaValue>, return_type: ParsedType) -> JavaValue{
        unsafe {
            let symbol: Symbol<unsafe extern fn(env, *jni::JNIEnv, ...) -> jdouble> = self.lib.get(mangled.as_bytes()).unwrap();
        }
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