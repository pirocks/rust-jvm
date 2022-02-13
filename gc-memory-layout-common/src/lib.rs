#![allow(dead_code)]
#![allow(unused_variables)]
#![feature(int_roundings)]
#![feature(box_syntax)]
#![feature(exclusive_range_pattern)]

use std::collections::{HashMap, HashSet};
use std::ffi::c_void;
use std::mem::size_of;
use std::ops::Range;
use std::ptr::{NonNull, null_mut};
use std::sync::atomic::{AtomicUsize, Ordering};

use itertools::{Either, Itertools};
use nix::sys::mman::{MapFlags, mmap, ProtFlags};
use num_integer::Integer;

use early_startup::{EXTRA_LARGE_REGION_SIZE, LARGE_REGION_SIZE, MEDIUM_REGION_SIZE, Regions, SMALL_REGION_SIZE, TERABYTE};
use jvmti_jni_bindings::{jint, jlong, jobject};
use rust_jvm_common::compressed_classfile::{CompressedParsedRefType, CPDType, CPRefType};
use rust_jvm_common::compressed_classfile::names::CClassName;
use rust_jvm_common::JavaThreadId;
use rust_jvm_common::loading::LoaderName;
use rust_jvm_common::runtime_type::RuntimeType;
use verification::verifier::Frame;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct FramePointerOffset(pub usize);

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum AllocatedObjectType {
    Class { thread: JavaThreadId, name: CClassName, loader: LoaderName, size: usize },
    ObjectArray { thread: JavaThreadId, sub_type: CPRefType, sub_type_loader: LoaderName, len: i32 },
    PrimitiveArray { thread: JavaThreadId, primitive_type: CPDType, len: i32 },
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
                let res = *len as usize * size_of::<jobject>() + size_of::<jint>();
                if res == 0 {
                    return 1;
                } else {
                    res
                }
            }
            AllocatedObjectType::PrimitiveArray { len, primitive_type, .. } => {
                if *len == 0 {
                    return 1;
                } else {
                    *len as usize * size_of::<jlong>() + size_of::<jlong>()/*match primitive_type {
                        CPDType::BooleanType => 1,
                        CPDType::ByteType => 1,
                        CPDType::ShortType => 2,
                        CPDType::CharType => 2,
                        CPDType::IntType => 4,
                        CPDType::LongType => 8,
                        CPDType::FloatType => 4,
                        CPDType::DoubleType => 8,
                        CPDType::VoidType => panic!(),
                        CPDType::Ref(_) => panic!(),
                    } + size_of::<jint>()*/
                }
            }
        }
    }

    pub fn as_cpdtype(&self) -> CPDType{
        match self {
            AllocatedObjectType::Class { name, .. } => {
                (*name).into()
            }
            AllocatedObjectType::ObjectArray { sub_type, .. } => {
                CPDType::Ref(CompressedParsedRefType::Array(box CPDType::Ref(sub_type.clone())))
            }
            AllocatedObjectType::PrimitiveArray { primitive_type, .. } => {
                CPDType::Ref(CompressedParsedRefType::Array(box primitive_type.clone()))
            }
        }
    }
}

#[repr(C)]
pub struct RegionData {
    pub region_base: *mut c_void,
    pub used_bitmap: *mut c_void,
    pub num_current_elements: AtomicUsize,
    pub region_type: AllocatedTypeID,
    pub region_elem_size: usize,
    pub region_max_elements: usize,
}

