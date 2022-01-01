#![feature(in_band_lifetimes)]
use std::collections::{HashMap, HashSet};
use std::ffi::c_void;
use std::mem::{size_of, transmute};
use std::ops::Deref;
use std::ptr::null_mut;
use std::slice::from_raw_parts;
use std::sync::{Arc, RwLock};
use bimap::BiHashMap;
use iced_x86::code_asm::{CodeAssembler, qword_ptr, rax, rbp, rbx, rsp};
use iced_x86::{BlockEncoder, BlockEncoderOptions, InstructionBlock, IntelFormatter};
use libc::{MAP_ANONYMOUS, MAP_GROWSDOWN, MAP_NORESERVE, MAP_PRIVATE, PROT_READ, PROT_WRITE};
use another_jit_vm::{BaseAddress, Method, Register, SavedRegistersWithoutIP, VMExitEvent, VMState};
use crate::vm_exit_abi::RuntimeVMExitInput;

#[cfg(test)]
mod tests;



#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct LabelName(u32);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct IRLabel {
    pub(crate) name: LabelName,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct RestartPointID(pub(crate) u64);

pub enum IRInstr {
    LoadFPRelative { from: FramePointerOffset, to: Register },
    StoreFPRelative { from: Register, to: FramePointerOffset },
    Load { to: Register, from_address: Register },
    Store { to_address: Register, from: Register },
    CopyRegister { from: Register, to: Register },
    Add { res: Register, a: Register },
    Sub { res: Register, to_subtract: Register },
    Div { res: Register, divisor: Register },
    Mod { res: Register, divisor: Register },
    Mul { res: Register, a: Register },
    BinaryBitAnd { res: Register, a: Register },
    ForwardBitScan { to_scan: Register, res: Register },
    Const32bit { to: Register, const_: u32 },
    Const64bit { to: Register, const_: u64 },
    BranchToLabel { label: LabelName },
    LoadLabel { label: LabelName, to: Register },
    LoadRBP { to: Register },
    WriteRBP { from: Register },
    BranchEqual { a: Register, b: Register, label: LabelName },
    BranchNotEqual { a: Register, b: Register, label: LabelName },
    Return { return_val: Option<Register>, temp_register_1: Register, temp_register_2: Register, temp_register_3: Register, temp_register_4: Register, frame_size: usize },
    // VMExit { before_exit_label: LabelName, after_exit_label: Option<LabelName>, exit_type: VMExitTypeWithArgs },
    RestartPoint(RestartPointID),
    VMExit2 { exit_type: IRVMExitType },
    NPECheck { possibly_null: Register,temp_register: Register, npe_exit_type: IRVMExitType },
    GrowStack { amount: usize },
    LoadSP { to: Register },
    WithAssembler { function: Box<dyn FnOnce(&mut CodeAssembler) -> ()> },
    IRNewFrame {
        current_frame_size: usize,
        temp_register: Register,
        return_to_rip: Register,
    },
    IRCall{
        temp_register_1: Register,
        temp_register_2: Register,
        current_frame_size: usize,
        new_frame_size : usize,
        target_address: *const c_void //todo perhaps this should be an ir_method id
    },
    FNOP,
    Label(IRLabel),
}



#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct IRMethodID(pub usize);

pub struct IRVMStateInner {
    // each IR function is distinct single java methods may many ir methods
    ir_method_id_max: IRMethodID,
    top_level_return_function_id: Option<IRMethodID>,
    current_implementation: BiHashMap<IRMethodID, MethodImplementationID>,
    frame_sizes_by_address: HashMap<*const c_void, usize>,//todo not used currently
    frame_sizes_by_ir_method_id: HashMap<IRMethodID, usize>,
    method_ir_offsets: HashMap<IRMethodID, BiHashMap<IRInstructNativeOffset, IRInstructIndex>>,
    method_ir: HashMap<IRMethodID, Vec<IRInstr>>,
    // index
    opaque_method_to_or_method_id: HashMap<u64, IRMethodID>,
    // function_ir_mapping: HashMap<IRMethodID, !>,
}

