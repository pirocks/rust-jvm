#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused)]

include!(concat!("../gen", "/bindings.rs"));

unsafe impl Send for JNIInvokeInterface_ {}

unsafe impl Sync for JNIInvokeInterface_ {}

pub trait JavaPrimitiveType {}

impl JavaPrimitiveType for jobject {}

impl JavaPrimitiveType for jboolean {}

impl JavaPrimitiveType for jbyte {}

impl JavaPrimitiveType for jchar {}

impl JavaPrimitiveType for jshort {}

impl JavaPrimitiveType for jint {}

impl JavaPrimitiveType for jlong {}

impl JavaPrimitiveType for jfloat {}

impl JavaPrimitiveType for jdouble {}