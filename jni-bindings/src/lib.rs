#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused)]

include!(concat!("../gen", "/bindings.rs"));

unsafe impl Send for JNIInvokeInterface_{}
unsafe impl Sync for JNIInvokeInterface_{}

