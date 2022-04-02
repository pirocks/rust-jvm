
use std::ffi::c_void;
use std::mem::size_of;
use std::ptr::NonNull;
use crate::memory_regions::RegionHeader;


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
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Regions {
    pub small_regions: NonNull<c_void>,
    pub medium_regions: NonNull<c_void>,
    pub large_regions: NonNull<c_void>,
    pub extra_large_regions: NonNull<c_void>,
}

impl Regions {
    pub fn base_regions_address(&self, region: Region) -> NonNull<c_void>{
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

#[derive(Copy, Clone,Eq, PartialEq,Hash,Debug)]
pub enum Region {
    Small,
    Medium,
    Large,
    ExtraLarge,
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
    panic!()
}

pub fn region_pointer_to_region_size(ptr: u64) -> u64 {
    let shifted = ptr >> MAX_REGIONS_SIZE_SIZE;
    //1 3 5 7
    let i = ((shifted - 1) >> 1) + 1;
    //1 2 3 4
    let base = 1 << i;
    let shift = (i >> 1) - ((i >> 2) << 1);
    let res = base + (1 << shift) << 1;
    res
}

impl Region {
    pub fn smallest_which_fits(size: usize) -> Region {
        match size + size_of::<RegionHeader>() {
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
    use crate::early_startup::{EXTRA_LARGE_REGION_SIZE_SIZE, get_regions, LARGE_REGION_SIZE_SIZE, MEDIUM_REGION_SIZE_SIZE, Region, region_pointer_to_region, region_pointer_to_region_size, SMALL_REGION_SIZE_SIZE};

    #[test]
    pub fn test_size() {
        let regions = get_regions();
        let size = region_pointer_to_region_size(regions.small_regions.as_ptr() as usize as u64) as usize;
        assert_eq!(size, SMALL_REGION_SIZE_SIZE);

        let size = region_pointer_to_region_size(regions.medium_regions.as_ptr() as usize as u64) as usize;
        assert_eq!(size, MEDIUM_REGION_SIZE_SIZE);

        let size = region_pointer_to_region_size(regions.large_regions.as_ptr() as usize as u64) as usize;
        assert_eq!(size, LARGE_REGION_SIZE_SIZE);

        let size = region_pointer_to_region_size(regions.extra_large_regions.as_ptr() as usize as u64) as usize;
        assert_eq!(size, EXTRA_LARGE_REGION_SIZE_SIZE);
    }

    #[test]
    pub fn test_region() {
        let regions = get_regions();
        let small_region = region_pointer_to_region(regions.small_regions.as_ptr() as usize as u64);
        match small_region {
            Region::Small => {}
            _ => panic!()
        }

        let medium_region = region_pointer_to_region(regions.medium_regions.as_ptr() as usize as u64);
        match medium_region {
            Region::Medium => {}
            _ => panic!()
        }
    }
}
