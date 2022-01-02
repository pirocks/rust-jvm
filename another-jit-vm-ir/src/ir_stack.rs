use std::ffi::c_void;
use std::ptr::null_mut;
use libc::{MAP_ANONYMOUS, MAP_GROWSDOWN, MAP_NORESERVE, MAP_PRIVATE, PROT_READ, PROT_WRITE};
use gc_memory_layout_common::{FramePointerOffset, MAGIC_1_EXPECTED, MAGIC_2_EXPECTED};
use rust_jvm_common::MethodId;
use std::mem::size_of;
use std::slice::from_raw_parts;
use crate::{IRMethodID, IRVMState};

// IR knows about stack so we should have a stack
// will have IR instruct for new frame, so IR also knows about frames
pub struct OwnedIRStack {
    pub mmaped_top: *mut c_void,
    pub(crate) mmaped_bottom: *mut c_void,
    max_stack: usize,
}

impl OwnedIRStack {
    pub fn new() -> Self {
        pub const MAX_STACK: usize = 1024 * 1024 * 1024;
        let mmaped_top = unsafe { libc::mmap(null_mut(), MAX_STACK, PROT_READ | PROT_WRITE, MAP_NORESERVE | MAP_PRIVATE | MAP_ANONYMOUS | MAP_GROWSDOWN, -1, 0) };
        unsafe {
            let page_size = 4096;
            Self {
                mmaped_top: mmaped_top.add(page_size),
                mmaped_bottom: mmaped_top.sub(MAX_STACK),
                max_stack: MAX_STACK,
            }
        }
    }


    pub unsafe fn frame_at(&'l self, frame_pointer: *const c_void) -> IRFrameRef<'l> {
        self.validate_frame_pointer(frame_pointer);
        let _frame_header = read_frame_ir_header(frame_pointer);
        IRFrameRef {
            ptr: frame_pointer,
            ir_stack: self,
        }
    }

    pub unsafe fn frame_at_mut(&'l mut self, frame_pointer: *mut c_void) -> IRFrameMut<'l> {
        self.validate_frame_pointer(frame_pointer);
        let _frame_header = read_frame_ir_header(frame_pointer);
        IRFrameMut {
            ptr: frame_pointer,
            ir_stack: self,
        }
    }

    pub unsafe fn frame_iter<ExtraData>(&self, start_frame: *mut c_void, ir_vm_state: &'vm_life IRVMState<'vm_life, ExtraData>) -> IRFrameIter<'_, 'vm_life, ExtraData> {
        IRFrameIter {
            ir_stack: self,
            current_frame_ptr: Some(start_frame),
            ir_vm_state,
        }
    }

    pub unsafe fn validate_frame_pointer(&self, frame_pointer: *const c_void) {
        if self.mmaped_top.offset_from(frame_pointer) > self.max_stack as isize || frame_pointer > self.mmaped_top {
            dbg!(self.mmaped_top);
            dbg!(frame_pointer);
            panic!()
        }
    }

    pub unsafe fn write_frame(&self, frame_pointer: *mut c_void, prev_rip: *const c_void, prev_rbp: *mut c_void, ir_method_id: Option<IRMethodID>, method_id: Option<MethodId>, data: &[u64]) {
        self.validate_frame_pointer(frame_pointer);
        let prev_rip_ptr = frame_pointer.sub(FRAME_HEADER_PREV_RIP_OFFSET) as *mut *const c_void;
        prev_rip_ptr.write(prev_rip);
        let prev_rpb_ptr = frame_pointer.sub(FRAME_HEADER_PREV_RBP_OFFSET) as *mut *mut c_void;
        prev_rpb_ptr.write(prev_rbp);
        let magic_1_ptr = frame_pointer.sub(FRAME_HEADER_PREV_MAGIC_1_OFFSET) as *mut u64;
        magic_1_ptr.write(MAGIC_1_EXPECTED);
        let magic_2_ptr = frame_pointer.sub(FRAME_HEADER_PREV_MAGIC_2_OFFSET) as *mut u64;
        magic_2_ptr.write(MAGIC_2_EXPECTED);
        let ir_method_id_ptr = frame_pointer.sub(FRAME_HEADER_IR_METHOD_ID_OFFSET) as *mut u64;
        ir_method_id_ptr.write(ir_method_id.unwrap_or(IRMethodID(usize::MAX)).0 as u64);
        let method_id_ptr = frame_pointer.sub(FRAME_HEADER_METHOD_ID_OFFSET) as *mut u64;
        method_id_ptr.write(method_id.unwrap_or((-1isize) as usize) as u64);
        for (i, data_elem) in data.iter().cloned().enumerate() {
            let data_elem_ptr = frame_pointer.sub(FRAME_HEADER_END_OFFSET).sub(i) as *mut u64;
            data_elem_ptr.write(data_elem)
        }
    }
}

// has ref b/c not valid to access this after top level stack has been modified
pub struct IRFrameIter<'l, 'vm_life, ExtraData: 'vm_life> {
    ir_stack: &'l OwnedIRStack,
    current_frame_ptr: Option<*mut c_void>,
    ir_vm_state: &'vm_life IRVMState<'vm_life, ExtraData>,
}

impl<'l, 'vm_life, ExtraData: 'vm_life> Iterator for IRFrameIter<'l, 'vm_life, ExtraData> {
    type Item = IRFrameRef<'l>;

