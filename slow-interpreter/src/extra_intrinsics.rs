use std::ptr::{NonNull, null_mut};

use libc::c_void;
use nonnull_const::{NonNullConst, NonNullMut};
use another_jit_vm::intrinsic_helpers::ExtraIntrinsicHelpers;
use gc_memory_layout_common::memory_regions::{ConstantRegionHeaderWrapper, MemoryRegions, RegionHeader};

unsafe extern "C" fn constant_size_allocation(region_header: *const RegionHeader) -> Option<NonNull<c_void>> {
    ConstantRegionHeaderWrapper::get_allocation(NonNullConst::new(region_header)?)
}

pub extern "C" fn find_vtable_ptr(ptr_in: *mut c_void) -> *mut c_void {
    MemoryRegions::find_type_vtable(match NonNull::new(ptr_in) {
        Some(x) => x,
        None => return null_mut(),
    }).map(|inner| inner.as_ptr() as *mut c_void).unwrap_or(null_mut())
}

pub extern "C" fn find_itable_ptr(ptr_in: *mut c_void) -> *mut c_void {
    MemoryRegions::find_type_itable(match NonNull::new(ptr_in) {
        Some(x) => x,
        None => return null_mut(),
    }).map(|inner| inner.as_ptr() as *mut c_void).unwrap_or(null_mut())
}

pub extern "C" fn find_class_ptr(ptr_in: *mut c_void) -> *mut c_void {
    MemoryRegions::find_class_ptr_cache(match NonNull::new(ptr_in) {
        Some(x) => x,
        None => return null_mut(),
    }).map(|inner| inner.as_ptr() as *mut c_void).unwrap_or(null_mut())
}

pub extern "C" fn find_object_region_header(ptr_in: *mut c_void) -> *mut c_void {
    MemoryRegions::find_object_region_header_raw(NonNull::new(ptr_in).unwrap()) as *mut c_void
}

pub fn extra_intrinsics(this_thread_obj_raw: NonNullMut<c_void>) -> ExtraIntrinsicHelpers {
    ExtraIntrinsicHelpers {
        constant_size_allocation: constant_size_allocation as *const c_void,
        current_thread_obj: this_thread_obj_raw,
        find_vtable_ptr: find_vtable_ptr as *const c_void,
        find_itable_ptr: find_itable_ptr as *const c_void,
        find_class_ptr: find_class_ptr as *const c_void,
        find_object_region_header: find_object_region_header as *const c_void,
    }
}
