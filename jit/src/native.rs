use std::mem::transmute;
use std::os::raw::c_void;
use std::sync::atomic::{fence, Ordering};

use iced_x86::{BlockEncoder, BlockEncoderOptions, BlockEncoderResult, InstructionBlock};
use memoffset::offset_of;
use nix::sys::mman::{MapFlags, mmap, ProtFlags};

use gc_memory_layout_common::FrameBackedStackframeMemoryLayout;
use jit_common::{JitCodeContext, SavedRegisters, VMExitType};
use jit_common::java_stack::JavaStack;
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

    pub unsafe fn run_jitted_coded(&self, id: usize, mut stack: JavaStack) {
        let as_ptr = self.code[id].raw;
        let vm_exits: &VMExits = &self.code[id].vm_exits;
        let SavedRegisters { stack_pointer, frame_pointer, instruction_pointer: _, status_register } = stack.handle_vm_entry();
        let rust_stack: u64 = stack_pointer as u64;
        let rust_frame: u64 = frame_pointer as u64;
        let jit_code_context = JitCodeContext {
            native_saved: SavedRegisters {
                stack_pointer: 0xdeaddeaddeaddead as *mut c_void,
                frame_pointer: 0xdeaddeaddeaddead as *mut c_void,
                instruction_pointer: 0xdeaddeaddeaddead as *mut c_void,
                status_register
            },
            java_saved: SavedRegisters {
                stack_pointer,
                frame_pointer,
                instruction_pointer: as_ptr as *mut c_void,
                status_register,
            },
        };
        dbg!(as_ptr);
        let jit_context_pointer = &jit_code_context as *const JitCodeContext as u64;
        ///pub struct FrameHeader {
        //     pub prev_rip: *mut c_void,
        //     pub prev_rpb: *mut c_void,
        //     pub frame_info_ptr: *mut FrameInfo,
        //     pub debug_ptr: *mut c_void,
        //     pub magic_part_1: u64,
        //     pub magic_part_2: u64,
        // }
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
        new_stack_pointer_offset = const 32,//(offset_of!(JitCodeContext,java_saved) + offset_of!(SavedRegisters,stack_pointer)),
        new_frame_pointer_offset = const 40,//(offset_of!(JitCodeContext,java_saved) + offset_of!(SavedRegisters,frame_pointer)),
        new_rip_offset = const 48,//(offset_of!(JitCodeContext,java_saved) + offset_of!(SavedRegisters,instruction_pointer))
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
            VMExitType::ExitDueToCompletion => {
                todo!()
            }
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
    use std::sync::{Arc, Mutex};

    use iced_x86::{Formatter, Instruction, InstructionBlock, IntelFormatter};

    use classfile_view::view::{ClassBackedView, ClassView, HasAccessFlags};
    use classfile_view::view::attribute_view::{BootstrapMethodsView, EnclosingMethodView, InnerClassesView, SourceFileView};
    use classfile_view::view::constant_info_view::ConstantInfoView;
    use classfile_view::view::field_view::{FieldIterator, FieldView};
    use classfile_view::view::interface_view::InterfaceIterator;
    use classfile_view::view::method_view::{MethodIterator, MethodView};
    use gc_memory_layout_common::{ArrayMemoryLayout, FramePointerOffset, ObjectMemoryLayout, StackframeMemoryLayout};
    use jit_common::VMExitType;
    use jit_common::java_stack::{JavaStack, JavaStatus};
    use jit_ir::{InstructionSink, IRInstruction, Size, VariableSize};
    use rust_jvm_common::classfile::{ACC_PRIVATE, ACC_STATIC, Classfile, InstructionInfo};
    use rust_jvm_common::classfile::StackMapFrame::SameFrameExtended;
    use rust_jvm_common::classnames::ClassName;
    use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CompressedClassfile, CompressedClassfileStringPool, CompressedMethodDescriptor, CompressedMethodInfo, CompressedParsedRefType, CPDType, CPRefType};
    use rust_jvm_common::compressed_classfile::code::{CInstruction, CInstructionInfo, CompressedCode};
    use rust_jvm_common::compressed_classfile::names::{CClassName, CompressedClassName, MethodName};
    use rust_jvm_common::loading::{LoaderName, NoopLivePoolGetter};
    use verification::{ClassFileGetter, VerifierContext, verify};

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
        unsafe { jitted_code.run_jitted_coded(id, JavaStack::new(11, todo!())); }
    }

    pub struct TestStackframeMemoryLayout {}

    impl StackframeMemoryLayout for TestStackframeMemoryLayout {
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

    #[test]
    pub fn test_basic_return() {
        let res = code_to_ir(vec![rust_jvm_common::compressed_classfile::code::CInstruction { offset: 0, instruction_size: 0, info: CInstructionInfo::return_ }], &TestStackframeMemoryLayout {} as &dyn StackframeMemoryLayout).expect("failed to compile");
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
        unsafe { jitted_code.run_jitted_coded(id, JavaStack::new(11, Box::into_raw(box JavaStatus::default()))); }
    }

    #[test]
    pub fn test_basic_debug_vm_exit() {
        let mut x86_instructions = InstructionSink::new();
        IRInstruction::VMExit(VMExitType::DebugTestExit {}).to_x86(&mut x86_instructions);
        let mut jitted_code = JITedCode {
            code: vec![]
        };
        let id = unsafe { jitted_code.add_code_region(x86_instructions) };
        unsafe { jitted_code.run_jitted_coded(id, JavaStack::new(11, Box::into_raw(box JavaStatus::default()))); }
    }


    #[test]
    pub fn test_basic_int_arithmetic() {
        let mut x86_instructions = InstructionSink::new();
        let java_instructions = vec![CInstructionInfo::iconst_0, CInstructionInfo::iconst_1, CInstructionInfo::iadd].into_iter().enumerate().map(|(offset, info)| CInstruction {
            offset: offset as u16,
            instruction_size: 1,
            info,
        }).collect_vec();
        let res = code_to_ir(java_instructions, todo!()).unwrap();
        for ir in res.main_block.instructions {
            ir.to_x86(&mut x86_instructions);
        }
        IRInstruction::VMExit(VMExitType::ExitDueToCompletion).to_x86(&mut x86_instructions);
        let mut jitted_code = JITedCode {
            code: vec![]
        };
        let id = unsafe { jitted_code.add_code_region(x86_instructions) };
        unsafe { jitted_code.run_jitted_coded(id, JavaStack::new(11, Box::into_raw(box JavaStatus::default()))); }
    }

    pub struct SingleMethodClassView {
        classfile: CompressedClassfile,
    }

    impl SingleMethodClassView {
        pub fn new(pool: &CompressedClassfileStringPool, instructions: Vec<CInstruction>, class_name: CClassName, method_name: MethodName, max_locals: u16, max_stack: u16) -> Self {
            let instructions = instructions.into_iter().map(|instr| (instr.offset, instr)).collect();
            Self {
                classfile: CompressedClassfile {
                    minor_version: 0,
                    major_version: 0,
                    access_flags: 0,
                    this_class: class_name,
                    super_class: None,
                    interfaces: vec![],
                    fields: vec![],
                    methods: vec![CompressedMethodInfo {
                        access_flags: ACC_STATIC & ACC_PRIVATE,
                        name: method_name.0,
                        descriptor: CompressedMethodDescriptor { arg_types: vec![], return_type: CPDType::VoidType },
                        descriptor_str: pool.add_name("()V", false),
                        code: Some(CompressedCode {
                            instructions,
                            max_locals,
                            max_stack,
                            exception_table: vec![],
                        }),
                    }],
                    bootstrap_methods: None,
                }
            }
        }
    }

    pub struct SingleClassViewGetter {
        class: CompressedClassfile,
    }

    impl ClassFileGetter for SingleClassViewGetter {
        fn get_classfile(&self, _loader: LoaderName, class: CClassName) -> Arc<dyn ClassView> {
            if class != self.class.this_class {
                panic!()
            }
            ClassBackedView {
                underlying_class: Arc::new(Classfile {}),
                backing_class: self.class.clone(),
                descriptor_index: Default::default(),
            }

            /*            ::from(Arc::new(Classfile{
                            magic: 0,
                            minor_version: 0,
                            major_version: 0,
                            constant_pool: vec![],
                            access_flags: 0,
                            this_class: 0,
                            super_class: 0,
                            interfaces: vec![],
                            fields: vec![],
                            methods: vec![],
                            attributes: vec![]
                        }),)
            */
        }
    }

    pub fn test_code(instruction_infos: Vec<CInstructionInfo>, pool: &CompressedClassfileStringPool) {
        let classname = CompressedClassName(pool.add_name("TestClass".to_string(), true));
        let mut verifier = VerifierContext {
            live_pool_getter: Arc::new(NoopLivePoolGetter {}),
            classfile_getter: Arc::new(NoopClassFileGetter {}),
            string_pool: pool,
            class_view_cache: Mutex::new(Default::default()),
            current_loader: LoaderName::BootstrapLoader,
            verification_types: Default::default(),
            debug: false,
        };

        verify(&mut verifier, classname, LoaderName::BootstrapLoader);
        let mut offset = 0;
        let java_instructions = instruction_infos.into_iter().map(|info| {
            let instruction_size = info.size(offset);
            let res = CInstruction {
                offset,
                instruction_size,
                info,
            };
            offset += instruction_size;
            res
        }).collect_vec();
        let types = verifier.verification_types;
        let res = code_to_ir(java_instructions, todo!());
    }
}
