use std::cell::RefCell;
use std::collections::HashMap;
use std::mem::{size_of, transmute};
use std::os::raw::c_void;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicBool, AtomicUsize, fence, Ordering};
use std::sync::RwLock;

use iced_x86::{BlockEncoder, BlockEncoderOptions, BlockEncoderResult, InstructionBlock};
use memoffset::offset_of;
use nix::sys::mman::{MapFlags, mmap, ProtFlags};

use gc_memory_layout_common::FrameBackedStackframeMemoryLayout;
use jit_common::{JavaStack, JitCodeContext, SavedRegisters, VMExitType};
use jit_ir::{AbsoluteOffsetInCodeRegion, InstructionSink, VMExits};
use rust_jvm_common::classfile::SameFrameExtended;

use crate::code_to_ir;

pub struct JITedCode {
    code: Vec<CodeRegion>,
}

struct CodeRegion {
    raw: *mut c_void,
    vm_exits: VMExits,
}


const MAX_CODE_SIZE: usize = 1_000_000usize;
const CODE_LOCATION: usize = 0x1_000_000usize;

impl JITedCode {
    pub unsafe fn add_code_region(&mut self, instructions: InstructionSink) -> usize {
        let prot_flags = ProtFlags::PROT_EXEC | ProtFlags::PROT_WRITE | ProtFlags::PROT_READ;
        let flags = MapFlags::MAP_ANONYMOUS | MapFlags::MAP_NORESERVE | MapFlags::MAP_PRIVATE;
        let mmap_addr = mmap(transmute(CODE_LOCATION), MAX_CODE_SIZE, prot_flags, flags, -1, 0).unwrap();
        let rip_start = mmap_addr as u64;

        let block = InstructionBlock::new(instructions.as_slice(), rip_start as u64);
        let BlockEncoderResult { mut code_buffer, .. } = BlockEncoder::encode(64, block, BlockEncoderOptions::NONE).unwrap();
        let len_before = self.code.len();

        if code_buffer.len() > MAX_CODE_SIZE {
            panic!("exceeded max code size");
        }

        libc::memcpy(mmap_addr, code_buffer.as_ptr() as *const c_void, code_buffer.len());

        self.code.push(CodeRegion {
            raw: mmap_addr as *mut c_void,
            vm_exits: instructions.get_vm_exits_given_installed_address(mmap_addr as *mut c_void),
        });
        fence(Ordering::SeqCst);
        // __clear_cache();//todo should use this
        return len_before;
    }

    pub unsafe fn run_jitted_coded(&self, id: usize, stack: JavaStack) {
        let as_ptr = self.code[id].raw;
        let vm_exits: &VMExits = &self.code[id].vm_exits;
        let SavedRegisters { stack_pointer, frame_pointer, instruction_pointer: _ } = stack.handle_vm_entry();
        let rust_stack: u64 = stack_pointer as u64;
        let rust_frame: u64 = frame_pointer as u64;
        let jit_code_context = JitCodeContext {
            native_saved: SavedRegisters {
                stack_pointer: 0xdeaddeaddeaddead as *mut c_void,
                frame_pointer: 0xdeaddeaddeaddead as *mut c_void,
                instruction_pointer: 0xdeaddeaddeaddead as *mut c_void,
            },
            java_saved: SavedRegisters {
                stack_pointer,
                frame_pointer,
                instruction_pointer: as_ptr as *mut c_void,
            },
        };
        let jit_context_pointer = &jit_code_context as *const JitCodeContext as u64;
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
        // load context pointer into r15
        // store old stack pointer into context
        "mov [{0} + {old_stack_pointer_offset}],rsp",
        // store old frame pointer into context
        "mov [{0} + {old_frame_pointer_offset}],rbp",
        // store exit instruction pointer into context
        "lea r15, [rip+after_call]",
        "mov [{0} + {old_rip_offset}],r15",
        "mov r15,{0}",
        // load java frame pointer
        "mov rbp, [{0} + {new_frame_pointer_offset}]",
        // load java stack pointer
        "mov rsp, [{0} + {new_stack_pointer_offset}]",
        // jump to jitted code
        "jmp [{0} + {new_rip_offset}]",
        //
        "after_call:",
        "pop rsp",
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop rbp",
        "pop rbx",
        in(reg) jit_context_pointer,
        old_stack_pointer_offset = const 0,//(offset_of!(JitCodeContext,native_saved) + offset_of!(SavedRegisters,stack_pointer)),
        old_frame_pointer_offset = const 8,//(offset_of!(JitCodeContext,native_saved) + offset_of!(SavedRegisters,frame_pointer)),
        old_rip_offset = const 16,//(offset_of!(JitCodeContext,native_saved) + offset_of!(SavedRegisters,instruction_pointer)),
        new_stack_pointer_offset = const 24,//(offset_of!(JitCodeContext,java_saved) + offset_of!(SavedRegisters,stack_pointer)),
        new_frame_pointer_offset = const 32,//(offset_of!(JitCodeContext,java_saved) + offset_of!(SavedRegisters,frame_pointer)),
        new_rip_offset = const 40,//(offset_of!(JitCodeContext,java_saved) + offset_of!(SavedRegisters,instruction_pointer))
        );

