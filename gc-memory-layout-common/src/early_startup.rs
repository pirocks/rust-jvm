use std::ffi::c_void;
use std::mem::size_of;
use std::num::NonZeroUsize;
use std::ptr::NonNull;

use crate::memory_regions::RegionHeader;

pub const MEGABYTE: usize = 1024 * 1024;
pub const GIGABYTE: usize = 1024 * MEGABYTE;
pub const TERABYTE: usize = 1024 * GIGABYTE;

pub const MAX_REGIONS_SIZE: usize = 8 * TERABYTE;
pub const MAX_REGIONS_SIZE_SIZE: usize = 43;
static_assertions::const_assert_eq!(1usize << MAX_REGIONS_SIZE_SIZE, MAX_REGIONS_SIZE);

pub const SMALL_REGION_SIZE_SIZE: usize = 12;
pub const SMALL_REGION_SIZE: usize = 4096;
static_assertions::const_assert_eq!(1 << SMALL_REGION_SIZE_SIZE, SMALL_REGION_SIZE);
pub const MEDIUM_REGION_SIZE_SIZE: usize = 15;
pub const MEDIUM_REGION_SIZE: usize = 32768;
static_assertions::const_assert_eq!(1 << MEDIUM_REGION_SIZE_SIZE, MEDIUM_REGION_SIZE);
pub const LARGE_REGION_SIZE_SIZE: usize = 20;
pub const LARGE_REGION_SIZE: usize = MEGABYTE;
static_assertions::const_assert_eq!(1 << LARGE_REGION_SIZE_SIZE, LARGE_REGION_SIZE);
pub const EXTRA_LARGE_REGION_SIZE_SIZE: usize = 34;
pub const EXTRA_LARGE_REGION_SIZE: usize = 16 * GIGABYTE;
static_assertions::const_assert_eq!(1 << EXTRA_LARGE_REGION_SIZE_SIZE, EXTRA_LARGE_REGION_SIZE);

#[repr(packed, C)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Regions {
    pub small_regions: NonNull<c_void>,
    pub medium_regions: NonNull<c_void>,
    pub large_regions: NonNull<c_void>,
    pub extra_large_regions: NonNull<c_void>,
}

unsafe impl Send for Regions {}

unsafe impl Sync for Regions {}

impl Regions {
    pub fn base_regions_address(&self, region: Region) -> NonNull<c_void> {
        match region {
            Region::Small => self.small_regions,
            Region::Medium => self.medium_regions,
            Region::Large => self.large_regions,
            Region::ExtraLarge => self.extra_large_regions
        }
    }
}

pub unsafe fn map_address(ptr: NonNull<c_void>) {
    let map_flags = libc::MAP_PRIVATE | libc::MAP_ANONYMOUS | libc::MAP_NORESERVE;
    let prot_flags = libc::PROT_WRITE | libc::PROT_READ;
    let res_ptr = libc::mmap(ptr.as_ptr(), MAX_REGIONS_SIZE, prot_flags, map_flags, -1, 0);
    if res_ptr != ptr.as_ptr() {
        panic!()
    }
}

pub const SMALL_REGION_BASE: usize = 1usize;
pub const MEDIUM_REGION_BASE: usize = 3usize;
pub const LARGE_REGION_BASE: usize = 5usize;
pub const EXTRA_LARGE_REGION_BASE: usize = 7usize;

