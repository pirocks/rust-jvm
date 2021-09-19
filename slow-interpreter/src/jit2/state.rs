use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::cmp::max;
use std::collections::HashMap;
use std::error::Error;
use std::ffi::c_void;
use std::intrinsics::copy_nonoverlapping;
use std::ptr::null_mut;
use std::thread;
use std::thread::LocalKey;

use itertools::Itertools;
use nix::sys::mman::{MapFlags, mmap, ProtFlags};

use gc_memory_layout_common::{ArrayMemoryLayout, FramePointerOffset, ObjectMemoryLayout, StackframeMemoryLayout};
use jit_common::{JitCodeContext, SavedRegisters};
use jit_common::java_stack::JavaStack;
use jvmti_jni_bindings::jvalue;
use rust_jvm_common::classfile::InstructionInfo::lookupswitch;
use rust_jvm_common::compressed_classfile::code::{CInstruction, CompressedCode, CompressedInstructionInfo};

use crate::jit2::{CompiledCodeID, exit_handler, ir_to_native, IRInstructionIndex, LabelName, to_ir, ToNative, transition_stack_frame, TransitionType, VMExitType};
use crate::jit2::ir::IRLabel;

thread_local! {
pub static CODE_ADDRESS : RefCell<*mut c_void> = RefCell::new(null_mut());
}

pub struct JITState {
    code: &'static LocalKey<RefCell<*mut c_void>>,
    method_id_to_code: HashMap<usize, CompiledCodeID>,
    // indexed by compiled id:
    current_end: *mut c_void,
    function_addresses: Vec<*mut c_void>,
    current_jit_instr: IRInstructionIndex,
    exits: HashMap<*mut c_void, VMExitType>,
    labels: HashMap<LabelName, *mut c_void>,
    labeler: Labeler,
}


pub struct NaiveStackframeLayout {
    max_locals: u16,
    stack_depth: HashMap<u16, u16>,
}

impl NaiveStackframeLayout {
    pub fn new(instructions: &Vec<&CInstruction>, max_locals: u16) -> Self {
        let mut stack_depth = HashMap::new();
        let current_depth = 0;
        for instruct in instructions {
            match &instruct.info {
                CompressedInstructionInfo::invokestatic { .. } => {}
                CompressedInstructionInfo::return_ => {}
                todo => todo!("{:?}", todo)
            }
            stack_depth.insert(instruct.offset, current_depth);
        }
        Self {
            max_locals,
            stack_depth,
        }
    }
}

impl StackframeMemoryLayout for NaiveStackframeLayout {
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
        todo!()
    }

    fn safe_temp_location(&self, pc: u16, i: u16) -> FramePointerOffset {
        todo!()
    }
}


impl JITState {
    pub fn new() -> Self {
        let code_address = CODE_ADDRESS.with(|address_refcell| {
            let address = *address_refcell.borrow();
            if address != null_mut() {
                panic!("multiple jitstates");
            }
            let thread_id_numeric = thread::current().id().as_u64();
            const BASE_CODE_ADDRESS: usize = 1024 * 1024 * 1024 * 1024;
            const THREAD_CODE_ADDRESS_MULTIPLIER: usize = 1024 * 1024 * 1024 * 2;
            const MAX_CODE_SIZE: usize = 2 * 1024 * 1024 * 1024 - 1;
            let addr = BASE_CODE_ADDRESS + (thread_id_numeric.get() as usize) * THREAD_CODE_ADDRESS_MULTIPLIER;
            let res_addr = unsafe { mmap(addr as *mut c_void, MAX_CODE_SIZE, ProtFlags::PROT_WRITE | ProtFlags::PROT_EXEC, MapFlags::MAP_ANONYMOUS | MapFlags::MAP_NORESERVE | MapFlags::MAP_PRIVATE, -1, 0).unwrap() } as *mut c_void;
            *address_refcell.borrow_mut() = res_addr;
            res_addr
        });
        Self {
            code: &CODE_ADDRESS,
            method_id_to_code: Default::default(),
            function_addresses: vec![],
            current_end: code_address,
            current_jit_instr: IRInstructionIndex(0),
            exits: HashMap::new(),
            labels: HashMap::new(),
            labeler: Labeler { current_label: 0 },
        }
    }


    pub fn add_function(&mut self, code: &CompressedCode, methodid: usize) -> *mut c_void {
        let next_code_id = CompiledCodeID(self.method_id_to_code.len() as u32);
        assert!(!self.method_id_to_code.values().contains(&next_code_id));
        self.method_id_to_code.insert(methodid, next_code_id);
        let current_code_id = next_code_id;
        let CompressedCode {
            instructions,
            max_locals,
            max_stack,
            exception_table,
            stack_map_table
        } = code;
        let cinstructions = instructions.iter().sorted_by_key(|(offset, _)| **offset).map(|(_, ci)| ci).collect_vec();
        let layout = NaiveStackframeLayout::new(&cinstructions, *max_locals);
        let ir = to_ir(cinstructions, self.current_jit_instr, &mut self.labeler, &layout).unwrap();
        let ToNative {
            code,
            new_labels
        } = ir_to_native(ir, self.current_end);
        let install_at = self.current_end;
        unsafe { self.current_end = install_at.offset(code.len() as isize); }
        self.code.with(|code_refcell| {
            const TWO_GIG: isize = 2 * 1024 * 1024 * 1024;
            unsafe {
                if self.current_end.offset_from(*code_refcell.borrow()) > TWO_GIG {
                    panic!()
                }
            }
        });
        self.labels.extend(new_labels.into_iter());
        unsafe {
            copy_nonoverlapping(
                code.as_ptr(),
                install_at as *mut u8,
                code.len(),
            )
        }
        let max_method_id = self.function_addresses.len();
        self.function_addresses.extend(itertools::repeat_n(null_mut(), max(max_method_id - current_code_id.0 as usize + 1, 0)));
        self.function_addresses[current_code_id.0 as usize] = install_at;
        install_at
    }

