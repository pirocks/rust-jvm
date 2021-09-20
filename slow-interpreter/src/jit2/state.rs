use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::cmp::max;
use std::collections::HashMap;
use std::env::current_exe;
use std::error::Error;
use std::ffi::c_void;
use std::intrinsics::copy_nonoverlapping;
use std::mem::size_of;
use std::ptr::null_mut;
use std::thread;
use std::thread::LocalKey;

use itertools::Itertools;
use nix::sys::mman::{MapFlags, mmap, ProtFlags};

use classfile_view::view::HasAccessFlags;
use gc_memory_layout_common::{ArrayMemoryLayout, FramePointerOffset, ObjectMemoryLayout, StackframeMemoryLayout};
use jit_common::{JitCodeContext, SavedRegisters};
use jit_common::java_stack::JavaStack;
use jvmti_jni_bindings::{jlong, jvalue};
use rust_jvm_common::classfile::InstructionInfo::lookupswitch;
use rust_jvm_common::compressed_classfile::code::{CInstruction, CompressedCode, CompressedInstructionInfo};

use crate::class_loading::check_initing_or_inited_class;
use crate::instructions::invoke::native::run_native_method;
use crate::interpreter::WasException;
use crate::interpreter_state::{InterpreterState, InterpreterStateGuard};
use crate::java_values::JavaValue;
use crate::jit2::{ByteCodeOffset, CompiledCodeID, exit_handler, ir_to_native, IRInstructionIndex, LabelName, to_ir, ToIR, ToNative, transition_stack_frame, TransitionType, VMExitType};
use crate::jit2::ir::{IRInstr, IRLabel};
use crate::jvm_state::JVMState;
use crate::method_table::MethodId;

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
    pub top_level_exit_code: *mut c_void,
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
        let mut res = Self {
            code: &CODE_ADDRESS,
            method_id_to_code: Default::default(),
            function_addresses: vec![],
            current_end: code_address,
            current_jit_instr: IRInstructionIndex(0),
            exits: HashMap::new(),
            labels: HashMap::new(),
            labeler: Labeler { current_label: 0 },
            top_level_exit_code: null_mut(),
        };
        res.top_level_exit_code = res.add_top_level_exit_code();
        res
    }


    fn add_top_level_exit_code(&mut self) -> *mut c_void {
        let mut labels = vec![];
        let exit_label = self.labeler.new_label(&mut labels);
        let ir = ToIR {
            labels,
            ir: vec![(ByteCodeOffset(0), IRInstr::VMExit { exit_label, exit_type: VMExitType::TopLevelReturn {} })],
        };

        let current_code_id = self.next_code_id((-1isize) as usize);
        self.add_from_ir(current_code_id, ir)
    }

    fn next_code_id(&mut self, method_id: MethodId) -> CompiledCodeID {
        let next_code_id = CompiledCodeID(self.method_id_to_code.len() as u32);
        assert!(!self.method_id_to_code.values().contains(&next_code_id));
        self.method_id_to_code.insert(method_id, next_code_id);
        next_code_id
    }

    pub fn add_function(&mut self, code: &CompressedCode, methodid: usize) -> *mut c_void {
        let current_code_id = self.next_code_id(methodid);
        let CompressedCode {
            instructions,
            max_locals,
            max_stack,
            exception_table,
            stack_map_table
        } = code;
        let cinstructions = instructions.iter().sorted_by_key(|(offset, _)| **offset).map(|(_, ci)| ci).collect_vec();
        let layout = NaiveStackframeLayout::new(&cinstructions, *max_locals);
        let ir = to_ir(cinstructions, &mut self.labeler, &layout).unwrap();
        self.add_from_ir(current_code_id, ir)
    }

    fn add_from_ir(&mut self, current_code_id: CompiledCodeID, ir: ToIR) -> *mut c_void {
        let ToNative {
            code,
            new_labels,
            exits
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
        for (label_name, exit_type) in exits {
            self.exits.insert(new_labels[&label_name], exit_type);
        }
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

    pub fn run_method_safe(&mut self, jvm: &'gc_life JVMState<'gc_life>, int_state: &mut InterpreterStateGuard<'gc_life, 'l>, methodid: MethodId) -> Result<Option<JavaValue>, WasException> {
        unsafe {
            self.run_method(jvm, int_state, methodid, self.method_id_to_code[&methodid])
        }
    }

    #[allow(named_asm_labels)]
    pub unsafe fn run_method(&mut self, jvm: &'gc_life JVMState<'gc_life>, int_state: &mut InterpreterStateGuard<'gc_life, '_>, methodid: MethodId, compiled_id: CompiledCodeID) -> Result<Option<JavaValue>, WasException> {
        let mut target_ip = self.function_addresses[compiled_id.0 as usize];
        loop {
            let java_stack: &mut JavaStack = int_state.get_java_stack();
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
            eprintln!("going in");
            dbg!(frame_pointer);
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
            dbg!(&self.exits);
            dbg!(old_java_ip);
            eprintln!("going out ");
            dbg!(java_stack.frame_pointer());
            let exit_type = self.exits.get(&old_java_ip).unwrap().clone();
            target_ip = match self.handle_exit(exit_type, jvm, int_state, methodid, old_java_ip) {
                None => {
                    return Ok(None);
                }
                Some(target_ip) => target_ip
            };
        }
    }
    fn handle_exit(&mut self, exit_type: VMExitType, jvm: &'gc_life JVMState<'gc_life>, int_state: &mut InterpreterStateGuard<'gc_life, '_>, methodid: usize, old_java_ip: *mut c_void) -> Option<*mut c_void> {
        match exit_type {
            VMExitType::ResolveInvokeStatic { method_name, desc, target_class } => {
                let inited_class = check_initing_or_inited_class(jvm, int_state, target_class).unwrap();
                let method_view = inited_class.unwrap_class_class().class_view.lookup_method(method_name, &desc).unwrap();
                dbg!(method_name.0.to_str(&jvm.string_pool));
                let to_call_function_method_id = jvm.method_table.write().unwrap().get_method_id(inited_class.clone(), method_view.method_i());
                if method_view.is_native() {
                    dbg!(int_state.get_java_stack().stack_pointer());
                    dbg!(int_state.get_java_stack().frame_pointer());
                    match run_native_method(jvm, int_state, inited_class.clone(), method_view.method_i()) {
                        Ok(Some(res)) => int_state.current_frame_mut().push(res),
                        Ok(None) => {}
                        Err(WasException {}) => todo!(),
                    };
                    return Some(old_java_ip);
                } else {
                    self.add_function(method_view.code_attribute().unwrap(), to_call_function_method_id);
                    let (current_function_rc, current_function_method_i) = jvm.method_table.read().unwrap().try_lookup(methodid).unwrap();
                    let method_view = current_function_rc.unwrap_class_class().class_view.method_view_i(current_function_method_i);
                    let code = method_view.code_attribute().unwrap();
                    self.recompile_method(methodid, code, int_state.get_java_stack(), TransitionType::ResolveCalls);
                    todo!()
                }
            }
            VMExitType::TopLevelReturn { .. } => {
                int_state.set_function_return(true);
                None
            }
            VMExitType::ResolveInvokeSpecial { .. } => {
                todo!()
            }
            VMExitType::Todo { .. } => {
                todo!()
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


pub struct NaiveStackframeLayout {
    max_locals: u16,
    stack_depth: HashMap<u16, u16>,
}

impl NaiveStackframeLayout {
    pub fn new(instructions: &Vec<&CInstruction>, max_locals: u16) -> Self {
        let mut stack_depth = HashMap::new();
        let mut current_depth = 0;
        for (i, instruct) in instructions.iter().enumerate() {
            match &instruct.info {
                CompressedInstructionInfo::invokestatic { .. } => {}
                CompressedInstructionInfo::return_ => {}
                CompressedInstructionInfo::aload_0 => {
                    current_depth += 1;
                }
                CompressedInstructionInfo::aload_1 => {
                    current_depth += 1;
                }
                CompressedInstructionInfo::invokespecial { method_name, descriptor, classname_ref_type } => {
                    current_depth -= 1;
                    current_depth -= descriptor.arg_types.len() as u16;
                }
                CompressedInstructionInfo::iconst_0 => {
                    current_depth += 1;
                }
                CompressedInstructionInfo::putfield { name, desc, target_class } => {
                    current_depth -= 1;
                }
                CompressedInstructionInfo::aconst_null => {
                    current_depth += 1;
                }
                todo => todo!("{:?}", todo)
            }
            stack_depth.insert(i as u16, current_depth);
        }
        Self {
            max_locals,
            stack_depth,
        }
    }
}

impl StackframeMemoryLayout for NaiveStackframeLayout {
    fn local_var_entry(&self, current_count: u16, i: u16) -> FramePointerOffset {
        FramePointerOffset(i as usize * size_of::<jlong>())
    }

    fn operand_stack_entry(&self, current_count: u16, from_end: u16) -> FramePointerOffset {
        dbg!(current_count);
        dbg!(&self.stack_depth);
        FramePointerOffset((self.max_locals + self.stack_depth[&current_count] - from_end) as usize * size_of::<jlong>())
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