    fn next(&mut self) -> Option<Self::Item> {
        let res = unsafe { self.ir_stack.frame_at(self.current_frame_ptr?) };
        unsafe {
            if self.current_frame_ptr? == self.ir_stack.mmaped_top {
                self.current_frame_ptr = None;
            } else {
                let option = self.ir_stack.frame_at(res.prev_rbp()).ir_method_id();
                let new_current_frame_size = *self.ir_vm_state.inner.read().unwrap().frame_sizes_by_ir_method_id.get(&option.unwrap()).unwrap();
                assert_eq!(res.prev_rbp().offset_from(self.current_frame_ptr.unwrap()) as usize, new_current_frame_size);
                self.current_frame_ptr = Some(res.prev_rbp());
            }
        }
        Some(res)
    }
}

// has ref b/c not valid to access this after top level stack has been modified
pub struct IRFrameRef<'l> {
    ptr: *const c_void,
    ir_stack: &'l OwnedIRStack,
}

impl IRFrameRef<'_> {
    pub fn header(&self) -> UnPackedIRFrameHeader {
        unsafe { read_frame_ir_header(self.ptr) }
    }

    pub fn read_at_offset(&self, offset: FramePointerOffset) -> u64 {
        unsafe { (self.ptr.offset(-(offset.0 as isize)) as *mut u64).read() }
    }


    pub fn ir_method_id(&self) -> Option<IRMethodID> {
        let res = self.read_at_offset(FramePointerOffset(FRAME_HEADER_IR_METHOD_ID_OFFSET));
        let frame_header = unsafe { read_frame_ir_header(self.ptr) };
        assert_eq!(res, frame_header.ir_method_id.0 as u64);
        if res == u64::MAX {
            return None;
        }
        Some(IRMethodID(res as usize))
    }


    pub fn method_id(&self) -> Option<MethodId> {
        let res = self.read_at_offset(FramePointerOffset(FRAME_HEADER_METHOD_ID_OFFSET));
        if res as i64 == -1i64 {
            return None;
        }
        let frame_header = unsafe { read_frame_ir_header(self.ptr) };
        assert_eq!(res, frame_header.method_id_ignored);
        Some(res as usize)
    }

    pub fn prev_rbp(&self) -> *mut c_void {
        let res = self.read_at_offset(FramePointerOffset(FRAME_HEADER_PREV_RBP_OFFSET));
        let frame_header = unsafe { read_frame_ir_header(self.ptr) };
        assert_eq!(res, frame_header.prev_rbp as u64);
        res as *mut c_void
    }

    pub fn frame_size<ExtraData>(&self, ir_vm_state: &IRVMState<ExtraData>) -> usize {
        let ir_method_id = match self.ir_method_id() {
            Some(x) => x,
            None => {
                //todo this is scuffed
                //frame header size + one data pointer for native frame data
                return FRAME_HEADER_END_OFFSET + 1*size_of::<*const c_void>()
            },
        };
        *ir_vm_state.inner.read().unwrap().frame_sizes_by_ir_method_id.get(&ir_method_id).unwrap()
    }

    pub fn data(&self, amount: usize) -> &[u64] {
        let data_raw_ptr = unsafe { self.ptr.sub(FRAME_HEADER_END_OFFSET) as *const u64 };
        unsafe { from_raw_parts(data_raw_ptr, amount) }
    }

    pub fn frame_ptr(&self) -> *const c_void {
        self.ptr
    }
}

