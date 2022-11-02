use std::ffi::c_void;
use std::mem::size_of;
use std::num::NonZeroUsize;
use std::ptr::NonNull;

use jvmti_jni_bindings::{jint, jlong};
use rust_jvm_common::compressed_classfile::compressed_types::CPDType;
use crate::accessor::Accessor;

pub struct ArrayMemoryLayout {
    sub_type: CPDType,
}

enum ArrayAlign {
    Byte,
    X86Word,
    X86DWord,
    X86QWord,
}

// #[repr(C)]
// pub union ArrayNativeJV{
//     pub bool: u8,
//     pub byte: i8,
//     pub char: u16,
//     pub short: i16,
//     pub int: i32,
//     pub float: f32,
//     pub long: i64,
//     pub double: f64,
//     pub obj: *mut c_void
// }

impl ArrayMemoryLayout {
    pub const fn from_cpdtype(sub_type: CPDType) -> Self {
        Self {
            sub_type
        }
    }
    /*pub fn from_unknown_cpdtype() -> Self {
        Self {
            sub_type: todo!()
        }
    }*/

    fn subtype_align(&self) -> ArrayAlign {
        match self.sub_type {
            CPDType::BooleanType => {
                ArrayAlign::Byte
            }
            CPDType::ByteType => {
                ArrayAlign::Byte
            }
            CPDType::ShortType => {
                ArrayAlign::X86Word
            }
            CPDType::CharType => {
                ArrayAlign::X86Word
            }
            CPDType::IntType => {
                ArrayAlign::X86DWord
            }
            CPDType::LongType => {
                ArrayAlign::X86QWord
            }
            CPDType::FloatType => {
                ArrayAlign::X86DWord
            }
            CPDType::DoubleType => {
                ArrayAlign::X86QWord
            }
            CPDType::VoidType => {
                todo!("?")
            }
            CPDType::Class(_) => {
                ArrayAlign::X86QWord
            }
            CPDType::Array { .. } => {
                ArrayAlign::X86QWord
            }
        }
    }

    pub fn calculate_index_address(&self, array_pointer: NonNull<c_void>, index: i32) -> ArrayAccessor {
        assert!(index >= 0);
        let inner_ptr = unsafe { NonNull::new(array_pointer.as_ptr().offset(self.elem_0_entry_offset() as isize).offset(index as isize * self.elem_size() as isize)).unwrap() };
        ArrayAccessor {
            expected_type: self.sub_type,
            inner: inner_ptr,
        }
    }

    pub fn calculate_len_address(&self, array_pointer: NonNull<c_void>) -> NonNull<i32> {
        unsafe { NonNull::new(array_pointer.as_ptr().offset(self.len_entry_offset() as isize)).unwrap().cast::<i32>() }
    }

    pub fn elem_0_entry_offset(&self) -> usize {
        match self.subtype_align() {
            ArrayAlign::Byte => {
                size_of::<jint>()
            }
            ArrayAlign::X86Word => {
                size_of::<jint>()
            }
            ArrayAlign::X86DWord => {
                size_of::<jint>()
            }
            ArrayAlign::X86QWord => {
                size_of::<jlong>()
            }
        }
    }
    pub fn len_entry_offset(&self) -> usize {
        0
    }
    pub fn elem_size(&self) -> usize {
        match self.subtype_align() {
            ArrayAlign::Byte => {
                1
            }
            ArrayAlign::X86Word => {
                2
            }
            ArrayAlign::X86DWord => {
                4
            }
            ArrayAlign::X86QWord => {
                8
            }
        }
    }
    pub fn array_size(&self, len: jint) -> NonZeroUsize {
        NonZeroUsize::new(self.elem_0_entry_offset() + len as usize * self.elem_size()).unwrap()
    }
}

#[derive(Copy, Clone)]
pub struct ArrayAccessor {
    expected_type: CPDType,
    inner: NonNull<c_void>,
}

impl Accessor for ArrayAccessor {
    fn expected_type(&self) -> CPDType {
        self.expected_type
    }

    fn read_impl<T>(&self) -> T {
        unsafe { self.inner.cast::<T>().as_ptr().read() }
    }


    fn write_impl<T>(&self, to_write: T) {
        unsafe { self.inner.cast::<T>().as_ptr().write(to_write) }
    }
}

