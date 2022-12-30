use std::num::NonZeroUsize;
use std::ptr::{copy_nonoverlapping, NonNull, null_mut};

use libc::c_void;
use nonnull_const::{NonNullConst, NonNullMut};

use another_jit_vm::intrinsic_helpers::ExtraIntrinsicHelpers;
use gc_memory_layout_common::memory_regions::{ConstantRegionHeaderWrapper, MemoryRegions, RegionHeader, VariableRegionHeaderWrapper};
use jvmti_jni_bindings::jobject;

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

unsafe extern "C" fn clone_fast(obj_to_clone: jobject) -> jobject {
    let obj_to_clone = obj_to_clone as *mut c_void;
    let region_header_ptr = NonNullConst::new(MemoryRegions::find_object_region_header_raw(NonNull::new(obj_to_clone).unwrap())).unwrap();
    let region_header = MemoryRegions::find_object_region_header(NonNull::new(obj_to_clone).unwrap());
    let (allocation, size) = if region_header.is_array {
        //todo use layout
        let elem_size = region_header.array_elem_size.unwrap().get();
        let elem_0_offset = region_header.array_elem0_offset;
        let len_offset = region_header.array_len_offset;
        let len = obj_to_clone.cast::<i32>().offset(len_offset as isize).read() as usize;
        let to_clone_size = NonZeroUsize::new(len * elem_size + elem_0_offset).unwrap();
        (match VariableRegionHeaderWrapper::get_allocation(region_header_ptr, to_clone_size) {
            Some(x) => x,
            None => {
                return null_mut();
            }
        }, to_clone_size)
    } else {
        let size = region_header.region_elem_size.unwrap();
        (match ConstantRegionHeaderWrapper::get_allocation(region_header_ptr) {
            Some(x) => x,
            None => { return null_mut(); }
        }, size)
    };
    copy_nonoverlapping(obj_to_clone, allocation.as_ptr(), size.get());
    allocation.as_ptr() as jobject
}


pub fn extra_intrinsics(this_thread_obj_raw: NonNullMut<c_void>) -> ExtraIntrinsicHelpers {
    ExtraIntrinsicHelpers {
        constant_size_allocation: constant_size_allocation as *const c_void,
        current_thread_obj: this_thread_obj_raw,
        find_vtable_ptr: find_vtable_ptr as *const c_void,
        find_itable_ptr: find_itable_ptr as *const c_void,
        find_class_ptr: find_class_ptr as *const c_void,
        find_object_region_header: find_object_region_header as *const c_void,
        clone_fast: clone_fast as *const c_void,
    }
}
