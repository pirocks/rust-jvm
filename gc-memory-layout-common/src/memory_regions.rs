use std::collections::HashMap;
use std::ffi::c_void;
use std::mem::size_of;
use std::ptr::{NonNull, null, null_mut};
use std::sync::Arc;
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

use iced_x86::code_asm::{cl, CodeAssembler, CodeLabel, ecx, rcx};
use memoffset::offset_of;

use another_jit_vm::Register;
use inheritance_tree::ClassID;
use inheritance_tree::paths::BitPath256;
use interface_vtable::ITableRaw;
use jvmti_jni_bindings::jclass;
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::{CPDType, CPRefType};


use rust_jvm_common::loading::LoaderName;
use vtable::RawNativeVTable;

use crate::early_startup::{EXTRA_LARGE_REGION_SIZE_SIZE, LARGE_REGION_SIZE_SIZE, MAX_REGIONS_SIZE_SIZE, MEDIUM_REGION_SIZE_SIZE, Region, region_pointer_to_region_size_size, Regions, SMALL_REGION_SIZE, SMALL_REGION_SIZE_SIZE, TERABYTE};
use crate::layout::ArrayMemoryLayout;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum AllocatedObjectType {
    Class {
        name: CClassName,
        loader: LoaderName,
        size: usize,
        vtable: NonNull<RawNativeVTable>,
        itable: NonNull<ITableRaw>,
        inheritance_bit_vec: Option<NonNull<BitPath256>>,
        interfaces: *const ClassID,
        interfaces_len: usize,
    },
    ObjectArray {
        sub_type: CPRefType,
        sub_type_loader: LoaderName,
        len: i32,
        object_vtable: NonNull<RawNativeVTable>,
        array_itable: NonNull<ITableRaw>,
        array_interfaces: *const ClassID,
        interfaces_len: usize,
    },
    PrimitiveArray {
        primitive_type: CPDType,
        len: i32,
        object_vtable: NonNull<RawNativeVTable>,
        array_itable: NonNull<ITableRaw>,
        array_interfaces: *const ClassID,
        interfaces_len: usize,
    },
    Raw { size: usize },
}

impl AllocatedObjectType {
    pub fn inheritance_bit_vec(&self) -> *const BitPath256 {
        match self {
            AllocatedObjectType::Class { inheritance_bit_vec, .. } => inheritance_bit_vec.map(|x| x.as_ptr() as *const BitPath256).unwrap_or(null()),
            AllocatedObjectType::ObjectArray { .. } |
            AllocatedObjectType::PrimitiveArray { .. } |
            AllocatedObjectType::Raw { .. } => {
                null_mut()
            }
        }
    }


    pub fn vtable(&self) -> Option<NonNull<RawNativeVTable>> {
        match self {
            AllocatedObjectType::Class { vtable, .. } => {
                Some(*vtable)
            }
            AllocatedObjectType::ObjectArray { object_vtable, .. } => Some(*object_vtable),
            AllocatedObjectType::PrimitiveArray { object_vtable, .. } => Some(*object_vtable),
            AllocatedObjectType::Raw { .. } => None,
        }
    }

