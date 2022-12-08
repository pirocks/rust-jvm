use std::cell::OnceCell;
use std::num::NonZeroUsize;
use std::ptr::{NonNull, null};
use itertools::Itertools;

use libc::c_void;
use once_cell::sync::OnceCell;

use rust_jvm_common::compressed_classfile::compressed_types::CompressedParsedDescriptorType;
use gc_memory_layout_common::allocated_object_types::{AllocatedObjectType, AllocatedObjectTypeWithSize};

use gc_memory_layout_common::memory_regions::{MemoryRegions};

static REGIONS: OnceCell<Regions> = OnceCell::new();

#[test]
pub fn allocate_small() {
    let regions = *REGIONS.get_or_init(||get_regions());
    let mut memory_regions = MemoryRegions::new(regions);
    let size_1 = AllocatedObjectTypeWithSize { allocated_object_type: AllocatedObjectType::RawConstantSize { id: 1 }, size: NonZeroUsize::new(1).unwrap() };
    let size_2 = AllocatedObjectTypeWithSize { allocated_object_type: AllocatedObjectType::RawConstantSize { id: 2 }, size: NonZeroUsize::new(8).unwrap() };
    let res_1_1 = memory_regions.allocate(&size_1);
    let res_1_2 = memory_regions.allocate(&size_1);
    let res_8_1 = memory_regions.allocate(&size_2);
    let res_8_2 = memory_regions.allocate(&size_2);
    assert_eq!(memory_regions.find_object_allocated_type(res_1_1), &size_1.allocated_object_type);
    assert_eq!(memory_regions.find_object_allocated_type(res_8_1), &size_2.allocated_object_type);
    for _ in 0..1000 {
        let res_1 = memory_regions.allocate(&size_1);
        assert_eq!(memory_regions.find_object_allocated_type(res_1), &size_1.allocated_object_type);
    }
    let size_3 = AllocatedObjectTypeWithSize { allocated_object_type: AllocatedObjectType::RawConstantSize { id: 3 }, size: NonZeroUsize::new(16).unwrap() };
    let size_3_allocs = (0..10000).map(|i|{
        let res_1 = memory_regions.allocate(&size_3);
        unsafe { libc::memset(res_1.as_ptr(), 0, 16); }
        assert_eq!(memory_regions.find_object_allocated_type(res_1), &size_3.allocated_object_type);
        res_1
    }).collect_vec();
    let object_vtable = NonNull::new(0x1111111111111111 as *mut c_void).unwrap().cast();
    let array_itable = NonNull::new(0x1111111111111111 as *mut c_void).unwrap().cast();
    let allocated_object_type = AllocatedObjectType::PrimitiveArray {
        primitive_type: CompressedParsedDescriptorType::BooleanType,
        object_vtable,
        array_itable,
        array_interfaces: null(),
        interfaces_len: 0,
    };
    let allocated_first = memory_regions.allocate(&AllocatedObjectTypeWithSize { allocated_object_type: allocated_object_type.clone(), size: NonZeroUsize::new(10).unwrap() });
    unsafe { allocated_first.as_ptr().offset(9).cast::<u8>().write(255); }
    let allocated_second = memory_regions.allocate(&AllocatedObjectTypeWithSize { allocated_object_type, size: NonZeroUsize::new(10).unwrap() });
    unsafe { allocated_second.as_ptr().cast::<u8>().write(15); }
    unsafe { assert_eq!(allocated_first.as_ptr().offset(9).cast::<u8>().read(), 255); }
    assert!(size_3_allocs.into_iter().all(|ptr|{
        memory_regions.find_object_allocated_type(ptr) ==  &size_3.allocated_object_type
    }));
    assert_eq!(memory_regions.find_object_allocated_type(res_1_1), &size_1.allocated_object_type);
    assert_eq!(memory_regions.find_object_allocated_type(res_8_1), &size_2.allocated_object_type);
}

use gc_memory_layout_common::early_startup::{EXTRA_LARGE_REGION_SIZE_SIZE, get_regions, LARGE_REGION_SIZE_SIZE, MEDIUM_REGION_SIZE_SIZE, Region, region_pointer_to_region, region_pointer_to_region_size, Regions, SMALL_REGION_SIZE_SIZE};

#[test]
pub fn test_size() {
    let regions = *REGIONS.get_or_init(||get_regions());
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
    let regions = *REGIONS.get_or_init(||get_regions());
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


#[test]
pub fn test_static_field_sync(){

}