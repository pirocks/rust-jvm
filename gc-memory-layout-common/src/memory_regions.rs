use std::collections::HashMap;
use std::ffi::c_void;
use std::mem::size_of;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicUsize, Ordering};

use early_startup::{EXTRA_LARGE_REGION_SIZE, LARGE_REGION_SIZE, LARGE_REGION_SIZE_SIZE, MEDIUM_REGION_SIZE, MEDIUM_REGION_SIZE_SIZE, region_pointer_to_region_size, Regions, SMALL_REGION_SIZE, SMALL_REGION_SIZE_SIZE, TERABYTE};
use rust_jvm_common::compressed_classfile::{CPDType, CPRefType};
use rust_jvm_common::compressed_classfile::names::CClassName;
use rust_jvm_common::loading::LoaderName;

use crate::layout::ArrayMemoryLayout;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct FramePointerOffset(pub usize);

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum AllocatedObjectType {
    Class { name: CClassName, loader: LoaderName, size: usize },
    ObjectArray { sub_type: CPRefType, sub_type_loader: LoaderName, len: i32 },
    PrimitiveArray { primitive_type: CPDType, len: i32 },
}

impl AllocatedObjectType {
    pub fn size(&self) -> usize {
        match self {
            AllocatedObjectType::Class { size, .. } => {
                if *size == 0 {
                    return 1;
                }
                *size
            }
            AllocatedObjectType::ObjectArray { len, .. } => {
                let array_layout = ArrayMemoryLayout::from_cpdtype(CClassName::object().into());
                let res = array_layout.array_size(*len);
                if res == 0 {
                    panic!()
                } else {
                    res
                }
            }
            AllocatedObjectType::PrimitiveArray { len, primitive_type, .. } => {
                let array_layout = ArrayMemoryLayout::from_cpdtype(*primitive_type);
                array_layout.array_size(*len)
            }
        }
    }

    pub fn as_cpdtype(&self) -> CPDType {
        match self {
            AllocatedObjectType::Class { name, .. } => {
                (*name).into()
            }
            AllocatedObjectType::ObjectArray { sub_type, .. } => {
                CPDType::array(sub_type.to_cpdtype())
            }
            AllocatedObjectType::PrimitiveArray { primitive_type, .. } => {
                CPDType::array(*primitive_type)
            }
        }
    }
}

#[repr(C)]
pub struct RegionHeader {
    pub num_current_elements: AtomicUsize,
    pub region_max_elements: usize,
    pub region_elem_size: usize,
    pub region_type: AllocatedTypeID,
    pub vtable_ptr: *mut c_void,
}

pub struct RegionData {
    pub region_base: NonNull<c_void>,
    pub num_current_elements: AtomicUsize,
    pub region_type: AllocatedTypeID,
    pub region_elem_size: usize,
    pub region_max_elements: usize,
}

impl RegionData {
    pub fn get_allocation(&mut self) -> NonNull<c_void> {
        let region_base = self.region_base;
        let current_index = self.num_current_elements.fetch_add(1, Ordering::SeqCst);
        assert!(current_index < self.region_max_elements);
        let res = unsafe { region_base.as_ptr().offset((current_index * self.region_elem_size) as isize) };
        unsafe { libc::memset(res, 0, self.region_elem_size); }
        NonNull::new(res).unwrap()
    }
}

unsafe impl Send for RegionData {}

unsafe impl Send for MemoryRegions {}

//work around thread locals requiring Sync, when not actually required
unsafe impl Sync for MemoryRegions {}

#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct AllocatedTypeID(pub u64);

//never directly accessed from native code to make syncing this somewhat sane.
// instead native code should get a view of this
// todo open question is what do about reallocs of the vecs, b/c this invalidates the native code view of this
// I guess cloning entire region data is only option?
//ideally I would just allocate a raw region to back the vecs
pub struct MemoryRegions {
    pub early_mmaped_regions: Regions,
    //indexed by region num
    //todo maybe there should be a more packed region to type.
    pub small_region_types: Vec<RegionData>,
    pub medium_region_types: Vec<RegionData>,
    pub large_region_types: Vec<RegionData>,
    pub extra_large_region_types: Vec<RegionData>,
    //end indexed by region num
    //indexed by allocated type id
    pub types: Vec<AllocatedObjectType>,
    pub current_region_type: Vec<RegionToUse>,
    pub type_to_region_datas: Vec<Vec<(RegionToUse, usize)>>,
    pub current_region_index: Vec<Option<usize>>,
    //end indexed by allocated type id
    pub types_reverse: HashMap<AllocatedObjectType, AllocatedTypeID>,
}

impl MemoryRegions {
    pub fn new(regions: Regions) -> MemoryRegions {
        MemoryRegions {
            early_mmaped_regions: regions,
            small_region_types: vec![],
            medium_region_types: vec![],
            large_region_types: vec![],
            extra_large_region_types: vec![],
            types: vec![],
            current_region_type: vec![],
            type_to_region_datas: vec![],
            current_region_index: vec![],
            types_reverse: Default::default(),
        }
    }