impl IRVMStateInner {
    pub fn new() -> Self {
        Self {
            ir_method_id_max: IRMethodID(0),
            top_level_return_function_id: None,
            current_implementation: Default::default(),
            frame_sizes_by_address: Default::default(),
            frame_sizes_by_ir_method_id: Default::default(),
            method_ir_offsets: Default::default(),
            method_ir: Default::default(),
            opaque_method_to_or_method_id: Default::default(),
        }
    }

    pub fn add_function_ir_offsets(&mut self, current_ir_id: IRMethodID,
                                   new_instruction_offsets: Vec<IRInstructNativeOffset>,
                                   ir_instruct_index_to_assembly_index: Vec<(IRInstructIndex, AssemblyInstructionIndex)>) {
        let mut res = BiHashMap::new();//todo these bihashmaps are dangerous, should assert nothing is ever overwritten
        for ((i, instruction_offset),(ir_instruction_index,assembly_instruction_index_2)) in new_instruction_offsets.into_iter().enumerate().zip(ir_instruct_index_to_assembly_index.into_iter()) {
            let assembly_instruction_index_1 = AssemblyInstructionIndex(i);
            assert_eq!(assembly_instruction_index_1,assembly_instruction_index_2);
            if let Some(ir_instruction_offset) = res.get_by_right(&ir_instruction_index){
                if *ir_instruction_offset > instruction_offset{
                    res.insert(instruction_offset, ir_instruction_index);
                    panic!("don't expect this to actually be needed")
                }
            }else {
                let overwritten = res.insert(instruction_offset, ir_instruction_index);
                assert!(!overwritten.did_overwrite());
            }
        }
        let indexes = res.iter().map(|(_, instruct)|*instruct).collect::<HashSet<_>>();
        assert_eq!(indexes.iter().max().unwrap().0 + 1, indexes.len());
        self.method_ir_offsets.insert(current_ir_id, res);
    }
}

pub struct IRVMState<'vm_life> {
    native_vm: VMState<'vm_life, u64, (Arc<JavaThread<'vm_life>>, JavaStackPosition, &'vm_life JVMState<'vm_life>)>,
    inner: RwLock<IRVMStateInner>,
}

pub const OPAQUE_FRAME_SIZE: usize = 1024;

impl<'vm_life> IRVMState<'vm_life> {
    pub fn lookup_opaque_ir_method_id(&self, opaque_id: u64) -> IRMethodID {
        let mut guard = self.inner.write().unwrap();
        match guard.opaque_method_to_or_method_id.get(&opaque_id) {
            None => {
                guard.ir_method_id_max.0 += 1;
                let new_ir_method_id = guard.ir_method_id_max;
                guard.opaque_method_to_or_method_id.insert(opaque_id, new_ir_method_id);
                guard.frame_sizes_by_ir_method_id.insert(new_ir_method_id, OPAQUE_FRAME_SIZE);
                drop(guard);
                return self.lookup_opaque_ir_method_id(opaque_id);
            }
            Some(ir_method_id) => {
                *ir_method_id
            }
        }
    }

    pub fn lookup_ir_method_id_pointer(&self, ir_method_id: IRMethodID) -> *const c_void {
        let guard = self.inner.read().unwrap();
        let current_implementation = &guard.current_implementation;
        let ir_method_implementation = *current_implementation.get_by_left(&ir_method_id).unwrap();
        drop(guard);
        self.native_vm.lookup_method_addresses(ir_method_implementation).start
    }

    pub fn get_top_level_return_ir_method_id(&self) -> IRMethodID {
        self.inner.read().unwrap().top_level_return_function_id.unwrap()
    }

    pub fn init_top_level_exit_id(&self, ir_method_id: IRMethodID) {
        let mut guard = self.inner.write().unwrap();
        assert!(guard.top_level_return_function_id.is_none());
        guard.top_level_return_function_id = Some(ir_method_id);
    }

    pub fn lookup_location_of_ir_instruct(&self, ir_method_id: IRMethodID, ir_instruct_index: IRInstructIndex) -> NativeInstructionLocation {
        let read_guard = self.inner.read().unwrap();
        let method_ir_offsets_for_this_method = read_guard.method_ir_offsets.get(&ir_method_id).unwrap();
        let offset = *method_ir_offsets_for_this_method.get_by_right(&ir_instruct_index).unwrap();
        let func_start = self.lookup_ir_method_id_pointer(ir_method_id);
        unsafe { NativeInstructionLocation(func_start.offset(offset.0 as isize)) }
    }
}

