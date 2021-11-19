use std::collections::HashMap;
use std::sync::RwLock;

use itertools::Itertools;

use another_jit_vm::VMExitAction;
use rust_jvm_common::compressed_classfile::code::{CompressedCode, CompressedInstruction, CompressedInstructionInfo};
use rust_jvm_common::compressed_classfile::CPDType;

use crate::{InterpreterStateGuard, JVMState};
use crate::jit::{ByteCodeOffset, MethodResolver, VMExitType};
use crate::jit::ir::IRInstr;
use crate::jit::state::Labeler;
use crate::method_table::MethodId;
use crate::native_to_ir_layer::{IRMethodID, IRVMExitEvent, IRVMState};

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub struct ExitNumber(u64);

pub struct JavaVMStateWrapperInner {
    method_id_to_ir_method_id: HashMap<MethodId, IRMethodID>,
    max_exit_number: ExitNumber,
    exit_numbers: HashMap<ExitNumber, VMExitType>,
}

impl JavaVMStateWrapperInner {
    fn handle_vm_exit(&self, jvm: &'gc_life JVMState<'gc_life>, int_state: &mut InterpreterStateGuard<'gc_life, '_>, method_id: MethodId, vm_exit_type: &VMExitType) -> VMExitAction<u64>{
        match vm_exit_type {
            VMExitType::ResolveInvokeStatic { .. } => todo!(),
            VMExitType::RunNativeStatic { .. } => todo!(),
            VMExitType::ResolveInvokeSpecial { .. } => todo!(),
            VMExitType::InvokeSpecialNative { .. } => todo!(),
            VMExitType::InitClass { .. } => todo!(),
            VMExitType::NeedNewRegion { .. } => todo!(),
            VMExitType::PutStatic { .. } => todo!(),
            VMExitType::Allocate { .. } => todo!(),
            VMExitType::LoadString { .. } => todo!(),
            VMExitType::LoadClass { .. } => todo!(),
            VMExitType::Throw { .. } => todo!(),
            VMExitType::MonitorEnter { .. } => todo!(),
            VMExitType::MonitorExit { .. } => todo!(),
            VMExitType::Trace { .. } => todo!(),
            VMExitType::TopLevelReturn { .. } => todo!(),
            VMExitType::Todo { .. } => todo!(),
            VMExitType::NPE { .. } => todo!(),
            VMExitType::AllocateVariableSizeArrayANewArray { .. } => todo!(),
        }
    }



}

pub struct JavaVMStateWrapper<'vm_life> {
    ir: IRVMState<'vm_life>,
    inner: RwLock<JavaVMStateWrapperInner>,
    labeler: Labeler,
}

impl<'vm_life> JavaVMStateWrapper<'vm_life> {
    pub fn add_method(&'vm_life self, resolver: &MethodResolver<'vm_life>, method_id: MethodId) {
        let compressed_code = resolver.get_compressed_code(method_id);
        let CompressedCode {
            instructions,
            max_locals,
            max_stack,
            exception_table,
            stack_map_table
        } = compressed_code;
        let cinstructions = instructions.iter().sorted_by_key(|(offset, _)| **offset).map(|(_, ci)| ci).collect_vec();

        let ir_instructions = compile_to_ir(resolver, cinstructions.as_slice(),&self.labeler);
        let ir_exit_handler = box move |ir_vm_exit_event: &IRVMExitEvent| {
            let ir_num = ExitNumber(ir_vm_exit_event.inner.saved_guest_registers.saved_registers_without_ip.rax as u64);
            let read_guard = self.inner.read().unwrap();
            let vm_exit_type = read_guard.exit_numbers.get(&ir_num).unwrap();
            todo!() as VMExitAction<u64>
        };
        let ir_method_id = self.ir.add_function(ir_instructions, ir_exit_handler);
        let mut write_guard = self.inner.write().unwrap();
        write_guard.method_id_to_ir_method_id.insert(method_id, ir_method_id);
    }
}

fn compile_to_ir(resolver: &MethodResolver<'vm_life>, cinstructions: &[&CompressedInstruction], labeler: &Labeler) -> Vec<IRInstr> {
    let mut initial_ir = vec![];
    let mut labels = vec![];
    for compressed_instruction in cinstructions {
        let current_offset = ByteCodeOffset(compressed_instruction.offset);
        match &compressed_instruction.info {
            CompressedInstructionInfo::invokestatic { method_name, descriptor, classname_ref_type } => {
                match resolver.lookup_static(CPDType::Ref(classname_ref_type.clone()), *method_name, descriptor.clone()) {
                    None => {
                        let exit_label = labeler.new_label(&mut labels);
                        initial_ir.push((
                            current_offset,
                            IRInstr::VMExit {
                                exit_label,
                                exit_type: VMExitType::ResolveInvokeStatic {
                                    method_name: *method_name,
                                    desc: descriptor.clone(),
                                    target_class: CPDType::Ref(classname_ref_type.clone()),
                                },
                            },
                        ));
                    }
                    Some((method_id, is_native)) => {
                        if is_native {
                            let exit_label = labeler.new_label(&mut labels);
                            initial_ir.push((
                                current_offset,
                                IRInstr::VMExit {
                                    exit_label,
                                    exit_type: VMExitType::RunNativeStatic {
                                        method_name: *method_name,
                                        desc: descriptor.clone(),
                                        target_class: CPDType::Ref(classname_ref_type.clone()),
                                    },
                                },
                            ));
                        } else {
                            todo!()
                        }
                    }
                }
            }
            _ => todo!()
        }
    }
    initial_ir.into_iter().map(|(_,ir)|ir).collect_vec()
}