use std::collections::{HashMap, HashSet};
use std::ffi::c_void;
use std::mem::size_of;

use itertools::{Either, Itertools};

use classfile_view::loading::LoaderName;
use jvmti_jni_bindings::jobject;
use verification::verifier::Frame;

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
    pub frame_info_ptr: *const FrameInfo,
    pub debug_ptr: *mut c_void,
    pub magic_part_1: u64,
    pub magic_part_2: u64,
}


#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct FramePointerOffset(pub usize);

pub trait StackframeMemoryLayout {
    fn local_var_entry(&self, pc: usize, i: usize) -> FramePointerOffset;
    fn operand_stack_entry(&self, pc: usize, from_end: usize) -> FramePointerOffset;
    fn operand_stack_entry_array_layout(&self, pc: usize, from_end: usize) -> ArrayMemoryLayout;
    fn operand_stack_entry_object_layout(&self, pc: usize, from_end: usize) -> ObjectMemoryLayout;
    fn full_frame_size(&self) -> usize;
    fn safe_temp_location(&self, pc: usize, i: usize) -> FramePointerOffset;
}

pub struct FrameBackedStackframeMemoryLayout {
    method_frames: HashMap<usize, Frame>,
    max_stack: usize,
    max_locals: usize,
}

impl FrameBackedStackframeMemoryLayout {
    pub fn new(max_stack: usize, max_locals: usize, frame_vtypes: HashMap<usize, Frame>) -> Self {
        Self {
            method_frames: frame_vtypes,
            max_stack,
            max_locals,
        }
    }
}

impl StackframeMemoryLayout for FrameBackedStackframeMemoryLayout {
    fn local_var_entry(&self, pc: usize, i: usize) -> FramePointerOffset {
        let locals = self.method_frames.get(&pc).unwrap().locals.clone();//todo this rc could cross threads
        FramePointerOffset(locals.iter().take(i).map(|_local_type| 8).sum())//for now everything is 8 bytes
    }

    fn operand_stack_entry(&self, pc: usize, from_end: usize) -> FramePointerOffset {
        let operand_stack = &self.method_frames.get(&pc).unwrap().stack_map.data;
        let len = operand_stack.len();
        let entry_idx = len - 1 - from_end;
        FramePointerOffset(self.max_locals * 8 + entry_idx)
    }

    fn operand_stack_entry_array_layout(&self, pc: usize, from_end: usize) -> ArrayMemoryLayout {
        todo!()
    }

    fn operand_stack_entry_object_layout(&self, pc: usize, from_end: usize) -> ObjectMemoryLayout {
        todo!()
    }

    fn full_frame_size(&self) -> usize {
        todo!()
    }

    fn safe_temp_location(&self, pc: usize, i: usize) -> FramePointerOffset {
        todo!()
    }
}


pub enum FrameInfo {
    FullyOpaque {
        loader: LoaderName
    },
    Native {
        runtime_class_id: usize,
        method_id: u16,
        loader: LoaderName,
        native_local_refs: Vec<HashSet<jobject>>,
    },
    JavaFrame {
        runtime_class_id: usize,
        method_id: u16,
        loader: LoaderName,
    },
}