// IR knows about stack so we should have a stack
// will have IR instruct for new frame, so IR also knows about frames
pub struct OwnedIRStack {
    pub(crate) mmaped_top: *mut c_void,
    pub(crate) mmaped_bottom: *mut c_void,
    max_stack: usize,
}

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
        let frame_header = read_frame_ir_header(frame_pointer);
        IRFrameRef {
            ptr: frame_pointer,
            ir_stack: self,
        }
    }

    pub unsafe fn frame_at_mut(&'l mut self, frame_pointer: *mut c_void) -> IRFrameMut<'l> {
        self.validate_frame_pointer(frame_pointer);
        let frame_header = read_frame_ir_header(frame_pointer);
        IRFrameMut {
            ptr: frame_pointer,
            ir_stack: self,
        }
    }

    pub unsafe fn frame_iter(&self, start_frame: *mut c_void, ir_vm_state: &'vm_life IRVMState<'vm_life>) -> IRFrameIter<'_, 'vm_life> {
        IRFrameIter {
            ir_stack: self,
            current_frame_ptr: Some(start_frame),
            ir_vm_state,
        }
    }

    unsafe fn validate_frame_pointer(&self, frame_pointer: *const c_void) {
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
pub struct IRFrameIter<'l, 'vm_life> {
    ir_stack: &'l OwnedIRStack,
    current_frame_ptr: Option<*mut c_void>,
    ir_vm_state: &'vm_life IRVMState<'vm_life>,
}

impl<'l, 'k> Iterator for IRFrameIter<'l, 'k> {
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

    pub fn frame_size(&self, ir_vm_state: &IRVMState) -> usize {
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

pub struct IRStackEntry {
    rbp: *mut c_void,
}

impl<'vm_life> IRVMState<'vm_life> {
    pub fn new() -> Self {
        let mut res = Self {
            native_vm: VMState::new(),
            inner: RwLock::new(IRVMStateInner::new()),
        };
        res
    }

    pub fn run_method(&self, method_id: IRMethodID, int_state: &mut InterpreterStateGuard<'vm_life, 'l>, frame_pointer: *const c_void, stack_pointer: *const c_void) -> u64 {
        let inner_read_guard = self.inner.read().unwrap();
        let current_implementation = *inner_read_guard.current_implementation.get_by_left(&method_id).unwrap();
        //todo for now we launch with zeroed registers, in future we may need to map values to stack or something

        let ir_stack = &mut int_state.java_stack().inner;
        unsafe { ir_stack.validate_frame_pointer(frame_pointer); }
        let mut initial_registers = SavedRegistersWithoutIP::new_with_all_zero();
        initial_registers.rbp = frame_pointer as *mut c_void;
        initial_registers.rsp = stack_pointer as *mut c_void;
        drop(int_state.int_state.take());
        drop(inner_read_guard);
        let res = self.native_vm.launch_vm(current_implementation, initial_registers, (int_state.thread.clone(), JavaStackPosition::Frame { frame_pointer }, int_state.jvm));
        unsafe { int_state.int_state = Some(transmute(int_state.thread.interpreter_state.lock().unwrap())) };
        res
    }

    fn debug_print_instructions(assembler: &CodeAssembler, offsets: &Vec<IRInstructNativeOffset>, base_address: BaseAddress) {
        let mut formatted_instructions = String::new();
        let mut formatter = IntelFormatter::default();
        for (i, instruction) in assembler.instructions().iter().enumerate() {
            unsafe { formatted_instructions.push_str(format!("{:?}:", base_address.0.offset(offsets[i].0 as isize)).as_ref()) }
            formatter.format(instruction, &mut formatted_instructions);
            formatted_instructions.push('\n');
        }
        // eprintln!("{}", formatted_instructions);
    }

    pub fn add_function(&'vm_life self, instructions: Vec<IRInstr>, frame_size: usize, ir_exit_handler: Box<dyn Fn(&IRVMExitEvent, &mut InterpreterStateGuard<'vm_life, '_>) -> VMExitAction<u64> + 'vm_life>) -> (IRMethodID, HashMap<RestartPointID, IRInstructIndex>) {
        let mut inner_guard = self.inner.write().unwrap();
        let current_ir_id = inner_guard.ir_method_id_max;
        inner_guard.ir_method_id_max.0 += 1;
        let (code_assembler, assembly_index_to_ir_instruct_index, restart_points) = add_function_from_ir(instructions);
        let base_address = self.native_vm.get_new_base_address();
        let block = InstructionBlock::new(code_assembler.instructions(), base_address.0 as u64);
        let result = BlockEncoder::encode(code_assembler.bitness(), block, BlockEncoderOptions::RETURN_NEW_INSTRUCTION_OFFSETS).unwrap();
        let new_instruction_offsets = result.new_instruction_offsets.into_iter().map(|new_instruction_offset| IRInstructNativeOffset(new_instruction_offset as usize)).collect_vec();
        Self::debug_print_instructions(&code_assembler,&new_instruction_offsets,base_address);
        inner_guard.add_function_ir_offsets(current_ir_id, new_instruction_offsets, assembly_index_to_ir_instruct_index);
        inner_guard.frame_sizes_by_ir_method_id.insert(current_ir_id, frame_size);
        let vm_exit_handler: Arc<dyn Fn(&VMExitEvent, &mut (Arc<JavaThread<'vm_life>>, JavaStackPosition, &'vm_life JVMState<'vm_life>)) -> VMExitAction<u64> + 'vm_life> =
            Arc::new(move |vm_exit_event: &VMExitEvent, (java_thread, current_stack_position, jvm)| {
                let mut guard = java_thread.interpreter_state.lock().unwrap();
                guard.deref_mut().current_stack_position = *current_stack_position;
                let mut new_int_state = InterpreterStateGuard::new(jvm, java_thread.clone(), guard);
                new_int_state.register_interpreter_state_guard(jvm);
                vm_exit_handler(self, vm_exit_event, &mut new_int_state, ir_exit_handler.deref())
            });
        let code = result.code_buffer;

        let method_implementation_id = self.native_vm.add_method_implementation(Method {
            code,
            exit_handler: vm_exit_handler,
        }, base_address);
        inner_guard.current_implementation.insert(current_ir_id, method_implementation_id);
        (current_ir_id,restart_points)
    }
}

fn vm_exit_handler<'vm_life, 'l>(ir_vm_state: &'vm_life IRVMState<'vm_life>, vm_exit_event: &VMExitEvent, int_state: &mut InterpreterStateGuard<'vm_life, 'l>, ir_exit_handler: &(dyn Fn(&IRVMExitEvent, &mut InterpreterStateGuard<'vm_life, 'l>) -> VMExitAction<u64> + 'vm_life)) -> VMExitAction<u64> {
    let implementation_id = vm_exit_event.method;
    let exit_address = vm_exit_event.saved_guest_registers.rip;
    let exit_method_base_address = vm_exit_event.method_base_address;
    let offset = unsafe { exit_address.offset_from(exit_method_base_address) };
    if offset < 0 {
        panic!()
    }
    assert!(offset < 1024 * 1024);// methods over a megabyte prob aren't a thing

    // let offset = IRInstructNativeOffset(offset as usize);
    let inner_read_guard = ir_vm_state.inner.read().unwrap();
    let ir_method_id = *inner_read_guard.current_implementation.get_by_right(&implementation_id).unwrap();

    drop(inner_read_guard);
    // let method_offsets = inner_read_guard.method_ir_offsets.get(&ir_method_id).unwrap();
    // dbg!(method_offsets);
    // dbg!(offset);
    // let ir_instruct_index = method_offsets.get_by_left(&offset).unwrap();

    let exit_type = RuntimeVMExitInput::from_register_state(&vm_exit_event.saved_guest_registers);

    let ir_vm_exit_event = IRVMExitEvent {
        inner: &vm_exit_event,
        ir_method: ir_method_id,
        exit_type,
        exiting_frame_position: JavaStackPosition::Frame {
            frame_pointer: vm_exit_event.saved_guest_registers.saved_registers_without_ip.rbp },
    };

    ir_exit_handler(&ir_vm_exit_event, int_state)
}


fn add_function_from_ir(instructions: Vec<IRInstr>) -> (CodeAssembler, Vec<(IRInstructIndex, AssemblyInstructionIndex)>, HashMap<RestartPointID, IRInstructIndex>) {
    let mut assembler = CodeAssembler::new(64).unwrap();
    let mut ir_instruct_index_to_assembly_instruction_index = Vec::new();
    let mut labels = HashMap::new();
    let mut restart_points = HashMap::new();
    for (i, instruction) in instructions.into_iter().enumerate() {
        let assembly_instruction_index_start = AssemblyInstructionIndex(assembler.instructions().len());
        let ir_instruction_index = IRInstructIndex(i);
        single_ir_to_native(&mut assembler, instruction, &mut labels, &mut restart_points, ir_instruction_index);
        let assembly_instruction_index_end = AssemblyInstructionIndex(assembler.instructions().len());
        assert!(!(assembly_instruction_index_start..assembly_instruction_index_end).is_empty());
        for assembly_index in assembly_instruction_index_start..assembly_instruction_index_end {
            ir_instruct_index_to_assembly_instruction_index.push((ir_instruction_index, assembly_index));
        }
    }
    (assembler, ir_instruct_index_to_assembly_instruction_index, restart_points)
}

fn single_ir_to_native(assembler: &mut CodeAssembler, instruction: IRInstr, labels: &mut HashMap<LabelName, CodeLabel>,
                       restart_points: &mut HashMap<RestartPointID, IRInstructIndex>, ir_instr_index: IRInstructIndex) {
    match instruction {
        IRInstr::LoadFPRelative { from, to } => {
            //stack grows down
            assembler.mov(to.to_native_64(), rbp - from.0).unwrap();
        }
        IRInstr::StoreFPRelative { from, to } => {
            assembler.mov(qword_ptr(rbp - to.0), from.to_native_64()).unwrap();
        }
        IRInstr::Load { .. } => todo!(),
        IRInstr::Store { from, to_address } => {
            assembler.mov(qword_ptr(to_address.to_native_64()), from.to_native_64()).unwrap()
        }
        IRInstr::CopyRegister { .. } => todo!(),
        IRInstr::Add { a, res } => {
            assembler.add(res.to_native_64(), a.to_native_64()).unwrap()
        }
        IRInstr::Sub { .. } => todo!(),
        IRInstr::Div { .. } => todo!(),
        IRInstr::Mod { .. } => todo!(),
        IRInstr::Mul { .. } => todo!(),
        IRInstr::BinaryBitAnd { .. } => todo!(),
        IRInstr::ForwardBitScan { .. } => todo!(),
        IRInstr::Const32bit { .. } => todo!(),
        IRInstr::Const64bit { const_, to } => {
            assembler.mov(to.to_native_64(), const_).unwrap();
        }
        IRInstr::BranchToLabel { label } => {
            let code_label = labels.entry(label).or_insert_with(|| assembler.create_label());
            assembler.jmp(code_label.clone()).unwrap();
        }
        IRInstr::LoadLabel { .. } => todo!(),
        IRInstr::LoadRBP { .. } => todo!(),
        IRInstr::WriteRBP { .. } => todo!(),
        IRInstr::BranchEqual { .. } => todo!(),
        IRInstr::BranchNotEqual { a, b, label, } => {
            let code_label = labels.entry(label).or_insert_with(|| assembler.create_label());
            assembler.cmp(a.to_native_64(), b.to_native_64()).unwrap();
            assembler.jne(code_label.clone()).unwrap();
        }
        IRInstr::Return { return_val, temp_register_1, temp_register_2, temp_register_3, temp_register_4, frame_size } => {
            if let Some(return_register) = return_val {
                assert_ne!(temp_register_1.to_native_64(), rax);
                assert_ne!(temp_register_2.to_native_64(), rax);
                assert_ne!(temp_register_3.to_native_64(), rax);
                assert_ne!(temp_register_4.to_native_64(), rax);
                assembler.mov(rax, return_register.to_native_64()).unwrap();
            }
            //rsp is now equal is to prev rbp qword, so that we can pop the previous rip in ret
            assembler.mov(rsp, rbp).unwrap();
            //load prev fram pointer
            assembler.mov(rbp, rbp - FRAME_HEADER_PREV_RBP_OFFSET).unwrap();
            assembler.ret().unwrap();
        }
        IRInstr::VMExit2 { exit_type } => {
            gen_vm_exit(assembler, exit_type);
        }
        IRInstr::GrowStack { .. } => todo!(),
        IRInstr::LoadSP { .. } => todo!(),
        IRInstr::WithAssembler { .. } => todo!(),
        IRInstr::FNOP => todo!(),
        IRInstr::Label(label) => {
            let label_name = label.name;
            let code_label = labels.entry(label_name).or_insert_with(|| assembler.create_label());
            assembler.set_label(code_label);
            assembler.nop().unwrap();
        }
        IRInstr::IRNewFrame {
            current_frame_size,
            temp_register,
            return_to_rip
        } => {
            todo!()
        }
        IRInstr::VMExit { .. } => panic!("legacy"),
        IRInstr::IRCall { current_frame_size, new_frame_size, temp_register_1, temp_register_2, target_address } => {
            let return_to_rip = temp_register_2.to_native_64();
            let temp_register = temp_register_1.to_native_64();
            let mut after_call_label = assembler.create_label();
            assembler.lea(return_to_rip, qword_ptr(after_call_label.clone())).unwrap();
            assembler.mov(temp_register, MAGIC_1_EXPECTED).unwrap();
            assembler.mov(rbp - (current_frame_size + FRAME_HEADER_PREV_MAGIC_1_OFFSET) as u64, temp_register).unwrap();
            assembler.mov(temp_register, MAGIC_2_EXPECTED).unwrap();
            assembler.mov(rbp - (current_frame_size + FRAME_HEADER_PREV_MAGIC_2_OFFSET) as u64, temp_register).unwrap();

            assembler.mov(rbp - (current_frame_size + FRAME_HEADER_PREV_RBP_OFFSET) as u64, rbp).unwrap();
            assembler.mov(rbp - (current_frame_size + FRAME_HEADER_PREV_RIP_OFFSET) as u64, return_to_rip).unwrap();
            assembler.mov(temp_register, target_address as u64).unwrap();
            assembler.jmp(temp_register).unwrap();
            assembler.set_label(&mut after_call_label);
        }
        IRInstr::NPECheck { temp_register, npe_exit_type, possibly_null } => {
            let mut after_exit_label = assembler.create_label();
            assembler.xor(temp_register.to_native_64(), temp_register.to_native_64()).unwrap();
            assembler.cmp(temp_register.to_native_64(), possibly_null.to_native_64()).unwrap();
            assembler.jne(after_exit_label).unwrap();
            gen_vm_exit(assembler, npe_exit_type);
            assembler.nop_1(rax).unwrap();
            assembler.set_label(&mut after_exit_label).unwrap();
        }
        IRInstr::RestartPoint(restart_point_id) => {
            assembler.nop_1(rbx).unwrap();
            restart_points.insert(restart_point_id, ir_instr_index);
        }
    }
}

fn gen_vm_exit(assembler: &mut CodeAssembler, exit_type: IRVMExitType) {
    let mut before_exit_label = assembler.create_label();
    let mut after_exit_label = assembler.create_label();
    let registers = vec![Register(1), Register(2), Register(3), Register(4), Register(5)];
    exit_type.gen_assembly(assembler, &mut before_exit_label, &mut after_exit_label, registers.clone());
    VMState::<u64, InterpreterStateGuard>::gen_vm_exit(assembler, &mut before_exit_label, &mut after_exit_label, registers.into_iter().collect());
}


//index is an index, offset is a byte offset from method start

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Ord, PartialOrd)]
pub struct IRInstructIndex(usize);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Ord, PartialOrd)]
pub struct IRInstructNativeOffset(usize);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Ord, PartialOrd)]
pub struct AssemblyInstructionIndex(usize);

impl std::iter::Step for AssemblyInstructionIndex {
    fn steps_between(start: &Self, end: &Self) -> Option<usize> {
        Some(end.0 - start.0)
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        Some(AssemblyInstructionIndex(start.0 + count))
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        Some(AssemblyInstructionIndex(start.0 - count))
    }
}


pub struct IRVMExitEvent<'l> {
    pub inner: &'l VMExitEvent,
    pub ir_method: IRMethodID,
    pub exit_type: RuntimeVMExitInput,
    exiting_frame_position: JavaStackPosition,
}

pub mod vm_exit_abi;