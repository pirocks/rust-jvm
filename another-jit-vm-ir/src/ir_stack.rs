use std::ffi::c_void;
use std::mem::size_of;
use std::ptr::{NonNull};
use nonnull_const::NonNullConst;
use another_jit_vm::{FramePointerOffset, IRMethodID, MAGIC_1_EXPECTED, MAGIC_2_EXPECTED};

use another_jit_vm::stack::{CannotAllocateStack, OwnedNativeStack};
use gc_memory_layout_common::layout::{FRAME_HEADER_END_OFFSET, FRAME_HEADER_IR_METHOD_ID_OFFSET, FRAME_HEADER_METHOD_ID_OFFSET, FRAME_HEADER_PREV_MAGIC_1_OFFSET, FRAME_HEADER_PREV_MAGIC_2_OFFSET, FRAME_HEADER_PREV_RBP_OFFSET, FRAME_HEADER_PREV_RIP_OFFSET};
use rust_jvm_common::MethodId;

use crate::{IRInstructIndex, IRVMState};

// IR knows about stack so we should have a stack
// will have IR instruct for new frame, so IR also knows about frames
pub struct OwnedIRStack {
    pub native: OwnedNativeStack,
}


impl<'k> OwnedIRStack {
    pub fn new() -> Result<Self,CannotAllocateStack> {
        Ok(Self {
            native: OwnedNativeStack::new()?
        })
    }

    pub unsafe fn frame_at<'l>(&'l self, frame_pointer: NonNullConst<c_void>) -> IRFrameRef<'l> {
        self.native.validate_frame_pointer(frame_pointer);
        let _frame_header = read_frame_ir_header(frame_pointer);
        IRFrameRef {
            ptr: frame_pointer,
            _ir_stack: self,
        }
    }

    pub unsafe fn frame_at_mut(&mut self, frame_pointer: NonNull<c_void>) -> IRFrameMut {
        self.native.validate_frame_pointer(frame_pointer.into());
        let _frame_header = read_frame_ir_header(frame_pointer.into());
        IRFrameMut {
            ptr: frame_pointer,
            ir_stack: self,
        }
    }

    pub unsafe fn frame_iter<'h, 'vm>(&'_ self, start_frame: NonNullConst<c_void>, ir_vm_state: &'h IRVMState<'vm>) -> IRFrameIterRef<'vm, '_, 'h> {
        IRFrameIterRef {
            ir_stack: self,
            current_frame_ptr: Some(start_frame),
            ir_vm_state,
        }
    }

    pub unsafe fn write_frame(&self, frame_pointer: NonNull<c_void>, prev_rip: *const c_void, prev_rbp: *mut c_void, ir_method_id: Option<IRMethodID>, method_id: i64, data: &[u64]) {
        self.native.validate_frame_pointer(frame_pointer.into());
        let prev_rip_ptr = frame_pointer.as_ptr().sub(FRAME_HEADER_PREV_RIP_OFFSET) as *mut *const c_void;
        prev_rip_ptr.write(prev_rip);
        let prev_rpb_ptr = frame_pointer.as_ptr().sub(FRAME_HEADER_PREV_RBP_OFFSET) as *mut *mut c_void;
        prev_rpb_ptr.write(prev_rbp);
        let magic_1_ptr = frame_pointer.as_ptr().sub(FRAME_HEADER_PREV_MAGIC_1_OFFSET) as *mut u64;
        magic_1_ptr.write(MAGIC_1_EXPECTED);
        let magic_2_ptr = frame_pointer.as_ptr().sub(FRAME_HEADER_PREV_MAGIC_2_OFFSET) as *mut u64;
        magic_2_ptr.write(MAGIC_2_EXPECTED);
        let ir_method_id_ptr = frame_pointer.as_ptr().sub(FRAME_HEADER_IR_METHOD_ID_OFFSET) as *mut u64;
        ir_method_id_ptr.write(ir_method_id.unwrap_or(IRMethodID(usize::MAX)).0 as u64);
        let method_id_ptr = frame_pointer.as_ptr().sub(FRAME_HEADER_METHOD_ID_OFFSET) as *mut i64;
        method_id_ptr.write(method_id);
        for (i, data_elem) in data.iter().cloned().enumerate() {
            let data_elem_ptr = frame_pointer.as_ptr().sub(FRAME_HEADER_END_OFFSET).sub(i * size_of::<u64>()) as *mut u64;
            data_elem_ptr.write(data_elem)
        }
    }
}

pub struct IRStackMut<'l> {
    pub owned_ir_stack: &'l mut OwnedIRStack,
    pub current_rbp: NonNull<c_void>,
    pub current_rsp: NonNull<c_void>,
}

