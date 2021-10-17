#![allow(dead_code)]
#![allow(unused_variables)]
#![feature(destructuring_assignment)]
#![feature(int_roundings)]
#![feature(box_syntax)]

use std::cell::UnsafeCell;
use std::collections::{HashMap, HashSet};
use std::ffi::c_void;
use std::mem::size_of;
use std::ops::Range;
use std::pin::Pin;
use std::ptr::{NonNull, null_mut};
use std::sync::{Mutex, MutexGuard};
use std::thread::ThreadId;

use itertools::{Either, Itertools};
use lazy_static::lazy_static;
use nix::sys::mman::{MapFlags, mmap, ProtFlags};
use num_integer::Integer;
use rangemap::RangeMap;

use early_startup::Regions;
use jvmti_jni_bindings::{jbyte, jint, jlong, jobject};
use rust_jvm_common::compressed_classfile::{CPDType, CPRefType};
use rust_jvm_common::compressed_classfile::names::CClassName;
use rust_jvm_common::loading::LoaderName;
use rust_jvm_common::runtime_type::RuntimeType;
use verification::verifier::Frame;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum AllocatedObjectType {
    Class {
        name: CClassName,
        loader: LoaderName,
        size: usize,
    },
    ObjectArray {
        sub_type: CPRefType,
        sub_type_loader: LoaderName,
        len: usize,
    },
    PrimitiveArray {
        primitive_type: CPDType,
        len: usize,
    },
}

impl AllocatedObjectType {
    pub fn size(&self) -> usize {
        match self {
            AllocatedObjectType::Class { size, name, loader } => {
                *size
            }
            AllocatedObjectType::ObjectArray { sub_type, sub_type_loader, len } => {
                *len * size_of::<jobject>() + size_of::<jint>()
            }
            AllocatedObjectType::PrimitiveArray { len, primitive_type } => {
                *len * match primitive_type {
                    CPDType::BooleanType => 1,
                    CPDType::ByteType => 1,
                    CPDType::ShortType => 2,
                    CPDType::CharType => 2,
                    CPDType::IntType => 4,
                    CPDType::LongType => 8,
                    CPDType::FloatType => 4,
                    CPDType::DoubleType => 8,
                    CPDType::VoidType => panic!(),
                    CPDType::Ref(_) => panic!()
                } + size_of::<jint>()
            }
        }
    }
}

#[repr(packed, C)]
pub struct RegionData {
    pub ptr: *mut c_void,
    pub used_bitmap: *mut c_void,
    pub free_search_index: usize,
    pub current_elements_count: usize,
    pub region_type: AllocatedObjectType,
    pub region_max_elements: usize,
}

impl RegionData {
    pub fn get_allocation(&mut self) -> NonNull<c_void> {
        let res = self.ptr;
        self.current_elements_count += 1;
        NonNull::new(res).unwrap()
    }
}


unsafe impl Send for RegionData {}

unsafe impl Send for MemoryRegions {}


//work around thread locals requiring Sync, when not actually required
unsafe impl Sync for MemoryRegions {}

pub struct MemoryRegions {
    early_mmaped_regions: Regions,
    regions: HashMap<AllocatedObjectType, Vec<Pin<Box<UnsafeCell<RegionData>>>>>,
    ptr_to_object_type: RangeMap<NonNull<c_void>, AllocatedObjectType>,
}

impl MemoryRegions {
    pub fn new(regions: Regions) -> MemoryRegions {
        MemoryRegions { early_mmaped_regions: regions, regions: HashMap::new(), ptr_to_object_type: RangeMap::new() }
    }

    pub fn find_or_new_region_for(&mut self, to_allocate_type: AllocatedObjectType, expected_new_region: Option<bool>) -> Pin<&mut UnsafeCell<RegionData>> {
        let mut new_allocated_range = None;
        let current_region = self.regions.entry(to_allocate_type.clone()).or_insert_with(|| {
            let (region_data, range) = MemoryRegions::region(to_allocate_type.clone(), 1);
            vec![Pin::new(box UnsafeCell::new(region_data))]
        }).last_mut().unwrap();
        if let Some(new_allocated_range) = new_allocated_range {
            self.ptr_to_object_type.insert(new_allocated_range, to_allocate_type.clone());
        }
        let region_as_bytes = current_region.get_mut().used_bitmap as *const u8;
        let mut all_in_use = true;
        for i in 0..current_region.get_mut().region_max_elements {
            let byte = unsafe { region_as_bytes.offset(i.div_floor(&8) as isize).read() };
            let current_bit = (byte >> (i % 8)) & 0b1;
            all_in_use |= current_bit == 1;
        }
        if let Some(expected_new_region) = expected_new_region {
            assert_eq!(expected_new_region, all_in_use);
        }
        if all_in_use {
            Self::push_new_region(self, to_allocate_type.clone());
        }
        self.regions.get_mut(&to_allocate_type).unwrap().last_mut().unwrap().as_mut()
    }

    pub fn find_object_allocated_type(&self, ptr: NonNull<c_void>) -> &AllocatedObjectType {
        self.ptr_to_object_type.get(&ptr).unwrap()
    }

