use std::collections::HashMap;
use std::ffi::c_void;
use std::marker::PhantomData;
use std::mem::size_of;
use std::ptr::{NonNull, null_mut};
use std::sync::RwLock;

use iced_x86::ConditionCode::o;
use itertools::Itertools;

use another_jit_vm::VMExitAction;
use rust_jvm_common::compressed_classfile::code::{CompressedCode, CompressedInstruction, CompressedInstructionInfo};
use rust_jvm_common::compressed_classfile::CPDType;
use rust_jvm_common::runtime_type::RuntimeType;

use crate::{InterpreterStateGuard, JavaValue, JVMState};
use crate::gc_memory_layout_common::FramePointerOffset;
use crate::java_values::GcManagedObject;
use crate::jit::{ByteCodeOffset, MethodResolver, VMExitType};
use crate::jit::ir::IRInstr;
use crate::jit::state::Labeler;
use crate::method_table::MethodId;
use crate::native_to_ir_layer::{IRFrameMut, IRFrameRef, IRMethodID, IRStack, IRVMExitEvent, IRVMState, IRVMStateInner};

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub struct ExitNumber(u64);

pub struct JavaVMStateWrapperInner<'gc_life> {
    method_id_to_ir_method_id: HashMap<MethodId, IRMethodID>,
    max_exit_number: ExitNumber,
    exit_types: HashMap<ExitNumber, VMExitType>,
    method_exit_handlers: HashMap<ExitNumber, Box<dyn Fn(&'gc_life JVMState<'gc_life>, &mut InterpreterStateGuard<'gc_life, '_>, MethodId, &VMExitType) -> JavaExitAction>>,
}

pub enum JavaExitAction {}

impl<'gc_life> JavaVMStateWrapperInner<'gc_life> {
    fn handle_vm_exit(&self, jvm: &'gc_life JVMState<'gc_life>, java_stack: &mut JavaStack2, method_id: MethodId, vm_exit_type: &VMExitType) -> VMExitAction<u64> {
        match vm_exit_type {
            VMExitType::ResolveInvokeStatic { .. } => todo!(),
            VMExitType::RunNativeStatic { .. } => todo!(),
            VMExitType::ResolveInvokeSpecial { .. } => todo!(),
            VMExitType::InvokeSpecialNative { .. } => todo!(),
            VMExitType::InitClass { .. } => todo!(),
            VMExitType::NeedNewRegion { .. } => todo!(),
            VMExitType::PutStatic { .. } => todo!(),
            VMExitType::Allocate { res, loader, bytecode_size, ptypeview } => {
                todo!()
            }
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
    inner: RwLock<JavaVMStateWrapperInner<'vm_life>>,
    labeler: Labeler,
}

impl<'vm_life> JavaVMStateWrapper<'vm_life> {
    pub fn add_method(&'vm_life self, jvm: &'vm_life JVMState<'vm_life>, resolver: &MethodResolver<'vm_life>, method_id: MethodId) {
        let compressed_code = resolver.get_compressed_code(method_id);
        let CompressedCode {
            instructions,
            max_locals,
            max_stack,
            exception_table,
            stack_map_table
        } = compressed_code;
        let cinstructions = instructions.iter().sorted_by_key(|(offset, _)| **offset).map(|(_, ci)| ci).collect_vec();

        let ir_instructions = compile_to_ir(resolver, cinstructions.as_slice(), &self.labeler);
        let ir_exit_handler = box move |ir_vm_exit_event: &IRVMExitEvent| {
            let frame_ptr = ir_vm_exit_event.inner.saved_guest_registers.saved_registers_without_ip.rbp;
            let ir_num = ExitNumber(ir_vm_exit_event.inner.saved_guest_registers.saved_registers_without_ip.rax as u64);
            let read_guard = self.inner.read().unwrap();
            let vm_exit_type = read_guard.exit_types.get(&ir_num).unwrap();
            read_guard.handle_vm_exit(jvm, todo!(), todo!(), vm_exit_type) as VMExitAction<u64>
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
    initial_ir.into_iter().map(|(_, ir)| ir).collect_vec()
}

pub struct JavaStack2<'vm_life, 'ir_vm_life, 'native_vm_life> {
    java_vm_state: &'vm_life JavaVMStateWrapperInner<'vm_life>,
    inner: IRStack<'ir_vm_life, 'native_vm_life>,
}


impl<'vm_life, 'ir_vm_life, 'native_vm_life> JavaStack2<'vm_life, 'ir_vm_life, 'native_vm_life> {
    pub fn frame_at(&self, frame_pointer: *mut c_void, jvm: &'vm_life JVMState<'vm_life>) -> RuntimeJavaStackFrameRef<'_, 'vm_life, 'ir_vm_life, 'native_vm_life> {
        let ir_frame = unsafe { self.inner.frame_at(frame_pointer) };
        let ir_method_id = ir_frame.ir_method_id();
        let method_id = ir_frame.method_id();
        let ir_method_id_2 = self.java_vm_state.method_id_to_ir_method_id.get(&method_id).unwrap();
        assert_eq!(ir_method_id_2, &ir_method_id);
        RuntimeJavaStackFrameRef {
            frame_ptr: frame_pointer,
            ir_ref: ir_frame,
            jvm,
            max_locals: jvm.max_locals_by_method_id(method_id),
        }
    }

    pub fn mut_frame_at(&mut self, frame_pointer: *mut c_void, jvm: &'vm_life JVMState<'vm_life>) -> RuntimeJavaStackFrameMut<'_, 'vm_life, 'ir_vm_life, 'native_vm_life> {
        let ir_frame = unsafe { self.inner.frame_at(frame_pointer) };
        let ir_method_id = ir_frame.ir_method_id();
        let method_id = ir_frame.method_id();
        let ir_method_id_2 = *self.java_vm_state.method_id_to_ir_method_id.get(&method_id).unwrap();
        assert_eq!(ir_method_id_2, ir_method_id);
        let ir_frame_mut = unsafe { self.inner.frame_at_mut(frame_pointer) };
        RuntimeJavaStackFrameMut {
            frame_ptr: frame_pointer,
            ir_mut: ir_frame_mut,
            jvm,
            max_locals: jvm.max_locals_by_method_id(method_id),
        }
    }
}

pub struct RuntimeJavaStackFrameRef<'l, 'vm_life, 'ir_vm_life, 'native_vm_life> {
    frame_ptr: *const c_void,
    ir_ref: IRFrameRef<'l, 'ir_vm_life, 'native_vm_life>,
    jvm: &'vm_life JVMState<'vm_life>,
    max_locals: u16,
}

impl<'vm_life> RuntimeJavaStackFrameRef<'_, 'vm_life, '_, '_> {
    pub fn method_id(&self) -> MethodId {
        self.ir_ref.method_id()
    }

    fn read_target(&self, offset: FramePointerOffset, rtype: RuntimeType) -> JavaValue<'vm_life> {
        let res = self.ir_ref.read_at_offset(offset);
        match rtype {
            RuntimeType::IntType => JavaValue::Int(res as i32),
            RuntimeType::FloatType => JavaValue::Float(f32::from_le_bytes((res as u32).to_le_bytes())),
            RuntimeType::DoubleType => JavaValue::Double(f64::from_le_bytes((res as f64).to_le_bytes())),
            RuntimeType::LongType => JavaValue::Long(res as i64),
            RuntimeType::Ref(ref_) => {
                let ptr = res as *mut c_void;
                JavaValue::Object(NonNull::new(ptr).map(|nonnull| GcManagedObject::from_native(nonnull, self.jvm)))
            }
            RuntimeType::TopType => {
                panic!()
            }
        }
    }

    pub fn nth_operand_stack_member(&self, n: usize, rtype: RuntimeType) -> JavaValue<'vm_life> {
        let offset = FramePointerOffset(self.max_locals as usize * size_of::<u64>() + n * size_of::<u64>());
        self.read_target(offset, rtype)
    }