impl<'l, 'k> IRStackMut<'l> {
    pub fn new(owned_ir_stack: &'l mut OwnedIRStack, current_rbp: NonNull<c_void>, exiting_stack_pointer: NonNull<c_void>) -> Self {
        unsafe { owned_ir_stack.native.validate_frame_pointer(current_rbp.into()) }
        Self {
            owned_ir_stack,
            current_rbp,
            current_rsp: exiting_stack_pointer,
        }
    }

    pub fn from_stack_start(owned_ir_stack: &'l mut OwnedIRStack) -> Self {
        let mmaped_top = owned_ir_stack.native.mmaped_top;
        Self {
            owned_ir_stack,
            current_rbp: mmaped_top,
            current_rsp: mmaped_top,
        }
    }

    pub fn push_frame<'vm_lfe>(&mut self, prev_rip: *const c_void, ir_method_id: Option<IRMethodID>, method_id: i64, data: &[u64], _ir_vm_state: &'_ IRVMState<'vm_lfe>) -> IRPushFrameGuard {
        unsafe {
            // if self.current_rsp != self.owned_ir_stack.native.mmaped_top && self.current_rbp != self.owned_ir_stack.native.mmaped_top {
            //     let offset = self.current_rbp.offset_from(self.current_rsp).abs() as usize;
            //     let expected_current_frame_size = self.current_frame_ref().frame_size(ir_vm_state);
            //     assert_eq!(offset, expected_current_frame_size);
            // }
            let prev_rbp = self.current_rbp;
            let prev_rsp = self.current_rsp;
            self.current_rbp = self.current_rsp;
            self.current_rsp = NonNull::new(self.current_rbp.as_ptr().sub(FRAME_HEADER_END_OFFSET + data.len() * size_of::<u64>())).unwrap();
            self.owned_ir_stack.write_frame(self.current_rbp, prev_rip, prev_rbp.as_ptr(), ir_method_id, method_id, data);
            assert!((self.current_frame_ref().ir_method_id() == ir_method_id));
            assert_ne!(self.current_rbp, self.current_rsp);
            IRPushFrameGuard {
                exited_correctly: false,
                return_to_rbp: prev_rbp,
                return_to_rsp: prev_rsp,
            }
        }
    }

    pub fn pop_frame(&mut self, mut frame_guard: IRPushFrameGuard) {
        self.current_rsp = self.current_rbp;
        self.current_rbp = self.current_frame_ref().prev_rbp().unwrap();
        frame_guard.exited_correctly = true;
        assert_eq!(frame_guard.return_to_rbp, self.current_rbp);
        assert_eq!(frame_guard.return_to_rsp, self.current_rsp);
    }

    pub fn debug_print_stack_strace<'vm, ExtraData>(&self, ir_vm_state: &'_ IRVMState<'vm>) {
        let frame_iter = self.frame_iter(ir_vm_state);
        eprintln!("Start IR stacktrace:");
        for frame in frame_iter {
            match frame.ir_method_id() {
                None => eprintln!("IR Method ID: unknown {:?}", frame.ptr),
                Some(id) => {
                    eprintln!("IR Method ID: {:?} {:?}", id.0, frame.ptr);
                }
            }
        }
        eprintln!("End IR stacktrace");
    }

    pub fn frame_iter<'h, 'vm>(&'l self, ir_vm_state: &'h IRVMState<'vm>) -> IRFrameIterRef<'vm, 'l, 'h> {
        unsafe { self.owned_ir_stack.frame_iter(self.current_rbp.into(), ir_vm_state) }
    }


    pub fn current_frame_ref(&self) -> IRFrameRef {
        IRFrameRef {
            ptr: self.current_rbp.into(),
            _ir_stack: self.owned_ir_stack,
        }
    }

    pub fn previous_frame_ref(&self) -> IRFrameRef {
        let prev_rbp = self.current_frame_ref().prev_rbp().unwrap();
        IRFrameRef {
            ptr: prev_rbp.into(),
            _ir_stack: self.owned_ir_stack,
        }
    }

    pub fn previous_frame_ir_instr<'vm>(&self, ir_vm_state: &IRVMState<'vm>) -> IRInstructIndex {
        let current = self.current_frame_ref();
        let prev_rip = current.prev_rip();
        let (ir_method_id_from_ip, ir_instruct) = ir_vm_state.lookup_ip(prev_rip);
        assert_eq!(self.previous_frame_ref().ir_method_id(), Some(ir_method_id_from_ip));
        ir_instruct
    }

    pub fn current_frame_mut(&'_ mut self) -> IRFrameMut<'_> {
        IRFrameMut {
            ptr: self.current_rbp,
            ir_stack: self.owned_ir_stack,
        }
    }
}

