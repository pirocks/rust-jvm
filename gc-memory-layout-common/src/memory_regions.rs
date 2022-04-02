use std::collections::HashMap;
use std::ffi::c_void;
use std::mem::size_of;
use std::ptr::{NonNull, null_mut};
use std::sync::atomic::{AtomicUsize, Ordering};

use iced_x86::code_asm::CodeAssembler;

use another_jit_vm::Register;
use rust_jvm_common::compressed_classfile::{CPDType, CPRefType};
use rust_jvm_common::compressed_classfile::names::CClassName;
use rust_jvm_common::loading::LoaderName;
use vtable::RawNativeVTable;
use crate::early_startup::{LARGE_REGION_SIZE_SIZE, MAX_REGIONS_SIZE_SIZE, MEDIUM_REGION_SIZE_SIZE, Region, region_pointer_to_region_size, Regions, SMALL_REGION_SIZE, SMALL_REGION_SIZE_SIZE, TERABYTE};

use crate::layout::ArrayMemoryLayout;

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
    pub vtable_ptr: *mut RawNativeVTable,
}

impl RegionHeader {
    pub unsafe fn get_allocation(region_header: NonNull<RegionHeader>) -> Option<NonNull<c_void>> {
        let region_base = region_header.as_ptr().add(size_of::<RegionHeader>());
        assert_eq!((region_header.as_ptr() as *mut c_void).add(size_of::<RegionHeader>()), region_base as *mut c_void);
        let current_index = region_header.as_ref().num_current_elements.fetch_add(1, Ordering::SeqCst);
        assert!(current_index < region_header.as_ref().region_max_elements);//todo for now we only get one object per region
        if current_index < region_header.as_ref().region_max_elements{
            return None
        }
        let res = (region_base as *mut c_void).add((current_index * region_header.as_ref().region_elem_size) as usize);
        libc::memset(res, 0, region_header.as_ref().region_elem_size);
        Some(NonNull::new(res).unwrap())
    }
}


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
    pub free_small_region_index: usize,
    pub free_medium_region_index: usize,
    pub free_large_region_index: usize,
    pub free_extra_large_region_index: usize,
    //indexed by allocated type id
    pub types: Vec<AllocatedObjectType>,
    pub current_region_type: Vec<Region>,
    pub type_to_region_datas: Vec<Vec<(Region, usize)>>,
    pub current_region_index: Vec<Option<usize>>,
    //end indexed by allocated type id
    pub types_reverse: HashMap<AllocatedObjectType, AllocatedTypeID>,
}

