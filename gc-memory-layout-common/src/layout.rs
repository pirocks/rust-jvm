use std::ffi::c_void;
use std::mem::size_of;
use std::num::NonZeroUsize;
use std::ptr::NonNull;

use memoffset::offset_of;

use another_jit_vm::FramePointerOffset;
use jvmti_jni_bindings::{jint, jlong};
use rust_jvm_common::compressed_classfile::compressed_types::CPDType;


pub struct ArrayMemoryLayout {
    sub_type: CPDType,
}

enum ArrayAlign{
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
    pub fn from_cpdtype(sub_type: CPDType) -> Self {
        Self {
            sub_type
        }
    }
    /*pub fn from_unknown_cpdtype() -> Self {
        Self {
            sub_type: todo!()
        }
    }*/

    fn subtype_align(&self) -> ArrayAlign{
        match self.sub_type{
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

    pub fn calculate_index_address(&self, array_pointer: NonNull<c_void>, index: i32) -> NonNull<c_void>{
        assert!(index >= 0);
        unsafe { NonNull::new(array_pointer.as_ptr().offset(self.elem_0_entry_offset() as isize).offset(index as isize * self.elem_size() as isize)).unwrap() }
    }

    pub fn calculate_len_address(&self, array_pointer: NonNull<c_void>) -> NonNull<i32>{
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
        match self.subtype_align(){
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

//todo frane info will need to be reworked to be based of rip
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct FrameHeader {
    pub prev_rip: *mut c_void,
    pub prev_rpb: *mut c_void,
    pub ir_method_id: u64,
    pub methodid: usize,
    pub magic_part_1: u64,
    pub magic_part_2: u64,
}


pub const FRAME_HEADER_PREV_RIP_OFFSET: usize = offset_of!(FrameHeader,prev_rip);
pub const FRAME_HEADER_PREV_RBP_OFFSET: usize = offset_of!(FrameHeader, prev_rpb);
pub const FRAME_HEADER_IR_METHOD_ID_OFFSET: usize = offset_of!(FrameHeader,ir_method_id);
pub const FRAME_HEADER_METHOD_ID_OFFSET: usize = offset_of!(FrameHeader,methodid);
pub const FRAME_HEADER_PREV_MAGIC_1_OFFSET: usize = offset_of!(FrameHeader,magic_part_1);
pub const FRAME_HEADER_PREV_MAGIC_2_OFFSET: usize = offset_of!(FrameHeader,magic_part_2);
pub const FRAME_HEADER_END_OFFSET: usize = size_of::<FrameHeader>();

const MAX_OPERAND_STACK_NEEDED_FOR_FUNCTION_INVOCATION: usize = 256 * size_of::<jlong>();


pub struct NativeStackframeMemoryLayout {
    pub num_locals: u16,// num_locals does include top native functions, to allow same ircall mechanism
}

impl NativeStackframeMemoryLayout {
    pub fn local_var_entry(&self, i: u16) -> FramePointerOffset {
        assert!(i < self.num_locals as u16);
        FramePointerOffset(size_of::<FrameHeader>() + i as usize * size_of::<jlong>())
    }

    pub fn data_entry(&self) -> FramePointerOffset {
        FramePointerOffset(size_of::<FrameHeader>() + self.num_locals as usize * size_of::<jlong>())
    }

    pub fn full_frame_size(&self) -> usize {
        size_of::<FrameHeader>() + self.num_locals as usize * size_of::<jlong>() + size_of::<jlong>() //extra jlong for extra native data entry
    }
}



