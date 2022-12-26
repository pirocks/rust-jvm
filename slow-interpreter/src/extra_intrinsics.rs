use std::ptr::NonNull;
use libc::c_void;
use nonnull_const::{NonNullConst, NonNullMut};
use another_jit_vm::intrinsic_helpers::ExtraIntrinsicHelpers;
use gc_memory_layout_common::memory_regions::{ConstantRegionHeaderWrapper, RegionHeader};

unsafe extern "C" fn constant_size_allocation(region_header: *const RegionHeader) -> Option<NonNull<c_void>> {
    ConstantRegionHeaderWrapper::get_allocation(NonNullConst::new(region_header)?)
}

pub fn extra_intrinsics(this_thread_obj_raw: NonNullMut<c_void>) -> ExtraIntrinsicHelpers{
    ExtraIntrinsicHelpers{
        constant_size_allocation: constant_size_allocation as *const c_void,
        current_thread_obj: this_thread_obj_raw,
    }
}