impl RegionData {
    pub fn get_allocation(&mut self) -> NonNull<c_void> {
        let region_base = self.region_base;
        let current_index = self.num_current_elements.fetch_add(1, Ordering::SeqCst);
        let res = unsafe { region_base.offset((current_index * self.region_elem_size) as isize) };
        unsafe { libc::memset(res, 0, self.region_elem_size); }
        if res as usize == 0x180000025000{
            eprintln!("ygi")
        }
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
                self.types_reverse.insert(type_.clone(),new_id);
                new_id
            }
            Some(cur_id) => *cur_id,
        }
    }

    pub fn find_or_new_region_for(&mut self, to_allocate_type: AllocatedObjectType) -> &mut RegionData {
        //todo doesn't actually find region
        let type_id = self.lookup_or_add_type(&to_allocate_type);
        let current_region_to_use = &self.current_region_type[type_id.0 as usize];
        let to_push_to = match current_region_to_use {
            RegionToUse::Small => &mut self.small_region_types,
            RegionToUse::Medium => &mut self.medium_region_types,
            RegionToUse::Large => &mut self.large_region_types,
            RegionToUse::ExtraLarge => &mut self.extra_large_region_types,
        };
        let new_region_base = match to_push_to.last() {
            Some(x) => unsafe { x.region_base.offset(current_region_to_use.region_size() as isize) },
            None => current_region_to_use.region_base(&self.early_mmaped_regions),
        };
        unsafe {
            let bitmap_length = ((current_region_to_use.region_size()) / (to_allocate_type.size())).div_ceil(&8);
            assert_ne!(bitmap_length, 0);
            to_push_to.push(RegionData {
                region_base: new_region_base,
                used_bitmap: mmap(null_mut(), bitmap_length, ProtFlags::PROT_WRITE | ProtFlags::PROT_READ, MapFlags::MAP_ANONYMOUS | MapFlags::MAP_PRIVATE, -1, 0).unwrap(),
                num_current_elements: AtomicUsize::new(0),
                region_type: type_id,
                region_elem_size: to_allocate_type.size(),
                region_max_elements: current_region_to_use.region_size() / to_allocate_type.size(),
            })
        }
        to_push_to.last_mut().unwrap()
    }

    pub fn find_object_allocated_type(&self, ptr: NonNull<c_void>) -> &AllocatedObjectType {
        let num_zeros = (8 * TERABYTE).trailing_zeros();
        let mask = !(!0u64 << num_zeros);
        let region_base_masked_ptr = ptr.as_ptr() as u64 & !mask;
        let region_type = if region_base_masked_ptr == self.early_mmaped_regions.small_regions as u64 {
            let region_index = ((ptr.as_ptr() as u64 & mask) / SMALL_REGION_SIZE as u64) as usize;
            let region_data = &self.small_region_types[region_index];
            region_data.region_type
        } else if region_base_masked_ptr == self.early_mmaped_regions.medium_regions as u64 {
            let region_index = ((ptr.as_ptr() as u64 & mask) / MEDIUM_REGION_SIZE as u64) as usize;
            let region_data = &self.medium_region_types[region_index];
            region_data.region_type
        } else {
            dbg!(self.early_mmaped_regions.large_regions);
            dbg!(&self.early_mmaped_regions);
            dbg!(region_base_masked_ptr as *mut c_void);
            dbg!(ptr.as_ptr());
            todo!()
        };
        &self.types[region_type.0 as usize]
    }

    fn region(to_allocate_type: AllocatedObjectType, to_use: RegionToUse) -> (RegionData, Range<NonNull<c_void>>) {
        // let object_size: usize = to_allocate_type.size();
        // let read_and_write = ProtFlags::PROT_READ | ProtFlags::PROT_WRITE;
        // let map_flags = MapFlags::MAP_NORESERVE | MapFlags::MAP_ANONYMOUS | MapFlags::MAP_PRIVATE;
        // let region_len = object_size * region_max_elements;
        // let ptr = unsafe { mmap(null_mut(), region_len, read_and_write, map_flags, -1, 0).unwrap() };
        // let used_bitmap = unsafe { mmap(null_mut(), region_max_elements.div_ceil(&size_of::<u8>()), read_and_write, map_flags, -1, 0).unwrap() };
        // unsafe { libc::memset(used_bitmap, 0, region_len) };
        // unsafe {
        //     (RegionData {
        //         ptr,
        //         used_bitmap,
        //         free_search_index: 0,
        //         current_elements_count: 0,
        //         region_type: to_allocate_type,
        //         region_max_elements: 1,
        //     }, NonNull::new(ptr).unwrap()..NonNull::new(ptr.offset(region_len as isize)).unwrap())
        // }
        todo!()
    }

    fn push_new_region(&mut self, to_allocated_type: AllocatedObjectType) {
        /*let regions = self.regions.get_mut(&to_allocated_type).unwrap();
        let RegionData { region_max_elements, .. } = regions.last_mut().unwrap().get_mut();
        let new_region_max_elements = *region_max_elements * 2;
        let (new_region, range) = Self::region(to_allocated_type.clone(), todo!());
        self.ptr_to_object_type.insert(range, to_allocated_type.clone());
        regions.push(Box::pin(UnsafeCell::new(new_region)));*/
        todo!()
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
        match size {
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

    pub fn region_base(&self, regions: &Regions) -> *mut c_void {
        match self {
            RegionToUse::Small => regions.small_regions,
            RegionToUse::Medium => regions.medium_regions,
            RegionToUse::Large => regions.large_regions,
            RegionToUse::ExtraLarge => regions.extra_large_regions,
        }
    }
}

pub struct GCState {
    roots: HashMap<*mut c_void, PointerMemoryLayout>,
    live_pointers: HashMap<*mut c_void, PointerMemoryLayout>,
}

impl GCState {
    pub fn allocate(&mut self, layout: PointerMemoryLayout) -> *mut c_void {
        let total_size = layout.total_size();
        let res: *mut c_void = unsafe { libc::malloc(total_size * size_of::<u8>()) };
        assert!(!self.live_pointers.contains_key(&res));
        self.live_pointers.insert(res, layout);
        res
    }

    unsafe fn free(&mut self, pointer: *mut c_void) {
        assert!(self.live_pointers.contains_key(&pointer));
        self.live_pointers.remove(&pointer);
        libc::free(pointer);
    }

    pub fn register_root(&mut self, root: *mut c_void) {
        // self.roots.insert(root, todo!());
        todo!()
    }

    pub fn gc(&mut self) {
        let mut touched_pointers: HashSet<*mut c_void> = HashSet::new();
        for (root, layout) in &self.roots {
            unsafe {
                self.gc_impl(*root, layout, &mut touched_pointers);
            }
        }
        let (new_live_pointers, to_free): (Vec<(_, _)>, Vec<_>) = self.live_pointers.iter()
            .partition_map(|(pointer, layout)| {
                if touched_pointers.contains(pointer) { Either::Left((*pointer, layout.clone())) } else { Either::Right(pointer) }
            });
        self.live_pointers = new_live_pointers.into_iter().collect::<HashMap<*mut c_void, PointerMemoryLayout>>();
        for to_free_pointer in to_free {
            unsafe { self.free(to_free_pointer) }
        }
    }

    unsafe fn gc_impl(&self, pointer: *mut c_void, layout: &PointerMemoryLayout, touched_pointers: &mut HashSet<*mut c_void>) {
        for offset in layout.get_gc_pointer_offsets() {
            let new_pointer = (pointer.offset(offset as isize) as *mut *mut c_void).read();
            if !touched_pointers.contains(&new_pointer) {
                touched_pointers.insert(new_pointer);
                let new_layout = self.live_pointers.get(&new_pointer).expect("GC is in broken state");
                self.gc_impl(new_pointer, &new_layout, touched_pointers)
            }
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub enum PointerMemoryLayout {}

impl PointerMemoryLayout {
    pub fn get_gc_pointer_offsets(&self) -> Vec<usize> {
        todo!()
    }

    pub fn as_object(&self) -> ObjectMemoryLayout {
        todo!()
    }

    pub fn as_array(&self) -> ArrayMemoryLayout {
        todo!()
    }

    pub fn monitor_entry(&self) -> usize {
        todo!()
    }

    pub fn class_pointer_entry(&self) -> usize {
        todo!()
    }

    pub fn total_size(&self) -> usize {
        todo!()
    }
}

pub struct ObjectMemoryLayout {
    elems: HashMap<usize /*filed id*/, usize>,
}

impl ObjectMemoryLayout {
    pub fn field_entry(&self) -> usize {
        todo!()
    }
}

pub struct ArrayMemoryLayout {}

impl ArrayMemoryLayout {
    pub fn elem_0_entry(&self) -> usize {
        todo!()
    }
    pub fn len_entry(&self) -> usize {
        todo!()
    }
    pub fn elem_size(&self) -> usize {
        todo!()
    }
}

pub const MAGIC_1_EXPECTED: u64 = 0xDEADBEEFDEADBEAF;
pub const MAGIC_2_EXPECTED: u64 = 0xDEADCAFEDEADDEAD;

//todo frane info will need to be reworked to be based of rip
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct FrameHeader {
    pub prev_rip: *mut c_void,
    pub prev_rpb: *mut c_void,
    pub frame_info_ptr: *mut FrameInfo,
    pub methodid: usize,
    pub magic_part_1: u64,
    pub magic_part_2: u64,
}


pub trait StackframeMemoryLayout {
    fn local_var_entry(&self, pc: u16, i: u16) -> FramePointerOffset;
    fn operand_stack_entry(&self, pc: u16, from_end: u16) -> FramePointerOffset;
    fn operand_stack_entry_array_layout(&self, pc: u16, from_end: u16) -> ArrayMemoryLayout;
    fn operand_stack_entry_object_layout(&self, pc: u16, from_end: u16) -> ObjectMemoryLayout;
    fn full_frame_size(&self) -> usize;
    fn safe_temp_location(&self, pc: u16, i: u16) -> FramePointerOffset;
}

pub struct FrameBackedStackframeMemoryLayout {
    method_frames: HashMap<u16, Frame>,
    max_stack: usize,
    max_locals: usize,
}

impl FrameBackedStackframeMemoryLayout {
    pub fn new(max_stack: usize, max_locals: usize, frame_vtypes: HashMap<u16, Frame>) -> Self {
        Self { method_frames: frame_vtypes, max_stack, max_locals }
    }
}

impl StackframeMemoryLayout for FrameBackedStackframeMemoryLayout {
    fn local_var_entry(&self, pc: u16, i: u16) -> FramePointerOffset {
        let locals = self.method_frames.get(&pc).unwrap().locals.clone(); //todo this rc could cross threads
        FramePointerOffset(locals.iter().take(i as usize).map(|_local_type| 8).sum())
        //for now everything is 8 bytes
    }

    fn operand_stack_entry(&self, pc: u16, from_end: u16) -> FramePointerOffset {
        let operand_stack = &self.method_frames.get(&pc).unwrap().stack_map.data;
        let len = operand_stack.len();
        let entry_idx = len - 1 - from_end as usize;
        FramePointerOffset(self.max_locals * 8 + entry_idx)
    }

    fn operand_stack_entry_array_layout(&self, pc: u16, from_end: u16) -> ArrayMemoryLayout {
        todo!()
    }

    fn operand_stack_entry_object_layout(&self, pc: u16, from_end: u16) -> ObjectMemoryLayout {
        todo!()
    }

    fn full_frame_size(&self) -> usize {
        size_of::<FrameHeader>() + (self.max_locals + self.max_stack + 1) * size_of::<jlong>()
    }

    fn safe_temp_location(&self, pc: u16, i: u16) -> FramePointerOffset {
        todo!()
    }
}

#[derive(Debug)]
pub enum FrameInfo {
    FullyOpaque {
        loader: LoaderName,
        operand_stack_depth: u16,
        operand_stack_types: Vec<RuntimeType>,
    },
    Native {
        method_id: usize,
        loader: LoaderName,
        operand_stack_depth: u16,
        native_local_refs: Vec<HashSet<jobject>>,
        operand_stack_types: Vec<RuntimeType>,
    },
    JavaFrame {
        method_id: usize,
        num_locals: u16,
        loader: LoaderName,
        java_pc: u16,
        pc_offset: i32,
        operand_stack_depth: u16,
        operand_stack_types: Vec<RuntimeType>,
        locals_types: Vec<RuntimeType>,
    },
}

impl FrameInfo {
    pub fn operand_stack_depth_mut(&mut self) -> &mut u16 {
        match self {
            FrameInfo::FullyOpaque { operand_stack_depth, .. } => operand_stack_depth,
            FrameInfo::Native { operand_stack_depth, .. } => operand_stack_depth,
            FrameInfo::JavaFrame { operand_stack_depth, .. } => todo!(),
        }
    }

    pub fn push_operand_stack(&mut self, ptype: RuntimeType) {
        match self {
            FrameInfo::FullyOpaque { operand_stack_types, .. } => operand_stack_types,
            FrameInfo::Native { operand_stack_types, .. } => operand_stack_types,
            FrameInfo::JavaFrame { operand_stack_types, .. } => operand_stack_types,
        }
            .push(ptype);
    }

    pub fn pop_operand_stack(&mut self) -> Option<RuntimeType> {
        match self {
            FrameInfo::FullyOpaque { operand_stack_types, .. } => operand_stack_types,
            FrameInfo::Native { operand_stack_types, .. } => operand_stack_types,
            FrameInfo::JavaFrame { operand_stack_types, .. } => operand_stack_types,
        }
            .pop()
    }

    pub fn set_local_var_type(&mut self, ptype: RuntimeType, i: usize) {
        match self {
            FrameInfo::FullyOpaque { .. } => panic!(),
            FrameInfo::Native { .. } => panic!(),
            FrameInfo::JavaFrame { locals_types, .. } => locals_types[i] = ptype,
        };
    }

    pub fn operand_stack_types(&self) -> Vec<RuntimeType> {
        match self {
            FrameInfo::FullyOpaque { operand_stack_types, .. } => operand_stack_types.clone(),
            FrameInfo::Native { operand_stack_types, .. } => operand_stack_types.clone(),
            FrameInfo::JavaFrame { operand_stack_types, .. } => operand_stack_types.clone(),
        }
    }

    pub fn methodid(&self) -> Option<usize> {
        match self {
            FrameInfo::FullyOpaque { .. } => None,
            FrameInfo::Native { method_id, .. } => Some(*method_id),
            FrameInfo::JavaFrame { method_id, .. } => Some(*method_id),
        }
    }
}

const MAX_OPERAND_STACK_NEEDED_FOR_FUNCTION_INVOCATION: usize = 256 * size_of::<jlong>();

pub struct FullyOpaqueFrame {
    pub max_stack: usize,
    pub max_frame: usize,
}

impl StackframeMemoryLayout for FullyOpaqueFrame {
    fn local_var_entry(&self, pc: u16, i: u16) -> FramePointerOffset {
        todo!()
    }

    fn operand_stack_entry(&self, pc: u16, from_end: u16) -> FramePointerOffset {
        todo!()
    }

    fn operand_stack_entry_array_layout(&self, pc: u16, from_end: u16) -> ArrayMemoryLayout {
        todo!()
    }

    fn operand_stack_entry_object_layout(&self, pc: u16, from_end: u16) -> ObjectMemoryLayout {
        todo!()
    }

    fn full_frame_size(&self) -> usize {
        size_of::<FrameHeader>() + MAX_OPERAND_STACK_NEEDED_FOR_FUNCTION_INVOCATION + size_of::<jlong>()
    }

    fn safe_temp_location(&self, pc: u16, i: u16) -> FramePointerOffset {
        todo!()
    }
}

pub struct NativeStackframeMemoryLayout {}

impl StackframeMemoryLayout for NativeStackframeMemoryLayout {
    fn local_var_entry(&self, pc: u16, i: u16) -> FramePointerOffset {
        todo!()
    }

    fn operand_stack_entry(&self, pc: u16, from_end: u16) -> FramePointerOffset {
        todo!()
    }

    fn operand_stack_entry_array_layout(&self, pc: u16, from_end: u16) -> ArrayMemoryLayout {
        todo!()
    }

    fn operand_stack_entry_object_layout(&self, pc: u16, from_end: u16) -> ObjectMemoryLayout {
        todo!()
    }

    fn full_frame_size(&self) -> usize {
        size_of::<FrameHeader>() + MAX_OPERAND_STACK_NEEDED_FOR_FUNCTION_INVOCATION + size_of::<jlong>()
    }

    fn safe_temp_location(&self, pc: u16, i: u16) -> FramePointerOffset {
        todo!()
    }
}