    pub fn lookup_or_add_type(&mut self, type_: &AllocatedObjectType) -> AllocatedTypeID {
        let new_id = AllocatedTypeID(self.types_reverse.len() as u64);
        let object_size = type_.size();
        match self.types_reverse.get(type_) {
            None => {
                self.types.push(type_.clone());
                let region_to_use = RegionToUse::smallest_which_fits(object_size);
                self.current_region_type.push(region_to_use);
                self.current_region_index.push(None);
                self.type_to_region_datas.push(vec![]);
                self.types_reverse.insert(type_.clone(), new_id);
                new_id
            }
            Some(cur_id) => *cur_id,
        }
    }

    pub fn find_or_new_region_for(&mut self, to_allocate_type: AllocatedObjectType) -> &mut RegionData {
        //todo doesn't actually find region
        let type_id = self.lookup_or_add_type(&to_allocate_type);
        let current_region_to_use = self.current_region_type[type_id.0 as usize];
        let early_mapped_regions = self.early_mmaped_regions;
        let to_push_to = self.region_from_region_to_use_mut(current_region_to_use);
        let res_target_index = to_push_to.len();
        self.type_to_region_datas[type_id.0 as usize].push((current_region_to_use.clone(), res_target_index));
        let to_push_to = self.region_from_region_to_use_mut(current_region_to_use);
        let new_region_base = match to_push_to.last() {
            Some(x) => unsafe { NonNull::new(x.region_base.as_ptr().offset(current_region_to_use.region_size() as isize)).unwrap() },
            None => current_region_to_use.region_base(&early_mapped_regions),
        };
        let bitmap_length = ((current_region_to_use.region_size()) / (to_allocate_type.size())).div_ceil(8usize);
        assert_ne!(bitmap_length, 0);
        to_push_to.push(RegionData {
            region_base: new_region_base,
            num_current_elements: AtomicUsize::new(0),
            region_type: type_id,
            region_elem_size: to_allocate_type.size(),
            region_max_elements: current_region_to_use.region_size() / to_allocate_type.size(),
        });
        to_push_to.last_mut().unwrap()
    }

    fn region_from_region_to_use_mut(&mut self, region_to_use: RegionToUse) -> &mut Vec<RegionData> {
        match region_to_use {
            RegionToUse::Small => &mut self.small_region_types,
            RegionToUse::Medium => &mut self.medium_region_types,
            RegionToUse::Large => &mut self.large_region_types,
            RegionToUse::ExtraLarge => &mut self.extra_large_region_types,
        }
    }

    fn region_from_region_to_use_ref(&self, region_to_use: RegionToUse) -> &Vec<RegionData> {
        match region_to_use {
            RegionToUse::Small => &self.small_region_types,
            RegionToUse::Medium => &self.medium_region_types,
            RegionToUse::Large => &self.large_region_types,
            RegionToUse::ExtraLarge => &self.extra_large_region_types,
        }
    }

    pub fn pointer_masks_for_type(&self, type_id: AllocatedTypeID) -> Vec<ObjPointerCompareMask> {
        let mut res = vec![];
        for (region_to_use, i) in self.type_to_region_datas[type_id.0 as usize].iter() {
            let region_datas = self.region_from_region_to_use_ref(*region_to_use);
            let region_data = &region_datas[*i];
            assert_eq!(region_data.region_type, type_id);
            let region_base_ptr = region_to_use.region_base(&self.early_mmaped_regions);
            let region_size = region_to_use.region_size();
            unsafe {
                res.push(ObjPointerCompareMask {
                    size: *region_to_use,
                    region_base_ptr,
                    index_in_region: *i,
                    obj_ptr: region_base_ptr.as_ptr().add(region_size * *i),
                });
            }
        }
        res
    }
}

pub struct ObjPointerCompareMask {
    size: RegionToUse,
    region_base_ptr: NonNull<c_void>,
    index_in_region: usize,
    obj_ptr: *mut c_void,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct BaseAddressAndMask {
    pub mask: u64,
    pub base_address: *mut c_void,
}

impl MemoryRegions {
    pub fn find_object_header(&self, ptr: NonNull<c_void>) -> &RegionHeader {
        let as_u64 = ptr.as_ptr() as u64;
        let region_size = region_pointer_to_region_size(as_u64);
        let region_mask = !(u64::MAX << region_size);
        let masked = as_u64 & region_mask;
        dbg!(masked);
        unsafe { (masked as *const c_void as *const RegionHeader).as_ref().unwrap() }
    }

