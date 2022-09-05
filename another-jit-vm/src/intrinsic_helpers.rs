use std::ffi::c_void;
use memoffset::offset_of;
use crate::JITContext;

#[repr(C)]
pub struct IntrinsicHelpers {
    memmove: *const c_void,
    //todo move over instance of to this
    instanceof_helper: ()
}

impl IntrinsicHelpers{
    pub fn new() -> IntrinsicHelpers{
        IntrinsicHelpers{
            memmove: libc::memmove as *const c_void,
            instanceof_helper: ()
        }
    }
}

#[derive(Debug, Clone)]
pub enum IntrinsicHelperType{
    Memmove,
    InstanceOf
}

impl IntrinsicHelperType {
    pub const fn r15_offset(&self) -> usize{
        offset_of!(JITContext,intrinsic_helpers) + match self {
            IntrinsicHelperType::Memmove => {
                offset_of!(IntrinsicHelpers,memmove)
            }
            IntrinsicHelperType::InstanceOf => {
                todo!()
            }
        }
    }
}