#[must_use]
pub struct IRPushFrameGuard {
    exited_correctly: bool,
    pub return_to_rbp: NonNull<c_void>,
    pub return_to_rsp: NonNull<c_void>,
}


impl Drop for IRPushFrameGuard {
    fn drop(&mut self) {
        assert!(self.exited_correctly);
    }
}


// has ref b/c not valid to access this after top level stack has been modified
pub struct IRFrameIterRef<'vm, 'l, 'h> {
    ir_stack: &'l OwnedIRStack,
    current_frame_ptr: Option<NonNullConst<c_void>>,
    ir_vm_state: &'h IRVMState<'vm>,
}

impl<'l, 'h, 'vm> Iterator for IRFrameIterRef<'vm, 'l, 'h> {
    type Item = IRFrameRef<'l>;

    fn next(&mut self) -> Option<Self::Item> {
        let res = unsafe { self.ir_stack.frame_at(self.current_frame_ptr?) };
        unsafe {
            if self.current_frame_ptr? == self.ir_stack.native.mmaped_top.into() {
                self.current_frame_ptr = None;
            } else {
                let prev_ir_frame_ref = self.ir_stack.frame_at(res.prev_rbp().unwrap().into());
                if let Some(new_current_frame_size) = prev_ir_frame_ref.try_frame_size(self.ir_vm_state) {
                    if res.prev_rbp() != None {
                        assert_eq!(res.prev_rbp().unwrap().as_ptr().offset_from(self.current_frame_ptr.unwrap().as_ptr()) as usize, new_current_frame_size);
                    }
                }
                self.current_frame_ptr = Some(res.prev_rbp().unwrap().into());
            }
        }
        Some(res)
    }
}

// has ref b/c not valid to access this after top level stack has been modified
pub struct IRFrameRef<'l> {
    pub ptr: NonNullConst<c_void>,
    pub _ir_stack: &'l OwnedIRStack,
}

impl IRFrameRef<'_> {
    pub fn header(&self) -> UnPackedIRFrameHeader {
        unsafe { read_frame_ir_header(self.ptr) }
    }

    pub fn read_at_offset(&self, offset: FramePointerOffset) -> u64 {
        unsafe { (self.ptr.as_ptr().offset(-(offset.0 as isize)) as *mut u64).read() }
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


    pub fn raw_method_id(&self) -> i64 {
        let res = self.read_at_offset(FramePointerOffset(FRAME_HEADER_METHOD_ID_OFFSET));
        res as i64
    }

    pub fn method_id(&self) -> Option<MethodId> {
        let res = self.read_at_offset(FramePointerOffset(FRAME_HEADER_METHOD_ID_OFFSET));
        if (res as i64) < 0 {
            return None;
        }
        let frame_header = unsafe { read_frame_ir_header(self.ptr) };
        assert_eq!(res, frame_header.method_id_ignored);
        Some(res as usize)
    }

    pub fn prev_rbp(&self) -> Option<NonNull<c_void>> {
        let res = self.read_at_offset(FramePointerOffset(FRAME_HEADER_PREV_RBP_OFFSET));
        let frame_header = unsafe { read_frame_ir_header(self.ptr) };
        assert_eq!(res, frame_header.prev_rbp as u64);
        NonNull::new(res as *mut c_void)
    }

    pub fn prev_rip(&self) -> *const c_void {
        let res = self.read_at_offset(FramePointerOffset(FRAME_HEADER_PREV_RIP_OFFSET));
        let frame_header = unsafe { read_frame_ir_header(self.ptr) };
        assert_eq!(res, frame_header.prev_rip as u64);
        res as *mut c_void
    }

    pub fn try_frame_size(&self, ir_vm_state: &IRVMState) -> Option<usize> {
        let ir_method_id = match self.ir_method_id() {
            Some(x) => x,
            None => {
                return None;
            }
        };
        Some(*ir_vm_state.inner.read().unwrap().frame_sizes_by_ir_method_id.get(&ir_method_id).unwrap())
    }


    pub fn frame_size(&self, ir_vm_state: &IRVMState) -> usize {
        match self.try_frame_size(ir_vm_state) {
            None => {
                DEFAULT_FRAME_SIZE
            }
            Some(res) => {
                res
            }
        }
    }

    pub fn data(&self, index: usize) -> u64 {
        let data_raw_ptr = unsafe { self.ptr.as_ptr().sub(FRAME_HEADER_END_OFFSET).sub(index * size_of::<u64>()) as *const u64 };
        unsafe { data_raw_ptr.read() }
    }

    pub fn all_data<'vm>(&self, ir_vm_state: &'_ IRVMState<'vm>) -> Vec<u64> {
        let _frame_size = self.frame_size(ir_vm_state);
        todo!()
    }

    pub fn frame_ptr(&self) -> NonNullConst<c_void> {
        self.ptr
    }
}

