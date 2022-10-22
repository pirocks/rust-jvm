use std::ptr::NonNull;
use libc::c_void;
use nonnull_const::NonNullConst;
use another_jit_vm::intrinsic_helpers::ExtraIntrinsicHelpers;
use gc_memory_layout_common::memory_regions::{ConstantRegionHeaderWrapper, RegionHeader};

unsafe extern "C" fn constant_size_allocation(region_header: NonNullConst<RegionHeader>) -> Option<NonNull<c_void>> {
    ConstantRegionHeaderWrapper::get_allocation(region_header)
}

pub fn extra_intrinsics() -> ExtraIntrinsicHelpers{
    ExtraIntrinsicHelpers{
        constant_size_allocation: constant_size_allocation as *const c_void
    }
}
