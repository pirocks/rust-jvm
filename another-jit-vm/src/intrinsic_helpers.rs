use std::ffi::c_void;
use std::ptr::null_mut;

use memoffset::offset_of;

use crate::JITContext;

extern "C" fn fremf(a: f32, b: f32) -> f32{
    a % b
}

extern "C" fn dremd(a: f64, b: f64) -> f64{
    a % b
}

#[repr(C)]
pub struct IntrinsicHelpers {
    memmove: *const c_void,
    fremf: *const c_void,
    dremd: *const c_void,
    //todo move over instance of to this
    instanceof_helper: *const c_void,
    malloc: *const c_void,
    free: *const c_void,
}

impl IntrinsicHelpers {
    pub fn new() -> IntrinsicHelpers {
        IntrinsicHelpers {
            memmove: libc::memmove as *const c_void,
            fremf: fremf as *const c_void,
            dremd: dremd as *const c_void,
            instanceof_helper: null_mut(),
            malloc: libc::malloc as *const c_void,
            free: libc::free as *const c_void
        }
    }
}

#[derive(Debug, Clone)]
pub enum IntrinsicHelperType {
    Memmove,
    FRemF,
    DRemD,
    InstanceOf,
    Malloc,
    Free,
}

impl IntrinsicHelperType {
    pub const fn r15_offset(&self) -> usize {
        offset_of!(JITContext,intrinsic_helpers) + match self {
            IntrinsicHelperType::Memmove => {
                offset_of!(IntrinsicHelpers,memmove)
            }
            IntrinsicHelperType::FRemF => {
                offset_of!(IntrinsicHelpers,fremf)
            }
            IntrinsicHelperType::InstanceOf => {
                todo!()
            }
            IntrinsicHelperType::DRemD => {
                offset_of!(IntrinsicHelpers,dremd)
            }
            IntrinsicHelperType::Malloc => {
                offset_of!(IntrinsicHelpers,malloc)
            }
            IntrinsicHelperType::Free => {
                offset_of!(IntrinsicHelpers,free)
            }
        }
    }
}