// has ref b/c not valid to access this after top level stack has been modified
pub struct IRFrameMut<'l> {
    ptr: *mut c_void,
    ir_stack: &'l mut OwnedIRStack,
}

impl<'l> IRFrameMut<'l> {
    pub fn downgrade(self) -> IRFrameRef<'l> {
        IRFrameRef {
            ptr: self.ptr,
            ir_stack: self.ir_stack,
        }
    }

    pub fn write_at_offset(&mut self, offset: FramePointerOffset, to_write: u64) {
        unsafe { (self.ptr.offset(-(offset.0 as isize)) as *mut u64).write(to_write) }
    }

    pub fn frame_ptr(&self) -> *mut c_void {
        self.ptr
    }

    pub fn set_prev_rip(&mut self, prev_rip: *const c_void) {
        self.write_at_offset(FramePointerOffset(FRAME_HEADER_PREV_RIP_OFFSET), prev_rip as u64);
    }
}


//iters up from position on stack
pub struct IRStackIter {
    current_rbp: *mut c_void,
    top: *mut c_void,
}

impl Iterator for IRStackIter {
    type Item = IRStackEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let frame_pointer = self.current_rbp;
        let ir_stack_read = unsafe { read_frame_ir_header(frame_pointer) };
        self.current_rbp = ir_stack_read.prev_rbp;
        Some(IRStackEntry {
            rbp: frame_pointer
        })
    }
}

pub const OPAQUE_FRAME_SIZE: usize = 1024;

pub struct UnPackedIRFrameHeader {
    prev_rip: *mut c_void,
    prev_rbp: *mut c_void,
    ir_method_id: IRMethodID,
    method_id_ignored: u64,
    magic_1: u64,
    magic_2: u64,
    // as above but grows down
    // magic_2: *mut c_void,
    // magic_1: *mut c_void,
    // ignored_java_data2: *mut c_void,
    // ignored_java_data1: *mut c_void,
    // prev_rbp: *mut c_void,
    // prev_rip: *mut c_void,
}

pub const FRAME_HEADER_PREV_RIP_OFFSET: usize = 0;
pub const FRAME_HEADER_PREV_RBP_OFFSET: usize = 8;
pub const FRAME_HEADER_IR_METHOD_ID_OFFSET: usize = 16;
pub const FRAME_HEADER_METHOD_ID_OFFSET: usize = 24;
pub const FRAME_HEADER_PREV_MAGIC_1_OFFSET: usize = 32;
pub const FRAME_HEADER_PREV_MAGIC_2_OFFSET: usize = 40;
pub const FRAME_HEADER_END_OFFSET: usize = 48;


unsafe fn read_frame_ir_header(frame_pointer: *const c_void) -> UnPackedIRFrameHeader {
    let rip_ptr = frame_pointer.offset(-(FRAME_HEADER_PREV_RIP_OFFSET as isize)) as *const *mut c_void;
    let rbp_ptr = frame_pointer.offset(-(FRAME_HEADER_PREV_RBP_OFFSET as isize)) as *const *mut c_void;
    let magic1_ptr = frame_pointer.offset(-(FRAME_HEADER_PREV_MAGIC_1_OFFSET as isize)) as *const u64;
    let magic2_ptr = frame_pointer.offset(-(FRAME_HEADER_PREV_MAGIC_2_OFFSET as isize)) as *const u64;
    let ir_method_id_ptr = frame_pointer.offset(-(FRAME_HEADER_IR_METHOD_ID_OFFSET as isize)) as *const usize;
    let method_id_ptr = frame_pointer.offset(-(FRAME_HEADER_METHOD_ID_OFFSET as isize)) as *const u64;
    let magic_1 = magic1_ptr.read();
    let magic_2 = magic2_ptr.read();
    assert_eq!(magic_1, MAGIC_1_EXPECTED);
    assert_eq!(magic_2, MAGIC_2_EXPECTED);
    UnPackedIRFrameHeader {
        prev_rip: rip_ptr.read(),
        prev_rbp: rbp_ptr.read(),
        ir_method_id: IRMethodID(*ir_method_id_ptr),
        method_id_ignored: *method_id_ptr,
        magic_1,
        magic_2,
    }
}

pub struct IRStackEntry {
    rbp: *mut c_void,
}