    pub fn find_object_allocated_type(&self, ptr: NonNull<c_void>) -> &AllocatedObjectType {
        let num_zeros = (8 * TERABYTE).trailing_zeros();
        let mask = !(!0u64 << num_zeros);
        let region_base_masked_ptr = ptr.as_ptr() as u64 & !mask;
        let region_type = if region_base_masked_ptr == self.early_mmaped_regions.small_regions.as_ptr() as u64 {
            let region_index = ((ptr.as_ptr() as u64 & mask) / SMALL_REGION_SIZE as u64) as usize;
            let region_data = &self.small_region_types[region_index];
            region_data.region_type
        } else if region_base_masked_ptr == self.early_mmaped_regions.medium_regions.as_ptr() as u64 {
            let region_index = ((ptr.as_ptr() as u64 & mask) / MEDIUM_REGION_SIZE as u64) as usize;
            let region_data = &self.medium_region_types[region_index];
            region_data.region_type
        } else if region_base_masked_ptr == self.early_mmaped_regions.large_regions.as_ptr() as u64 {
            let region_index = ((ptr.as_ptr() as u64 & mask) / LARGE_REGION_SIZE as u64) as usize;
            let region_data = &self.large_region_types[region_index];
            region_data.region_type
        } else if region_base_masked_ptr == self.early_mmaped_regions.extra_large_regions.as_ptr() as u64 {
            todo!();
            /*let region_index = ((ptr.as_ptr() as u64 & mask) / LARGE_REGION_SIZE as u64) as usize;
            let region_data = &self.extra_large_region_types[region_index];
            region_data.region_type*/
        } else {
            dbg!(self.early_mmaped_regions.large_regions);
            dbg!(&self.early_mmaped_regions);
            dbg!(region_base_masked_ptr as *mut c_void);
            dbg!(ptr.as_ptr());
            todo!()
        };
        &self.types[region_type.0 as usize]
    }

    pub fn find_object_base_address_and_mask(&self, ptr: NonNull<c_void>) -> BaseAddressAndMask {
        let num_zeros = (8 * TERABYTE).trailing_zeros();
        let top_level_mask = !(!0u64 << num_zeros);
        let region_base_masked_ptr = ptr.as_ptr() as u64 & !top_level_mask;
        if region_base_masked_ptr == self.early_mmaped_regions.small_regions.as_ptr() as u64 {
            assert!(SMALL_REGION_SIZE.is_power_of_two());
            let region_mask = 1 << SMALL_REGION_SIZE_SIZE;
            BaseAddressAndMask {
                mask: region_mask,
                base_address: (ptr.as_ptr() as u64 & region_mask) as *mut c_void,
            }
        } else if region_base_masked_ptr == self.early_mmaped_regions.medium_regions.as_ptr() as u64 {
            let region_mask = 1 << MEDIUM_REGION_SIZE_SIZE;
            BaseAddressAndMask {
                mask: region_mask,
                base_address: (ptr.as_ptr() as u64 & region_mask) as *mut c_void,
            }
        } else if region_base_masked_ptr == self.early_mmaped_regions.large_regions.as_ptr() as u64 {
            let region_mask = 1 << LARGE_REGION_SIZE_SIZE;
            BaseAddressAndMask {
                mask: region_mask,
                base_address: (ptr.as_ptr() as u64 & region_mask) as *mut c_void,
            }
            // let region_index = ((ptr.as_ptr() as u64 & top_level_mask) / LARGE_REGION_SIZE as u64) as usize;
            // todo!()
        } else {
            dbg!(self.early_mmaped_regions.large_regions);
            dbg!(&self.early_mmaped_regions);
            dbg!(region_base_masked_ptr as *mut c_void);
            dbg!(ptr.as_ptr());
            todo!()
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum RegionToUse {
    Small,
    Medium,
    Large,
    ExtraLarge,
}

impl RegionToUse {
    pub fn smallest_which_fits(size: usize) -> RegionToUse {
        match size + size_of::<RegionHeader>() {
            0..SMALL_REGION_SIZE => RegionToUse::Small,
            SMALL_REGION_SIZE..MEDIUM_REGION_SIZE => RegionToUse::Medium,
            MEDIUM_REGION_SIZE..LARGE_REGION_SIZE => RegionToUse::Large,
            LARGE_REGION_SIZE..=EXTRA_LARGE_REGION_SIZE => RegionToUse::ExtraLarge,
            _ => panic!("this is a rather large object"),
        }
    }

    pub fn region_size(&self) -> usize {
        match self {
            RegionToUse::Small => SMALL_REGION_SIZE,
            RegionToUse::Medium => MEDIUM_REGION_SIZE,
            RegionToUse::Large => LARGE_REGION_SIZE,
            RegionToUse::ExtraLarge => EXTRA_LARGE_REGION_SIZE,
        }
    }

    pub fn region_base(&self, regions: &Regions) -> NonNull<c_void> {
        match self {
            RegionToUse::Small => regions.small_regions,
            RegionToUse::Medium => regions.medium_regions,
            RegionToUse::Large => regions.large_regions,
            RegionToUse::ExtraLarge => regions.extra_large_regions,
        }
    }
}