impl MemoryRegions {
    pub fn new(regions: Regions) -> MemoryRegions {
        MemoryRegions {
            early_mmaped_regions: regions,
            free_small_region_index: 0,
            free_medium_region_index: 0,
            free_large_region_index: 0,
            free_extra_large_region_index: 0,
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
                let region_to_use = Region::smallest_which_fits(object_size);
                self.current_region_type.push(region_to_use);
                self.current_region_index.push(None);
                self.type_to_region_datas.push(vec![]);
                self.types_reverse.insert(type_.clone(), new_id);
                new_id
            }
            Some(cur_id) => *cur_id,
        }
    }

    pub fn new_region_for(&mut self, to_allocate_type: AllocatedObjectType) -> NonNull<RegionHeader> {
        let early_mapped_regions = self.early_mmaped_regions;
        let type_id = self.lookup_or_add_type(&to_allocate_type);
        let current_region_to_use = self.current_region_type[type_id.0 as usize];
        let regions_base = self.early_mmaped_regions.base_regions_address(current_region_to_use);
        let free_index = self.current_free_index_by_region(current_region_to_use);
        let our_index = *free_index;
        *free_index += 1;
        let region_header_ptr = NonNull::new(unsafe { regions_base.as_ptr().add(current_region_to_use.region_size() * our_index) }).unwrap().cast::<RegionHeader>();
        self.type_to_region_datas[type_id.0 as usize].push((current_region_to_use.clone(), our_index));
        unsafe {
            region_header_ptr.as_ptr().write(RegionHeader {
                num_current_elements: AtomicUsize::new(0),
                region_max_elements: current_region_to_use.region_size() / to_allocate_type.size(),
                region_elem_size: to_allocate_type.size(),
                region_type: type_id,
                vtable_ptr: null_mut()
            });
        }
        region_header_ptr
    }


    fn current_free_index_by_region(&mut self, region: Region) -> &mut usize{
        match region {
            Region::Small => &mut self.free_small_region_index,
            Region::Medium => &mut self.free_medium_region_index,
            Region::Large => &mut self.free_large_region_index,
            Region::ExtraLarge => &mut self.free_extra_large_region_index,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct BaseAddressAndMask {
    pub mask: u64,
    pub base_address: *mut c_void,
}

impl MemoryRegions {
    pub fn generate_find_vtable_ptr(assembler: &mut CodeAssembler, ptr: Register, temp_1: Register, temp_2: Register, out: Register) {
        todo!()
    }

    pub fn generate_find_allocated_type_id(assembler: &mut CodeAssembler, ptr: Register, temp_1: Register, temp_2: Register, out: Register) {
        todo!()
    }

    pub fn generate_find_object_region_header(assembler: &mut CodeAssembler, ptr: Register, temp_1: Register, temp_2: Register, out: Register) {
        //from compiled region_pointer_to_region_size
        //let as_u64 = ptr.as_ptr() as u64;
        //let region_size = region_pointer_to_region_size(as_u64);
        //let region_mask = !(u64::MAX << region_size);
        //let masked = as_u64 & region_mask;
        //unsafe { (masked as *const c_void as *const RegionHeader).as_ref().unwrap() }
        assembler.mov(temp_2.to_native_64(), ptr.to_native_64()).unwrap();
        Self::generate_region_pointer_to_region_size(assembler, ptr, temp_1, out);
        assembler.mov(ptr.to_native_64(), temp_2.to_native_64()).unwrap();
        assembler.mov(temp_1.to_native_64(), (-1i64) as u64).unwrap();
        assembler.shl(temp_1.to_native_64(), out.to_native_8()).unwrap();
        assembler.not(temp_1.to_native_64()).unwrap();
        //temp_1 is region_mask
        assembler.and(ptr.to_native_64(), temp_1.to_native_64()).unwrap();
        //ptr is masked
        assembler.mov(out.to_native_64(), ptr.to_native_64()).unwrap();
        assembler.mov(ptr.to_native_64(), temp_2.to_native_64()).unwrap();
    }

    fn generate_region_pointer_to_region_size(assembler: &mut CodeAssembler, ptr: Register, temp: Register, out: Register) {
        assembler.shr(ptr.to_native_64(), MAX_REGIONS_SIZE_SIZE as u32).unwrap();
        assembler.add(ptr.to_native_32(), -1).unwrap();
        assembler.shr(ptr.to_native_32(), 1).unwrap();
        assembler.add(ptr.to_native_8(), 1).unwrap();
        assembler.add(out.to_native_32(), 1).unwrap();
        assembler.add(temp.to_native_32(), 1).unwrap();
        assembler.shl(temp.to_native_32(), ptr.to_native_8()).unwrap();
        assembler.shr(ptr.to_native_8(), 1).unwrap();
        assembler.and(ptr.to_native_8(), 1).unwrap();
        assembler.shl(out.to_native_64(), ptr.to_native_8()).unwrap();
        assembler.add(out.to_native_64(), temp.to_native_64()).unwrap();
        assembler.add(out.to_native_64(), out.to_native_64()).unwrap();
    }


    pub fn find_object_region_header(&self, ptr: NonNull<c_void>) -> &RegionHeader {
        let as_u64 = ptr.as_ptr() as u64;
        let region_size = region_pointer_to_region_size(as_u64);
        let region_mask = !(u64::MAX << region_size);
        let masked = as_u64 & region_mask;
        unsafe { (masked as *const c_void as *const RegionHeader).as_ref().unwrap() }
    }

    pub fn find_object_allocated_type(&self, ptr: NonNull<c_void>) -> &AllocatedObjectType {
        let header = self.find_object_region_header(ptr);
        let allocated_type_id = header.region_type;
        &self.types[allocated_type_id.0 as usize]
        // let num_zeros = (8 * TERABYTE).trailing_zeros();
        // let mask = !(!0u64 << num_zeros);
        // let region_base_masked_ptr = ptr.as_ptr() as u64 & !mask;
        /*let region_type = if region_base_masked_ptr == self.early_mmaped_regions.small_regions.as_ptr() as u64 {
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
        &self.types[region_type.0 as usize]*/
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