        dbg!(*(stack_pointer.offset(0) as *mut u64));
        dbg!(*(stack_pointer.offset(8) as *mut u64));
        dbg!(*(stack_pointer.offset(16) as *mut u64));
        dbg!(*(stack_pointer.offset(24) as *mut u64));
        dbg!(*(stack_pointer.offset(36) as *mut u64));
        dbg!(jit_code_context.java_saved);
        let vm_exit_type = vm_exits.memory_offset_to_vm_exit.get(&AbsoluteOffsetInCodeRegion(jit_code_context.java_saved.instruction_pointer)).expect("Unexpected VM exit");
        match vm_exit_type {
            VMExitType::CheckCast => todo!(),
            VMExitType::InstanceOf => todo!(),
            VMExitType::Throw => todo!(),
            VMExitType::InvokeDynamic => todo!(),
            VMExitType::InvokeStaticResolveTarget { .. } => todo!(),
            VMExitType::InvokeVirtualResolveTarget { .. } => todo!(),
            VMExitType::InvokeSpecialResolveTarget { .. } => todo!(),
            VMExitType::InvokeInterfaceResolveTarget { .. } => todo!(),
            VMExitType::MonitorEnter => todo!(),
            VMExitType::MonitorExit => todo!(),
            VMExitType::MultiNewArray => todo!(),
            VMExitType::ArrayOutOfBounds => todo!(),
            VMExitType::DebugTestExit => {
                return;//we are in test and expected this
            }
            VMExitType::ExitDueToCompletion => todo!()
        }
    }

    // pub fn handle_vm_exit(&self, vm_exit_type: VMExitType) -> !{
    //     match vm_exit_type{
    //         VMExitType::CheckCast => todo!(),
    //         VMExitType::InstanceOf => todo!(),
    //         VMExitType::Throw => todo!(),
    //         VMExitType::InvokeDynamic => todo!(),
    //         VMExitType::InvokeStaticResolveTarget { .. } => todo!(),
    //         VMExitType::InvokeVirtualResolveTarget { .. } => todo!(),
    //         VMExitType::InvokeSpecialResolveTarget { .. } => todo!(),
    //         VMExitType::InvokeInterfaceResolveTarget { .. } => todo!(),
    //         VMExitType::MonitorEnter => todo!(),
    //         VMExitType::MonitorExit => todo!(),
    //         VMExitType::MultiNewArray => todo!(),
    //         VMExitType::ArrayOutOfBounds => todo!(),
    //         VMExitType::DebugTestExit => todo!(),
    //         VMExitType::ExitDueToCompletion => {}
    //     }
    // }
}


pub struct CodeCache {
    //todo actually write one of these
    jitted_code: JITedCode,
}


pub struct JittedFunction {}

impl JittedFunction {
    pub fn call_code_jit(&self, jitted_code: &mut JITedCode, args: Vec<()>) {
        let stack_memory_layout: FrameBackedStackframeMemoryLayout = todo!();
        assert!(args.is_empty());
        let res = code_to_ir(todo!(), &stack_memory_layout).expect("failed to compile");
    }
}

#[cfg(test)]
pub mod test {
    use iced_x86::{Formatter, Instruction, InstructionBlock, IntelFormatter};

    use gc_memory_layout_common::{ArrayMemoryLayout, FramePointerOffset, ObjectMemoryLayout, StackframeMemoryLayout};
    use jit_common::{JavaStack, VMExitType};
    use jit_ir::{InstructionSink, IRInstruction, Size, VariableSize};
    use rust_jvm_common::classfile::InstructionInfo;

    use crate::code_to_ir;
    use crate::native::JITedCode;

    #[test]
    pub fn test() {
        let mut instructions: InstructionSink = InstructionSink::new();
        IRInstruction::LoadAbsolute { address_from: FramePointerOffset(10), output_offset: FramePointerOffset(10), size: Size::Int }.to_x86(&mut instructions);
        // IRInstruction::Return { return_value: None, to_pop: VariableSize(0) }.to_x86(&mut instructions);
        let mut formatter = IntelFormatter::new();
        let mut res = String::new();
        // for instruction in &instructions {
        //     formatter.format(instruction, &mut res);
        //     res.push_str("\n")
        // }
        // println!("{}", res);
        let mut jitted_code = JITedCode {
            code: vec![]
        };
        let id = unsafe { jitted_code.add_code_region(instructions) };
        unsafe { jitted_code.run_jitted_coded(id, JavaStack::new(11)); }
    }

    pub struct TestStackframeMemoryLayout {}

    impl StackframeMemoryLayout for TestStackframeMemoryLayout {
        fn local_var_entry(&self, pc: usize, i: usize) -> FramePointerOffset {
            todo!()
        }

        fn operand_stack_entry(&self, pc: usize, from_end: usize) -> FramePointerOffset {
            todo!()
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

    #[test]
    pub fn test_basic_return() {
        let res = code_to_ir(vec![rust_jvm_common::classfile::Instruction { offset: 0, instruction: InstructionInfo::return_ }], &TestStackframeMemoryLayout {} as &dyn StackframeMemoryLayout).expect("failed to compile");
        let mut x86_instructions = InstructionSink::new();
        for ir_instruction in res.main_block.instructions {
            ir_instruction.to_x86(&mut x86_instructions);
        }
        for block in res.additional_blocks {
            for ir_instruction in block.instructions {
                ir_instruction.to_x86(&mut x86_instructions);
            }
        }
        let mut jitted_code = JITedCode {
            code: vec![]
        };
        let id = unsafe { jitted_code.add_code_region(x86_instructions) };
        unsafe { jitted_code.run_jitted_coded(id, JavaStack::new(11)); }
    }

    #[test]
    pub fn test_basic_debug_vm_exit() {
        let mut x86_instructions = InstructionSink::new();
        IRInstruction::VMExit(VMExitType::DebugTestExit {}).to_x86(&mut x86_instructions);
        let mut jitted_code = JITedCode {
            code: vec![]
        };
        let id = unsafe { jitted_code.add_code_region(x86_instructions) };
        unsafe { jitted_code.run_jitted_coded(id, JavaStack::new(11)); }
    }
}
