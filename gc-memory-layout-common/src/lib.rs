#![feature(exclusive_range_pattern)]
#![feature(once_cell)]

use std::ptr::NonNull;

use libc::size_t;

use array_memory_layout::layout::ArrayMemoryLayout;
use jvmti_jni_bindings::{jint, jobject};

use crate::memory_regions::MemoryRegions;

pub mod frame_layout;
pub mod memory_regions;
pub mod allocated_object_types;
pub mod early_startup;
#[cfg(test)]
pub mod test;


//todo need somewhere better for this
pub unsafe extern "C" fn array_copy_no_validate(src: jobject, src_pos: jint, dst: jobject, dst_pos: jint, length: jint) {
    let array_elem_type = MemoryRegions::find_object_region_header(NonNull::new(src).unwrap().cast()).array_elem_type.unwrap();
    let array_layout = ArrayMemoryLayout::from_cpdtype(array_elem_type);
    let elem_size = array_layout.elem_size().get() as usize;
    let length = length as usize;

    let dst = NonNull::new(dst.cast()).unwrap();
    let src = NonNull::new(src.cast()).unwrap();

    let dst_raw = array_layout.calculate_index_address(dst, dst_pos).inner();
    let src_raw = array_layout.calculate_index_address(src, src_pos).inner();

    if dst_raw.as_ptr().add(length * elem_size) > src_raw.as_ptr() && src_raw.as_ptr() > dst_raw.as_ptr() {
        libc::memcpy(dst_raw.as_ptr(),
                     src_raw.as_ptr(), (length * elem_size) as size_t);
    } else {
        libc::memmove(dst_raw.as_ptr(),
                      src_raw.as_ptr(), (length * elem_size) as size_t);
    }
}
