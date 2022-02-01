// #[macro_use]
// extern crate static_assertions;

use std::ffi::c_void;

// use assert_no_alloc::*;
use procmaps::Mappings;

// #[cfg(debug_assertions)] // required when disable_release is set (default)
// #[global_allocator]
// static A: AllocDisabler = AllocDisabler;

pub const MEGABYTE: usize = 1024 * 1024;
pub const GIGABYTE: usize = 1024 * MEGABYTE;
pub const TERABYTE: usize = 1024 * GIGABYTE;

pub const MAX_REGIONS_SIZE: usize = 8 * TERABYTE;
pub const MAX_REGIONS_SIZE_SIZE: usize = 43;
static_assertions::const_assert_eq!(1usize << MAX_REGIONS_SIZE_SIZE, MAX_REGIONS_SIZE);

pub const SMALL_REGION_SIZE_SIZE: usize = 6;
pub const SMALL_REGION_SIZE: usize = 64;
static_assertions::const_assert_eq!(1 << SMALL_REGION_SIZE_SIZE, SMALL_REGION_SIZE);
pub const MEDIUM_REGION_SIZE_SIZE: usize = 12;
pub const MEDIUM_REGION_SIZE: usize = 4096;
static_assertions::const_assert_eq!(1 << MEDIUM_REGION_SIZE_SIZE, MEDIUM_REGION_SIZE);
pub const LARGE_REGION_SIZE_SIZE: usize = 20;
pub const LARGE_REGION_SIZE: usize = MEGABYTE;
static_assertions::const_assert_eq!(1 << LARGE_REGION_SIZE_SIZE, LARGE_REGION_SIZE);
pub const EXTRA_LARGE_REGION_SIZE_SIZE: usize = 34;
pub const EXTRA_LARGE_REGION_SIZE: usize = 16 * GIGABYTE;
static_assertions::const_assert_eq!(1 << EXTRA_LARGE_REGION_SIZE_SIZE, EXTRA_LARGE_REGION_SIZE);

#[repr(packed, C)]
pub struct Regions {
    pub small_regions: *mut c_void,
    pub medium_regions: *mut c_void,
    pub large_regions: *mut c_void,
    pub extra_large_regions: *mut c_void,
}

pub unsafe fn map_address(ptr: *mut c_void) {
    let map_flags = /*libc::MAP_FIXED |*/ libc::MAP_PRIVATE | libc::MAP_ANONYMOUS | libc::MAP_NORESERVE;
    let prot_flags = libc::PROT_WRITE | libc::PROT_READ;
    let res_ptr = libc::mmap(ptr, MAX_REGIONS_SIZE, prot_flags, map_flags, -1, 0);
    if res_ptr != ptr {
        panic!()
    }
}

pub const SMALL_REGION_BASE: usize = 1usize;
pub const MEDIUM_REGION_BASE: usize = 3usize;
pub const LARGE_REGION_BASE: usize = 5usize;
pub const EXTRA_LARGE_REGION_BASE: usize = 7usize;

pub fn get_regions() -> Regions {
    let _maps_before = Mappings::from_pid(unsafe { libc::getpid() }).unwrap();
    // let res = assert_no_alloc(|| {
    //whats going on with the shifts here is that we want to encode region size in address, but can't put the whole region size , so small region is 1 and grows from there
    //todo will need to remmap twice to shrink once has lazily grown . https://linux.die.net/man/2/mremap
    let small_regions = (SMALL_REGION_BASE << MAX_REGIONS_SIZE_SIZE) as *mut c_void;
    let medium_regions = (MEDIUM_REGION_BASE << MAX_REGIONS_SIZE_SIZE) as *mut c_void;
    let large_regions = (LARGE_REGION_BASE << MAX_REGIONS_SIZE_SIZE) as *mut c_void;
    let extra_large_regions = (EXTRA_LARGE_REGION_BASE << MAX_REGIONS_SIZE_SIZE) as *mut c_void;
    unsafe {
        map_address(small_regions);
        map_address(medium_regions);
        map_address(large_regions);
        map_address(extra_large_regions);
    }
    let res = Regions { small_regions, medium_regions, large_regions, extra_large_regions };
    // });
    let _maps_after = Mappings::from_pid(unsafe { libc::getpid() }).unwrap();
    res
}
