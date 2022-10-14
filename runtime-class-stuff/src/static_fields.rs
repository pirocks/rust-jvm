use std::ptr::NonNull;
use itertools::Itertools;

use rust_jvm_common::NativeJavaValue;

use crate::StaticFieldNumber;

pub struct RawStaticFields<'gc> {
    len: usize,
    ptr: *mut NativeJavaValue<'gc>,
}

impl<'gc> RawStaticFields<'gc> {
    pub fn new(len: usize) -> Self {
        let mut native_jv_vec = (0..len).map(|_| NativeJavaValue { as_u64: 0 }).collect_vec();
        native_jv_vec.shrink_to_fit();
        let (ptr, len, cap) = native_jv_vec.into_raw_parts();
        assert_eq!(len, cap);
        Self {
            len,
            ptr,
        }
    }

    pub fn raw_ptr(&self) -> *mut NativeJavaValue<'gc> {
        self.ptr
    }

    pub fn get(&self, static_number: StaticFieldNumber) -> NonNull<NativeJavaValue<'gc>> {
        assert!((static_number.0 as usize) < self.len);
        NonNull::new(unsafe { self.raw_ptr().offset(static_number.0 as isize) }).unwrap()
    }
}
