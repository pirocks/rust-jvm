#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused)]

include!(concat!("../gen", "/bindings.rs"));
pub const ACC_SYNCHRONIZED: u16 = 0x0020; //todo but why do I have to define this? shouldn't it be in bindings?

unsafe impl Send for JNIInvokeInterface_{}
unsafe impl Sync for JNIInvokeInterface_{}