    pub fn nth_local(&self, n: usize, rtype: RuntimeType) -> JavaValue<'vm_life> {
        let offset = FramePointerOffset(n * size_of::<u64>());
        self.read_target(offset, rtype)
    }
}

pub struct RuntimeJavaStackFrameMut<'l, 'vm_life, 'ir_vm_life, 'native_vm_life> {
    frame_ptr: *const c_void,
    ir_mut: IRFrameMut<'l, 'ir_vm_life, 'native_vm_life>,
    jvm: &'vm_life JVMState<'vm_life>,
    max_locals: u16,
}

impl<'k, 'l, 'vm_life, 'ir_vm_life, 'native_vm_life> RuntimeJavaStackFrameMut<'l, 'vm_life, 'ir_vm_life, 'native_vm_life> {
    pub fn downgrade(self) -> RuntimeJavaStackFrameRef<'l, 'vm_life, 'ir_vm_life, 'native_vm_life> {
        RuntimeJavaStackFrameRef {
            frame_ptr: self.frame_ptr,
            ir_ref: self.ir_mut.downgrade(),
            jvm: self.jvm,
            max_locals: self.max_locals,
        }
    }

    fn write_target(&mut self, offset: FramePointerOffset, jv: JavaValue<'vm_life>) {
        let to_write = match jv {
            JavaValue::Long(long) => { long as u64 }
            JavaValue::Int(int) => { int as u64 }
            JavaValue::Short(short) => { short as u64 }
            JavaValue::Byte(byte) => { byte as u64 }
            JavaValue::Boolean(boolean) => { boolean as u64 }
            JavaValue::Char(char) => { char as u64 }
            JavaValue::Float(float) => { u32::from_le_bytes(float.to_le_bytes()) as u64 }
            JavaValue::Double(double) => { u64::from_le_bytes(double.to_le_bytes()) }
            JavaValue::Object(obj) => {
                match obj {
                    None => 0u64,
                    Some(obj) => {
                        obj.raw_ptr_usize() as u64
                    }
                }
            }
            JavaValue::Top => {
                panic!()
            }
        };
        self.ir_mut.write_at_offset(offset, to_write);
    }

    pub fn set_nth_local(&mut self, n: usize, jv: JavaValue<'vm_life>) {
        let offset = FramePointerOffset(n * size_of::<u64>());
    }
}