    fn region(to_allocate_type: AllocatedObjectType, region_max_elements: usize) -> (RegionData, Range<NonNull<c_void>>) {
        let object_size: usize = to_allocate_type.size();
        let read_and_write = ProtFlags::PROT_READ | ProtFlags::PROT_WRITE;
        let map_flags = MapFlags::MAP_NORESERVE | MapFlags::MAP_ANONYMOUS | MapFlags::MAP_PRIVATE;
        let region_len = object_size * region_max_elements;
        let ptr = unsafe { mmap(null_mut(), region_len, read_and_write, map_flags, -1, 0).unwrap() };
        let used_bitmap = unsafe { mmap(null_mut(), region_max_elements.div_ceil(&size_of::<u8>()), read_and_write, map_flags, -1, 0).unwrap() };
        unsafe { libc::memset(used_bitmap, 0, region_len) };
        unsafe {
            (RegionData {
                ptr,
                used_bitmap,
                free_search_index: 0,
                current_elements_count: 0,
                region_type: to_allocate_type,
                region_max_elements: 1,
            }, NonNull::new(ptr).unwrap()..NonNull::new(ptr.offset(region_len as isize)).unwrap())
        }
    }

    fn push_new_region(&mut self, to_allocated_type: AllocatedObjectType) {
        let regions = self.regions.get_mut(&to_allocated_type).unwrap();
        let RegionData { region_max_elements, .. } = regions.last_mut().unwrap().get_mut();
        let new_region_max_elements = *region_max_elements * 2;
        let (new_region, range) = Self::region(to_allocated_type.clone(), new_region_max_elements);
        self.ptr_to_object_type.insert(range, to_allocated_type.clone());
        regions.push(Box::pin(UnsafeCell::new(new_region)));
    }
}


pub struct GCState {
    roots: HashMap<*mut c_void, PointerMemoryLayout>,
    live_pointers: HashMap<*mut c_void, PointerMemoryLayout>,
}

impl GCState {
    pub fn allocate(&mut self, layout: PointerMemoryLayout) -> *mut c_void {
        let total_size = layout.total_size();
        let res: *mut c_void = unsafe {
            libc::malloc(total_size * size_of::<u8>())
        };
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
        self.roots.insert(root, todo!());
    }

    pub fn gc(&mut self) {
        let mut touched_pointers: HashSet<*mut c_void> = HashSet::new();
        for (root, layout) in &self.roots {
            unsafe { self.gc_impl(*root, layout, &mut touched_pointers); }
        }
        let (new_live_pointers, to_free): (Vec<(_, _)>, Vec<_>) = self.live_pointers.iter().partition_map(|(pointer, layout)| {
            if touched_pointers.contains(pointer) {
                Either::Left((*pointer, layout.clone()))
            } else {
                Either::Right(pointer)
            }
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
                self.gc_impl(new_pointer, &new_layout.clone(), touched_pointers)
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
    elems: HashMap<usize/*filed id*/, usize>,
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
    pub debug_ptr: *mut c_void,
    pub magic_part_1: u64,
    pub magic_part_2: u64,
}


#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct FramePointerOffset(pub usize);

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
        Self {
            method_frames: frame_vtypes,
            max_stack,
            max_locals,
        }
    }
}

impl StackframeMemoryLayout for FrameBackedStackframeMemoryLayout {
    fn local_var_entry(&self, pc: u16, i: u16) -> FramePointerOffset {
        let locals = self.method_frames.get(&pc).unwrap().locals.clone();//todo this rc could cross threads
        FramePointerOffset(locals.iter().take(i as usize).map(|_local_type| 8).sum())//for now everything is 8 bytes
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
        size_of::<FrameHeader>() + self.max_stack * size_of::<jlong>() + self.max_locals * size_of::<jlong>()
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
            FrameInfo::JavaFrame { operand_stack_depth, .. } => operand_stack_depth,
        }
    }

    pub fn push_operand_stack(&mut self, ptype: RuntimeType) {
        match self {
            FrameInfo::FullyOpaque { operand_stack_types, .. } => operand_stack_types,
            FrameInfo::Native { operand_stack_types, .. } => operand_stack_types,
            FrameInfo::JavaFrame { operand_stack_types, .. } => operand_stack_types,
        }.push(ptype);
    }

    pub fn pop_operand_stack(&mut self) -> Option<RuntimeType> {
        match self {
            FrameInfo::FullyOpaque { operand_stack_types, .. } => operand_stack_types,
            FrameInfo::Native { operand_stack_types, .. } => operand_stack_types,
            FrameInfo::JavaFrame { operand_stack_types, .. } => operand_stack_types,
        }.pop()
    }

    pub fn set_local_var_type(&mut self, ptype: RuntimeType, i: usize) {
        match self {
            FrameInfo::FullyOpaque { .. } => panic!(),
            FrameInfo::Native { .. } => panic!(),
            FrameInfo::JavaFrame { locals_types, .. } => { locals_types[i] = ptype }
        };
    }

    pub fn operand_stack_types(&self) -> Vec<RuntimeType> {
        match self {
            FrameInfo::FullyOpaque { operand_stack_types, .. } => operand_stack_types.clone(),
            FrameInfo::Native { operand_stack_types, .. } => operand_stack_types.clone(),
            FrameInfo::JavaFrame { operand_stack_types, .. } => operand_stack_types.clone()
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