    pub fn recompile_method(&mut self, methodid: usize, code: &CompressedCode, java_stack: &mut JavaStack, transition_type: TransitionType) {
        transition_stack_frame(transition_type, java_stack);
        self.add_function(code, methodid);
        todo!("resume execution, adjust ip.")
    }

    pub fn run_method_safe(&mut self, methodid: usize, java_stack: &mut JavaStack) -> Result<Option<jvalue>, Box<dyn Error>> {
        unsafe {
            self.run_method(methodid, self.method_id_to_code[&methodid], java_stack);
        }
        todo!()
    }

    #[allow(named_asm_labels)]
    pub unsafe fn run_method(&mut self, methodid: usize, compiled_id: CompiledCodeID, java_stack: &mut JavaStack) {
        let target_ip = self.function_addresses[compiled_id.0 as usize];
        loop {
            let SavedRegisters { stack_pointer, frame_pointer, instruction_pointer: as_ptr, status_register } = java_stack.handle_vm_entry();
            let rust_stack: u64 = stack_pointer as u64;
            let rust_frame: u64 = frame_pointer as u64;

            let mut jit_code_context = JitCodeContext {
                native_saved: SavedRegisters {
                    stack_pointer: 0xdeaddeaddeaddead as *mut c_void,
                    frame_pointer: 0xdeaddeaddeaddead as *mut c_void,
                    instruction_pointer: 0xdeaddeaddeaddead as *mut c_void,
                    status_register,
                },
                java_saved: SavedRegisters {
                    stack_pointer,
                    frame_pointer,
                    instruction_pointer: target_ip,
                    status_register,
                },
                exit_handler_ip: exit_handler as *mut c_void,
            };
            let jit_context_pointer = &jit_code_context as *const JitCodeContext as u64;
            ///pub struct FrameHeader {
            //     pub prev_rip: *mut c_void,
            //     pub prev_rpb: *mut c_void,
            //     pub frame_info_ptr: *mut FrameInfo,
            //     pub debug_ptr: *mut c_void,
            //     pub magic_part_1: u64,
            //     pub magic_part_2: u64,
            // }
            let old_java_ip: *mut c_void;
            asm!(
            "push rbx",
            "push rbp",
            "push r12",
            "push r13",
            "push r14",
            "push r15",
            // technically these need only be saved on windows:
            // "push xmm*",
            //todo perhaps should use pusha/popa here, b/c this must be really slow
            "push rsp",
            // store old stack pointer into context
            "mov [{0} + {old_stack_pointer_offset}],rsp",
            // store old frame pointer into context
            "mov [{0} + {old_frame_pointer_offset}],rbp",
            // store exit instruction pointer into context
            "lea r15, [rip+__rust_jvm_internal_after_call]",
            "mov [{0} + {old_rip_offset}],r15",
            "mov r15,{0}",
            // load java frame pointer
            "mov rbp, [{0} + {new_frame_pointer_offset}]",
            // load java stack pointer
            "mov rsp, [{0} + {new_stack_pointer_offset}]",
            // jump to jitted code
            "jmp [{0} + {new_rip_offset}]",
            //
            "__rust_jvm_internal_after_call:",
            // gets old java ip from call back to here in java
            "pop {1}",
            "pop rsp",
            "pop r15",
            "pop r14",
            "pop r13",
            "pop r12",
            "pop rbp",
            "pop rbx",
            in(reg) jit_context_pointer,
            out(reg) old_java_ip,
            old_stack_pointer_offset = const 0,//(offset_of!(JitCodeContext,native_saved) + offset_of!(SavedRegisters,stack_pointer)),
            old_frame_pointer_offset = const 8,//(offset_of!(JitCodeContext,native_saved) + offset_of!(SavedRegisters,frame_pointer)),
            old_rip_offset = const 16,//(offset_of!(JitCodeContext,native_saved) + offset_of!(SavedRegisters,instruction_pointer)),
            new_stack_pointer_offset = const 32,//(offset_of!(JitCodeContext,java_saved) + offset_of!(SavedRegisters,stack_pointer)),
            new_frame_pointer_offset = const 40,//(offset_of!(JitCodeContext,java_saved) + offset_of!(SavedRegisters,frame_pointer)),
            new_rip_offset = const 48,//(offset_of!(JitCodeContext,java_saved) + offset_of!(SavedRegisters,instruction_pointer))
            );
            jit_code_context.java_saved.instruction_pointer = old_java_ip;
            java_stack.saved_registers = Some(jit_code_context.java_saved.clone());
            //todo exception handling
            match self.exits.get(&old_java_ip).unwrap().clone() {
                VMExitType::ResolveInvokeStatic { method_name, desc } => {
                    self.recompile_method(methodid, todo!(), java_stack, TransitionType::ResolveCalls);
                }
            }
        }
    }
}


pub struct Labeler {
    current_label: u32,
}

impl Labeler {
    pub fn new_label(&mut self, labels_vec: &mut Vec<IRLabel>) -> LabelName {
        let current_label = self.current_label.checked_add(1).unwrap();
        self.current_label = current_label;
        let res = LabelName(current_label);
        labels_vec.push(IRLabel { name: res });
        res
    }
}