use std::ptr::NonNull;
use itertools::Itertools;


use crate::StaticFieldNumber;

pub struct RawStaticFields {
    len: usize,
    ptr: *mut u64,
}

impl<'gc> RawStaticFields {
    pub fn new(len: usize) -> Self {
        let mut native_jv_vec = (0..len).map(|_| 0u64).collect_vec();
        native_jv_vec.shrink_to_fit();
        let (ptr, len, cap) = native_jv_vec.into_raw_parts();
        assert_eq!(len, cap);
        Self {
            len,
            ptr,
        }
    }

    pub fn raw_ptr(&self) -> *mut u64 {
        self.ptr
    }

    pub fn get(&self, static_number: StaticFieldNumber) -> NonNull<u64> {
        assert!((static_number.0 as usize) < self.len);
        NonNull::new(unsafe { self.raw_ptr().offset(static_number.0 as isize) }).unwrap()
    }
}
