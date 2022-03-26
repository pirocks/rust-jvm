#![allow(dead_code)]
#![allow(unused_variables)]
#![feature(int_roundings)]
#![feature(box_syntax)]
#![feature(exclusive_range_pattern)]

use std::ffi::c_void;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;

#[derive(Copy, Clone)]
pub union NativeJavaValue<'gc> {
    pub byte: i8,
    pub boolean: u8,
    pub short: i16,
    pub char: u16,
    pub int: i32,
    pub long: i64,
    pub float: f32,
    pub double: f64,
    pub object: *mut c_void,
    phantom_data: PhantomData<&'gc ()>,
    pub as_u64: u64,
}

impl Debug for NativeJavaValue<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unsafe { write!(f, "NativeJavaValue({:?})", self.object) }
    }
}


pub mod layout;
pub mod memory_regions;