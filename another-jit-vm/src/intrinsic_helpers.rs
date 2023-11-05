use std::ffi::c_void;
use std::ptr::null_mut;

use memoffset::offset_of;
use nonnull_const::{NonNullMut};

use crate::JITContext;

extern "C" fn fremf(a: f32, b: f32) -> f32 {
    a % b
}

extern "C" fn dremd(a: f64, b: f64) -> f64 {
    a % b
}

#[derive(Copy, Clone)]
pub struct ExtraIntrinsicHelpers {
    pub constant_size_allocation: *const c_void,
    pub current_thread_obj: NonNullMut<c_void>,
    pub find_vtable_ptr: *const c_void,
    pub find_itable_ptr: *const c_void,
    pub find_class_ptr: *const c_void,
    pub find_object_region_header: *const c_void,
    pub clone_fast: *const c_void,
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
    constant_size_allocation: *const c_void,
    find_vtable_ptr: *const c_void,
    find_itable_ptr: *const c_void,
    find_class_ptr: *const c_void,
    find_object_region_header: *const c_void,
    clone_fast: *const c_void,
}

impl IntrinsicHelpers {
    pub fn new(extra: &ExtraIntrinsicHelpers) -> IntrinsicHelpers {
        let ExtraIntrinsicHelpers {
            constant_size_allocation, current_thread_obj: _,
            find_vtable_ptr,
            find_itable_ptr,
            find_class_ptr,
            find_object_region_header, clone_fast
        } = *extra;
        IntrinsicHelpers {
            memmove: libc::memmove as *const c_void,
            fremf: fremf as *const c_void,
            dremd: dremd as *const c_void,
            instanceof_helper: null_mut(),
            malloc: libc::malloc as *const c_void,
            free: libc::free as *const c_void,
            constant_size_allocation,
            find_vtable_ptr,
            find_itable_ptr,
            find_class_ptr,
            find_object_region_header,
            clone_fast
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum IntrinsicHelperType {
    Memmove,
    FRemF,
    DRemD,
    InstanceOf,
    Malloc,
    Free,
    GetConstantAllocation,
    FindVTablePtr,
    FindITablePtr,
    FindClassPtr,
    FindObjectHeader,
    FastClone,

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
            IntrinsicHelperType::GetConstantAllocation => {
                offset_of!(IntrinsicHelpers,constant_size_allocation)
            }
            IntrinsicHelperType::FindVTablePtr => {
                offset_of!(IntrinsicHelpers,find_vtable_ptr)
            }
            IntrinsicHelperType::FindITablePtr => {
                offset_of!(IntrinsicHelpers,find_itable_ptr)
            }
            IntrinsicHelperType::FindClassPtr => {
                offset_of!(IntrinsicHelpers,find_class_ptr)
            }
            IntrinsicHelperType::FindObjectHeader => {
                offset_of!(IntrinsicHelpers,find_object_region_header)
            }
            IntrinsicHelperType::FastClone => {
                offset_of!(IntrinsicHelpers,clone_fast)
            }
        }
    }
}

#[repr(C)]
pub struct ThreadLocalIntrinsicHelpers {
    pub current_thread_obj: NonNullMut<c_void>,
}

impl ThreadLocalIntrinsicHelpers {
    pub fn new(extra: &ExtraIntrinsicHelpers) -> ThreadLocalIntrinsicHelpers {
        ThreadLocalIntrinsicHelpers {
            current_thread_obj: extra.current_thread_obj,
        }
    }
}