pub fn get_regions() -> Regions {
    // let res = assert_no_alloc(|| {
    //whats going on with the shifts here is that we want to encode region size in address, but can't put the whole region size , so small region is 1 and grows from there
    //todo will need to remmap twice to shrink once has lazily grown . https://linux.die.net/man/2/mremap
    let small_regions = NonNull::new((SMALL_REGION_BASE << MAX_REGIONS_SIZE_SIZE) as *mut c_void).unwrap();
    let medium_regions = NonNull::new((MEDIUM_REGION_BASE << MAX_REGIONS_SIZE_SIZE) as *mut c_void).unwrap();
    let large_regions = NonNull::new((LARGE_REGION_BASE << MAX_REGIONS_SIZE_SIZE) as *mut c_void).unwrap();
    let extra_large_regions = NonNull::new((EXTRA_LARGE_REGION_BASE << MAX_REGIONS_SIZE_SIZE) as *mut c_void).unwrap();
    unsafe {
        map_address(small_regions);
        map_address(medium_regions);
        map_address(large_regions);
        map_address(extra_large_regions);
    }
    let res = Regions { small_regions, medium_regions, large_regions, extra_large_regions };
    res
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Region {
    Small,
    Medium,
    Large,
    ExtraLarge,
}

impl Region {
    pub fn bigger(&self) -> Region {
        match self {
            Region::Small => Region::Medium,
            Region::Medium => Region::Large,
            Region::Large => Region::ExtraLarge,
            Region::ExtraLarge => Region::ExtraLarge,
        }
    }

    pub fn max(self, other: Region) -> Region{
        match self {
            Region::Small => {
                other
            }
            Region::Medium => {
                match other {
                    Region::Small => Region::Medium,
                    _ => other
                }
            }
            Region::Large => {
                match other {
                    Region::Small => Region::Large,
                    Region::Medium => Region::Large,
                    _ => other,
                }
            }
            Region::ExtraLarge => {
                Region::ExtraLarge
            }
        }
    }
}

pub fn region_pointer_to_region(ptr: u64) -> Region {
    let shifted = ptr >> MAX_REGIONS_SIZE_SIZE;
    if shifted == SMALL_REGION_BASE as u64 {
        return Region::Small;
    }
    if shifted == MEDIUM_REGION_BASE as u64 {
        return Region::Medium;
    }
    if shifted == LARGE_REGION_BASE as u64 {
        return Region::Large;
    }
    if shifted == EXTRA_LARGE_REGION_BASE as u64 {
        return Region::ExtraLarge;
    }
    eprintln!("{:X}", ptr);
    panic!()
}

pub fn region_pointer_to_region_size(ptr: u64) -> u64 {
    let res_shift = region_pointer_to_region_size_size(ptr);
    let res = 1 << res_shift;
    res
}

pub fn region_pointer_to_region_size_size(ptr: u64) -> u8 {
    let res = match ptr >> MAX_REGIONS_SIZE_SIZE {
        1 => SMALL_REGION_SIZE_SIZE,
        3 => MEDIUM_REGION_SIZE_SIZE,
        5 => LARGE_REGION_SIZE_SIZE,
        7 => EXTRA_LARGE_REGION_SIZE_SIZE,
        _ => {
            dbg!(ptr);
            eprintln!("{:X}", ptr);
            panic!()
        }
    };
    res as u8
}

impl Region {
    pub fn smallest_which_fits(size: NonZeroUsize) -> Region {
        match size.get() + size_of::<RegionHeader>() {
            0..SMALL_REGION_SIZE => Region::Small,
            SMALL_REGION_SIZE..MEDIUM_REGION_SIZE => Region::Medium,
            MEDIUM_REGION_SIZE..LARGE_REGION_SIZE => Region::Large,
            LARGE_REGION_SIZE..=EXTRA_LARGE_REGION_SIZE => Region::ExtraLarge,
            _ => panic!("this is a rather large object"),
        }
    }

    pub fn region_size(&self) -> usize {
        match self {
            Region::Small => SMALL_REGION_SIZE,
            Region::Medium => MEDIUM_REGION_SIZE,
            Region::Large => LARGE_REGION_SIZE,
            Region::ExtraLarge => EXTRA_LARGE_REGION_SIZE,
        }
    }

    pub fn region_base(&self, regions: &Regions) -> NonNull<c_void> {
        match self {
            Region::Small => regions.small_regions,
            Region::Medium => regions.medium_regions,
            Region::Large => regions.large_regions,
            Region::ExtraLarge => regions.extra_large_regions,
        }
    }
}

#[cfg(test)]
pub mod test {
    use std::ffi::c_void;
    use std::ptr::NonNull;

    use crate::early_startup::{EXTRA_LARGE_REGION_BASE, EXTRA_LARGE_REGION_SIZE_SIZE, LARGE_REGION_BASE, LARGE_REGION_SIZE_SIZE, MAX_REGIONS_SIZE, MAX_REGIONS_SIZE_SIZE, MEDIUM_REGION_BASE, MEDIUM_REGION_SIZE_SIZE, Region, region_pointer_to_region, region_pointer_to_region_size_size, SMALL_REGION_BASE, SMALL_REGION_SIZE_SIZE};

    #[test]
    pub fn test_region_pointer_to_region_size_size() {
        unsafe {
            let a_ptr = NonNull::new((SMALL_REGION_BASE << MAX_REGIONS_SIZE_SIZE) as *mut c_void).unwrap().as_ptr();
            for offset in [0u64, 10000u64, 8967550u64, (MAX_REGIONS_SIZE - 1) as u64] {
                assert_eq!(region_pointer_to_region(a_ptr.offset(offset as isize) as u64), Region::Small);
                assert_eq!(region_pointer_to_region_size_size(a_ptr.offset(offset as isize) as u64), SMALL_REGION_SIZE_SIZE as u8);
            }

            let a_ptr = NonNull::new((MEDIUM_REGION_BASE << MAX_REGIONS_SIZE_SIZE) as *mut c_void).unwrap().as_ptr();
            for offset in [0u64, 10000u64, 8967550u64, (MAX_REGIONS_SIZE - 1) as u64] {
                assert_eq!(region_pointer_to_region(a_ptr.offset(offset as isize) as u64), Region::Medium);
                assert_eq!(region_pointer_to_region_size_size(a_ptr.offset(offset as isize) as u64), MEDIUM_REGION_SIZE_SIZE as u8);
            }

            let a_ptr = NonNull::new((LARGE_REGION_BASE << MAX_REGIONS_SIZE_SIZE) as *mut c_void).unwrap().as_ptr();
            for offset in [0u64, 10000u64, 8967550u64, (MAX_REGIONS_SIZE - 1) as u64] {
                assert_eq!(region_pointer_to_region(a_ptr.offset(offset as isize) as u64), Region::Large);
                assert_eq!(region_pointer_to_region_size_size(a_ptr.offset(offset as isize) as u64), LARGE_REGION_SIZE_SIZE as u8);
            }

            let a_ptr = NonNull::new((EXTRA_LARGE_REGION_BASE << MAX_REGIONS_SIZE_SIZE) as *mut c_void).unwrap().as_ptr();
            for offset in [0u64, 10000u64, 8967550u64, (MAX_REGIONS_SIZE - 1) as u64] {
                assert_eq!(region_pointer_to_region(a_ptr.offset(offset as isize) as u64), Region::ExtraLarge);
                assert_eq!(region_pointer_to_region_size_size(a_ptr.offset(offset as isize) as u64), EXTRA_LARGE_REGION_SIZE_SIZE as u8);
            }
        }
    }
}