    pub fn itable(&self) -> Option<NonNull<ITableRaw>> {
        match self {
            AllocatedObjectType::Class { itable, .. } => {
                Some(*itable)
            }
            AllocatedObjectType::ObjectArray { array_itable, .. } => Some(*array_itable),
            AllocatedObjectType::PrimitiveArray { array_itable, .. } => Some(*array_itable),
            AllocatedObjectType::Raw { .. } => None,
        }
    }

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
            AllocatedObjectType::Raw { size } => {
                *size
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
            AllocatedObjectType::Raw { .. } => {
                panic!()
            }
        }
    }

    pub fn interfaces_ptr(&self) -> *const ClassID {
        *match self {
            AllocatedObjectType::Class { interfaces, .. } => {
                interfaces
            }
            AllocatedObjectType::ObjectArray { array_interfaces, .. } => {
                array_interfaces
            }
            AllocatedObjectType::PrimitiveArray { array_interfaces, .. } => {
                array_interfaces
            }
            AllocatedObjectType::Raw { .. } => panic!()
        }
    }

    pub fn interfaces_len(&self) -> usize {
        *match self {
            AllocatedObjectType::Class { interfaces_len, .. } => {
                interfaces_len
            }
            AllocatedObjectType::ObjectArray { interfaces_len, .. } => {
                interfaces_len
            }
            AllocatedObjectType::PrimitiveArray { interfaces_len, .. } => {
                interfaces_len
            }
            AllocatedObjectType::Raw { .. } => panic!()
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct RegionHeader {
    region_header_magic_1: u32,
    pub num_current_elements: AtomicUsize,
    pub region_max_elements: usize,
    pub region_elem_size: usize,
    pub region_type: AllocatedTypeID,
    pub inheritance_bit_path_ptr: *const BitPath256,
    pub vtable_ptr: *mut RawNativeVTable,
    pub itable_ptr: *mut ITableRaw,
    //todo in future instead of iterating this could be done with zero page mapped everywhere to make sparse array
    pub interface_ids_list: *const ClassID,
    pub interface_ids_list_len: usize,
    pub class_pointer_cache: jclass,
    region_header_magic_2: u32,
}

impl RegionHeader {
    pub const REGION_HEADER_MAGIC: u32 = 0xddeeaadd;

    pub unsafe fn get_allocation(region_header: NonNull<RegionHeader>) -> Option<NonNull<c_void>> {
        // assert!(dbg!(size_of::<RegionHeader>()) < SMALL_REGION_SIZE);//todo deal with this
        let region_base = region_header.as_ptr().add(1);
        assert_eq!(region_header.as_ref().region_header_magic_1, RegionHeader::REGION_HEADER_MAGIC);
        assert_eq!(region_header.as_ref().region_header_magic_2, RegionHeader::REGION_HEADER_MAGIC);
        let before_type = region_header.as_ref().region_type;
        assert_eq!((region_header.as_ptr() as *mut c_void).add(size_of::<RegionHeader>()), region_base as *mut c_void);
        let current_index = region_header.as_ref().num_current_elements.fetch_add(1, Ordering::SeqCst);
        assert!(current_index < region_header.as_ref().region_max_elements);
        if current_index == region_header.as_ref().region_max_elements {
            return None;
        }
        assert!(current_index < region_header.as_ref().region_max_elements);
        let res = (region_base as *mut c_void).add((current_index * region_header.as_ref().region_elem_size) as usize);
        libc::memset(res, 0, region_header.as_ref().region_elem_size);
        assert_eq!(region_header.as_ref().region_header_magic_1, RegionHeader::REGION_HEADER_MAGIC);
        assert_eq!(region_header.as_ref().region_header_magic_2, RegionHeader::REGION_HEADER_MAGIC);
        assert_eq!(before_type, region_header.as_ref().region_type);
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
    pub current_region_header: Vec<Arc<AtomicPtr<RegionHeader>>>,
    pub current_region_type: Vec<Region>,
    pub type_to_region_datas: Vec<Vec<(Region, usize)>>,
    pub current_region_index: Vec<Option<usize>>,
    //end indexed by allocated type id
    pub types_reverse: HashMap<AllocatedObjectType, AllocatedTypeID>,
}


static mut OBJECT_ALLOCS: u64 = 0;

impl MemoryRegions {
    pub fn new(regions: Regions) -> MemoryRegions {
        MemoryRegions {
            early_mmaped_regions: regions,
            free_small_region_index: 0,
            free_medium_region_index: 0,
            free_large_region_index: 0,
            free_extra_large_region_index: 0,
            types: vec![],
            current_region_header: vec![],
            current_region_type: vec![],
            type_to_region_datas: vec![],
            current_region_index: vec![],
            types_reverse: Default::default(),
        }
    }

    pub fn get_region_header_raw_ptr(&self, id: AllocatedTypeID) -> *const AtomicPtr<RegionHeader> {
        let other_arc = self.current_region_header[id.0 as usize].clone();
        Arc::into_raw(other_arc)
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
                self.current_region_header.push(Arc::new(AtomicPtr::new(null_mut())));
                self.types_reverse.insert(type_.clone(), new_id);
                new_id
            }
            Some(cur_id) => *cur_id,
        }
    }

    pub fn allocate_with_size(&mut self, to_allocate_type: &AllocatedObjectType) -> (NonNull<c_void>, usize) {
        let region = match self.find_empty_region_for(to_allocate_type) {
            Err(FindRegionError::NoRegion) => {
                //todo need to use bigger region as needed.
                self.new_region_for(to_allocate_type, None, None)
            }
            Err(FindRegionError::RegionFull { prev_region_size, prev_vtable_ptr }) => {
                self.new_region_for(to_allocate_type, Some(prev_region_size.bigger()), Some(prev_vtable_ptr))
            }
            Ok(region) => region,
        };
        let region_type = unsafe { region.as_ref() }.region_type;
        let size = unsafe { region.as_ref() }.region_elem_size;
        unsafe { OBJECT_ALLOCS += size as u64; }
        unsafe { assert_eq!(region.as_ref().region_header_magic_1, RegionHeader::REGION_HEADER_MAGIC); }
        unsafe { assert_eq!(region.as_ref().region_header_magic_2, RegionHeader::REGION_HEADER_MAGIC); }
        assert_ne!(size, 0);
        self.current_region_header[region_type.0 as usize].store(region.as_ptr(), Ordering::SeqCst);
        let res_ptr = unsafe {
            match RegionHeader::get_allocation(region) {
                Some(x) => x,
                None => panic!("this allocation failure really shouldn't happen"),
            }
        };
        let size = unsafe { region.as_ref() }.region_elem_size;
        assert_ne!(size, 0);
        let after_region_type = MemoryRegions::find_object_region_header(res_ptr).region_type;
        assert_eq!(region_type, after_region_type);
        (res_ptr, size)
    }

    pub fn allocate(&mut self, to_allocate_type: &AllocatedObjectType) -> NonNull<c_void> {
        self.allocate_with_size(to_allocate_type).0
    }

    fn region_header_at(&self, region: Region, index: usize, assert: bool) -> NonNull<RegionHeader> {
        let regions_base = self.early_mmaped_regions.base_regions_address(region);
        let res = NonNull::new(unsafe { regions_base.as_ptr().add(region.region_size() * index) }).unwrap().cast::<RegionHeader>();
        if assert {
            unsafe { assert_eq!(res.as_ref().region_header_magic_1, RegionHeader::REGION_HEADER_MAGIC); }
            unsafe { assert_eq!(res.as_ref().region_header_magic_2, RegionHeader::REGION_HEADER_MAGIC); }
            unsafe { assert_ne!(res.as_ref().region_elem_size, 0); }
        }
        res
    }
}

pub enum FindRegionError {
    RegionFull {
        prev_region_size: Region,
        prev_vtable_ptr: *mut RawNativeVTable,
    },
    NoRegion,
}

impl MemoryRegions {
    fn find_empty_region_for(&mut self, to_allocate_type: &AllocatedObjectType) -> Result<NonNull<RegionHeader>, FindRegionError> {
        let type_id = self.lookup_or_add_type(&to_allocate_type);
        let (region, index) = self.type_to_region_datas[type_id.0 as usize].last().ok_or(FindRegionError::NoRegion)?;
        let region_header_ptr = self.region_header_at(*region, *index, true);
        let num_current_elements = unsafe { region_header_ptr.as_ref() }.num_current_elements.load(Ordering::SeqCst);//atomic not useful atm b/c protected by memeory region lock but in future will need atomics
        let max_elements = unsafe { region_header_ptr.as_ref() }.region_max_elements;
        unsafe { assert_eq!(region_header_ptr.as_ref().region_header_magic_1, RegionHeader::REGION_HEADER_MAGIC) }
        unsafe { assert_eq!(region_header_ptr.as_ref().region_header_magic_2, RegionHeader::REGION_HEADER_MAGIC) }
        if num_current_elements >= max_elements {
            // eprintln!("was empty: {}", type_id.0);
            unsafe {
                return Err(FindRegionError::RegionFull {
                    prev_region_size: *region,
                    prev_vtable_ptr: region_header_ptr.as_ref().vtable_ptr,
                });
            }
        }
        Ok(region_header_ptr)
    }


    fn new_region_for(&mut self, to_allocate_type: &AllocatedObjectType, size_override: Option<Region>, prev_vtable_ptr: Option<*mut RawNativeVTable>) -> NonNull<RegionHeader> {
        let early_mapped_regions = self.early_mmaped_regions;
        let type_id = self.lookup_or_add_type(&to_allocate_type);
        let mut current_region_to_use = self.current_region_type[type_id.0 as usize];
        if let Some(size_override) = size_override {
            current_region_to_use = size_override;
        }
        let free_index = self.current_free_index_by_region(current_region_to_use);
        let our_index = *free_index;
        *free_index += 1;
        let region_header_ptr = self.region_header_at(current_region_to_use, our_index, false);
        self.type_to_region_datas[type_id.0 as usize].push((current_region_to_use, our_index));
        if let Some(prev_vtable_ptr) = prev_vtable_ptr {
            assert_eq!(to_allocate_type.vtable().unwrap().as_ptr(), prev_vtable_ptr);
        }
        unsafe {
            let region_elem_size = to_allocate_type.size();
            assert_ne!(region_elem_size, 0);
            region_header_ptr.as_ptr().write(RegionHeader {
                region_header_magic_2: RegionHeader::REGION_HEADER_MAGIC,
                num_current_elements: AtomicUsize::new(0),
                region_max_elements: (current_region_to_use.region_size() - size_of::<RegionHeader>()) / region_elem_size,
                region_elem_size,
                region_type: type_id,
                vtable_ptr: to_allocate_type.vtable().map(|vtable| vtable.as_ptr()).unwrap_or(null_mut()),
                region_header_magic_1: RegionHeader::REGION_HEADER_MAGIC,
                itable_ptr: to_allocate_type.itable().map(|itable| itable.as_ptr()).unwrap_or(null_mut()),
                interface_ids_list: to_allocate_type.interfaces_ptr(),
                interface_ids_list_len: to_allocate_type.interfaces_len(),
                inheritance_bit_path_ptr: to_allocate_type.inheritance_bit_vec(),
                class_pointer_cache: null_mut(),
            });
        }
        region_header_ptr
    }


    fn current_free_index_by_region(&mut self, region: Region) -> &mut usize {
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
    pub fn generate_find_vtable_ptr(assembler: &mut CodeAssembler, ptr: Register, temp_1: Register, temp_2: Register, temp_3: Register, out: Register) {
        Self::generate_find_object_region_header(assembler, ptr, temp_1, temp_2, temp_3, out);
        assembler.mov(out.to_native_64(), out.to_native_64() + offset_of!(RegionHeader,vtable_ptr)).unwrap();
    }

    pub fn generate_find_itable_ptr(assembler: &mut CodeAssembler, ptr: Register, temp_1: Register, temp_2: Register, temp_3: Register, out: Register, fail_label: CodeLabel) {
        Self::generate_find_object_region_header(assembler, ptr, temp_1, temp_2, temp_3, out);
        assembler.test(out.to_native_64(), out.to_native_64()).unwrap();
        assembler.jz(fail_label).unwrap();
        assembler.mov(out.to_native_64(), out.to_native_64() + offset_of!(RegionHeader,itable_ptr)).unwrap();
    }

    pub fn generate_find_class_ptr(assembler: &mut CodeAssembler, ptr: Register, temp_1: Register, temp_2: Register, temp_3: Register, out: Register) {
        Self::generate_find_object_region_header(assembler, ptr, temp_1, temp_2, temp_3, out);
        assembler.mov(out.to_native_64(), out.to_native_64() + offset_of!(RegionHeader,class_pointer_cache)).unwrap();
    }

    pub fn generate_find_allocated_type_id(assembler: &mut CodeAssembler, ptr: Register, temp_1: Register, temp_2: Register, out: Register) {
        todo!()
    }

    pub fn generate_find_object_region_header(assembler: &mut CodeAssembler, ptr: Register, temp_1: Register, temp_2: Register, temp_3: Register, out: Register) {
        //from compiled region_pointer_to_region_size
        //let as_u64 = ptr.as_ptr() as u64;
        //let region_size = region_pointer_to_region_size(as_u64);
        //let region_mask = u64::MAX << region_size;
        //let masked = as_u64 & region_mask;
        //unsafe { (masked as *const c_void as *const RegionHeader).as_ref().unwrap() }
        assembler.mov(temp_3.to_native_64(), ptr.to_native_64()).unwrap();
        Self::generate_region_pointer_to_region_size(assembler, ptr, temp_1, temp_2, out);
        assert_eq!(temp_2.to_native_64(), rcx);
        assembler.mov(temp_2.to_native_64(), out.to_native_64()).unwrap();
        //temp1 is region mask
        assembler.mov(temp_1.to_native_64(), u64::MAX).unwrap();
        assembler.shl(temp_1.to_native_64(), cl).unwrap();
        assembler.and(temp_3.to_native_64(), temp_1.to_native_64()).unwrap();
        assembler.mov(out.to_native_64(), temp_3.to_native_64()).unwrap();
    }

    fn generate_region_pointer_to_region_size(assembler: &mut CodeAssembler, ptr: Register, temp_1: Register, temp_2: Register, out: Register) {
        assert_eq!(temp_2.to_native_64(), rcx);
        assembler.sub(temp_1.to_native_64(), temp_1.to_native_64()).unwrap();
        assembler.sub(out.to_native_64(), out.to_native_64()).unwrap();
        // example::region_pointer_to_region_size_size:
        // shr     rdi, 43
        assembler.shr(ptr.to_native_64(), MAX_REGIONS_SIZE_SIZE as u32).unwrap();
        // add     edi, -1
        assembler.add(ptr.to_native_32(), -1).unwrap();
        // shr     edi
        assembler.shr(ptr.to_native_32(), 1).unwrap();
        // add     dil, 1
        assembler.add(ptr.to_native_8(), 1).unwrap();
        // mov     ecx, edi
        assembler.mov(ecx, ptr.to_native_32()).unwrap();
        // and     cl, 7
        assembler.and(cl, 7).unwrap();
        // mov     al, 1
        assembler.mov(out.to_native_8(), 1).unwrap();
        // mov     dl, 1
        assembler.mov(temp_1.to_native_8(), 1).unwrap();
        // shl     dl, cl
        assembler.shl(temp_1.to_native_8(), cl).unwrap();
        // shr     dil
        assembler.shr(ptr.to_native_8(), 1).unwrap();
        // and     dil, 1
        assembler.and(ptr.to_native_8(), 1).unwrap();
        // mov     ecx, edi
        assembler.mov(ecx, ptr.to_native_32()).unwrap();
        // shl     al, cl
        assembler.shl(out.to_native_8(), cl).unwrap();
        // add     al, dl
        assembler.add(out.to_native_8(), temp_1.to_native_8()).unwrap();
        // add     al, al
        assembler.add(out.to_native_8(), out.to_native_8()).unwrap();
    }

    //todo this lifetime is maybe not right
    pub fn find_object_region_header<'l>(ptr: NonNull<c_void>) -> &'l mut RegionHeader {
        let as_u64 = ptr.as_ptr() as u64;
        let region_size = region_pointer_to_region_size_size(as_u64);
        let region_mask = u64::MAX << region_size;
        let masked = as_u64 & region_mask;
        unsafe { (masked as *mut c_void as *mut RegionHeader).as_mut().unwrap() }
    }

    pub fn find_type_vtable(&self, ptr: NonNull<c_void>) -> Option<NonNull<RawNativeVTable>> {
        NonNull::new(MemoryRegions::find_object_region_header(ptr).vtable_ptr)
    }

    pub fn find_type_itable(&self, ptr: NonNull<c_void>) -> Option<NonNull<ITableRaw>> {
        NonNull::new(MemoryRegions::find_object_region_header(ptr).itable_ptr)
    }


    pub fn find_object_allocated_type(&self, ptr: NonNull<c_void>) -> &AllocatedObjectType {
        let header = MemoryRegions::find_object_region_header(ptr);
        let allocated_type_id = header.region_type;
        &self.types[allocated_type_id.0 as usize]
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
        } else if region_base_masked_ptr == self.early_mmaped_regions.extra_large_regions.as_ptr() as u64 {
            let region_mask = 1 << EXTRA_LARGE_REGION_SIZE_SIZE;
            BaseAddressAndMask {
                mask: region_mask,
                base_address: (ptr.as_ptr() as u64 & region_mask) as *mut c_void,
            }
        } else {
            dbg!(self.early_mmaped_regions.large_regions);
            dbg!(&self.early_mmaped_regions);
            dbg!(region_base_masked_ptr as *mut c_void);
            dbg!(ptr.as_ptr());
            todo!()
        }
    }
}

#[cfg(test)]
pub mod test {
    use crate::early_startup::get_regions;
    use crate::memory_regions::{AllocatedObjectType, MemoryRegions};

    #[test]
    pub fn allocate_small() {
        let mut memory_regions = MemoryRegions::new(get_regions());
        let size_1 = AllocatedObjectType::Raw { size: 1 };
        let size_2 = AllocatedObjectType::Raw { size: 8 };
        let res_1_1 = memory_regions.allocate(&size_1);
        let res_1_2 = memory_regions.allocate(&size_1);
        let res_8_1 = memory_regions.allocate(&size_2);
        let res_8_2 = memory_regions.allocate(&size_2);
        assert_eq!(memory_regions.find_object_allocated_type(res_1_1), &size_1);
        assert_eq!(memory_regions.find_object_allocated_type(res_8_1), &size_2);
        for _ in 0..1000 {
            let res_1 = memory_regions.allocate(&size_1);
            assert_eq!(memory_regions.find_object_allocated_type(res_1), &size_1);
        }
        let size_3 = AllocatedObjectType::Raw { size: 16 };
        for _ in 0..10000 {
            let res_1 = memory_regions.allocate(&size_3);
            unsafe { libc::memset(res_1.as_ptr(), 0, 16); }
            assert_eq!(memory_regions.find_object_allocated_type(res_1), &size_3);
        }
    }
}