//todo this is scuffed
//frame header size + one data pointer for native frame data
#[allow(clippy::identity_op)]
pub const DEFAULT_FRAME_SIZE: usize = FRAME_HEADER_END_OFFSET + 1 * size_of::<*const c_void>();

// has ref b/c not valid to access this after top level stack has been modified
pub struct IRFrameMut<'l> {
    pub ptr: NonNull<c_void>,
    pub ir_stack: &'l mut OwnedIRStack,
}

impl<'l> IRFrameMut<'l> {
    pub fn downgrade_owned(self) -> IRFrameRef<'l> {
        IRFrameRef {
            ptr: self.ptr.into(),
            _ir_stack: self.ir_stack,
        }
    }

    pub fn downgrade<'new_l>(&'new_l self) -> IRFrameRef<'new_l> {
        IRFrameRef {
            ptr: self.ptr.into(),
            _ir_stack: self.ir_stack,
        }
    }

    pub fn write_data(&self, index: usize, data: u64) {
        let data_raw_ptr = unsafe { self.ptr.as_ptr().sub(FRAME_HEADER_END_OFFSET).sub(index * size_of::<u64>()) as *mut u64 };
        unsafe { data_raw_ptr.write(data); }
    }

    pub fn write_at_offset(&mut self, offset: FramePointerOffset, to_write: u64) {
        unsafe { (self.ptr.as_ptr().offset(-(offset.0 as isize)) as *mut u64).write(to_write) }
    }

    pub fn frame_ptr(&self) -> NonNull<c_void> {
        self.ptr
    }

    pub fn set_ir_method_id(&mut self, ir_method_id: IRMethodID) {
        self.write_at_offset(FramePointerOffset(FRAME_HEADER_IR_METHOD_ID_OFFSET), ir_method_id.0 as u64);
    }

    pub fn set_prev_rip(&mut self, prev_rip: *const c_void) {
        self.write_at_offset(FramePointerOffset(FRAME_HEADER_PREV_RIP_OFFSET), prev_rip as u64);
    }

    pub fn assert_prev_rip(&mut self, prev_rip: *const c_void) {
        let actual_prev_rip = self.downgrade().read_at_offset(FramePointerOffset(FRAME_HEADER_PREV_RIP_OFFSET));
        assert_eq!(actual_prev_rip, prev_rip as u64);
    }
}


pub const OPAQUE_FRAME_SIZE: usize = 1024;

#[derive(Debug)]
#[derive(Copy, Clone)]
pub struct UnPackedIRFrameHeader {
    prev_rip: *mut c_void,
    pub prev_rbp: *mut c_void,
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


pub unsafe fn read_frame_ir_header(frame_pointer: NonNullConst<c_void>) -> UnPackedIRFrameHeader {
    let rip_ptr = frame_pointer.as_ptr().sub(FRAME_HEADER_PREV_RIP_OFFSET) as *const *mut c_void;
    let rbp_ptr = frame_pointer.as_ptr().sub(FRAME_HEADER_PREV_RBP_OFFSET) as *const *mut c_void;
    let magic1_ptr = frame_pointer.as_ptr().sub(FRAME_HEADER_PREV_MAGIC_1_OFFSET) as *const u64;
    let magic2_ptr = frame_pointer.as_ptr().sub(FRAME_HEADER_PREV_MAGIC_2_OFFSET) as *const u64;
    let ir_method_id_ptr = frame_pointer.as_ptr().sub(FRAME_HEADER_IR_METHOD_ID_OFFSET) as *const usize;
    let method_id_ptr = frame_pointer.as_ptr().sub(FRAME_HEADER_METHOD_ID_OFFSET) as *const u64;
    let magic_1 = magic1_ptr.read();
    let magic_2 = magic2_ptr.read();
    let res = UnPackedIRFrameHeader {
        prev_rip: rip_ptr.read(),
        prev_rbp: rbp_ptr.read(),
        ir_method_id: IRMethodID(*ir_method_id_ptr),
        method_id_ignored: *method_id_ptr,
        magic_1,
        magic_2,
    };
    if res.magic_1 != MAGIC_1_EXPECTED || res.magic_2 != MAGIC_2_EXPECTED {
        dbg!(res);
        eprintln!("{:X}", res.magic_1);
        eprintln!("{:X}", res.magic_2);
        panic!()
    }
    res
}

