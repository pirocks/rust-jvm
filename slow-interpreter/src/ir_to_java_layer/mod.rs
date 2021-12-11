use std::collections::HashMap;
use std::ffi::c_void;
use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::Deref;
use std::ptr::{NonNull, null_mut};
use std::sync::RwLock;

use iced_x86::CC_b::c;
use iced_x86::CC_np::po;
use iced_x86::ConditionCode::{o, s};
use itertools::Itertools;
use libc::read;

use another_jit_vm::VMExitAction;
use rust_jvm_common::compressed_classfile::code::{CompressedCode, CompressedInstruction, CompressedInstructionInfo};
use rust_jvm_common::compressed_classfile::CPDType;
use rust_jvm_common::runtime_type::RuntimeType;

use crate::{check_loaded_class_force_loader, InterpreterStateGuard, JavaValue, JVMState};
use crate::gc_memory_layout_common::{FramePointerOffset, StackframeMemoryLayout};
use crate::ir_to_java_layer::compiler::{compile_to_ir, JavaCompilerMethodAndFrameData};
use crate::ir_to_java_layer::java_stack::{JavaStackPosition, OpaqueFrameIdOrMethodID, OwnedJavaStack};
use crate::ir_to_java_layer::vm_exit_abi::{AllocateVMExit, VMExitType};
use crate::java_values::GcManagedObject;
use crate::jit::{ByteCodeOffset, MethodResolver};
use crate::jit::ir::{IRInstr, Register};
use crate::jit::state::{Labeler, NaiveStackframeLayout};
use crate::jit_common::java_stack::JavaStack;
use crate::method_table::MethodId;
use crate::native_to_ir_layer::{IRFrameMut, IRFrameRef, IRMethodID, IRVMExitEvent, IRVMState, IRVMStateInner, OwnedIRStack};
use crate::runtime_class::RuntimeClass;

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub struct ExitNumber(u64);

pub struct JavaVMStateWrapperInner<'gc_life> {
    method_id_to_ir_method_id: HashMap<MethodId, IRMethodID>,
    max_exit_number: ExitNumber,
    exit_types: HashMap<ExitNumber, VMExitType>,
    method_exit_handlers: HashMap<ExitNumber, Box<dyn Fn(&'gc_life JVMState<'gc_life>, &mut InterpreterStateGuard<'gc_life, '_>, MethodId, &VMExitType) -> JavaExitAction>>,
}

pub enum JavaExitAction {}

pub enum VMExitEvent<'vm_life> {
    Allocate { size: usize, return_to: *mut c_void },
    TopLevelExitEvent {
        //todo when this stuff is registers can't have gc.
        _return: JavaValue<'vm_life>
    },
}

impl<'gc_life> JavaVMStateWrapperInner<'gc_life> {
    fn handle_vm_exit(&self, jvm: &'gc_life JVMState<'gc_life>, int_state: &mut InterpreterStateGuard<'gc_life, '_>, method_id: MethodId, vm_exit_type: &VMExitType) -> VMExitAction<u64> {
        match vm_exit_type {
            VMExitType::Allocate(AllocateVMExit {}) => {
                let size_register = AllocateVMExit::SIZE;
                let res_register = AllocateVMExit::RES;
                let saved_registers = todo!();
                /*let rc = check_loaded_class_force_loader(jvm, int_state, &ptypeview, loader).unwrap();
                int_state.get_java_stack().saved_registers = save;
                let allocated = match rc.deref() {
                    RuntimeClass::Array(_) => todo!(),
                    RuntimeClass::Object(obj) => JavaValue::new_object(jvm, rc).unwrap(),
                    _ => panic!(),
                };*/
                todo!()
            }
            VMExitType::TopLevelReturn { .. } => {
                VMExitAction::ExitVMCompletely { return_data: todo!() }
            }
            VMExitType::LoadClassAndRecompile => {
                todo!()
            }
            VMExitType::RunStaticNative => {
                todo!()
            }
        }
    }
}

pub struct JavaVMStateWrapper<'vm_life> {
    pub ir: IRVMState<'vm_life>,
    pub inner: RwLock<JavaVMStateWrapperInner<'vm_life>>,
    // should be per thread
    labeler: Labeler,
}

impl<'vm_life> JavaVMStateWrapper<'vm_life> {
    pub fn new() -> Self {
        Self {
            ir: IRVMState::new(),
            inner: RwLock::new(JavaVMStateWrapperInner {
                method_id_to_ir_method_id: Default::default(),
                max_exit_number: ExitNumber(0),
                exit_types: Default::default(),
                method_exit_handlers: Default::default(),
            }),
            labeler: Labeler::new(),
        }
    }

    pub fn add_method(&'vm_life self, jvm: &'vm_life JVMState<'vm_life>, resolver: &MethodResolver<'vm_life>, method_id: MethodId) {
        let mut java_function_frame_guard = jvm.java_function_frame_data.write().unwrap();
        let java_frame_data = &java_function_frame_guard.entry(method_id).or_insert_with(|| JavaCompilerMethodAndFrameData::new(jvm, method_id));
        let ir_instructions = compile_to_ir(resolver, &self.labeler, java_frame_data);
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

    pub fn run_method(&'vm_life self, jvm: &'vm_life JVMState<'vm_life>, java_stack: &OwnedJavaStack, method_id: MethodId, location: JavaStackPosition) -> u64 {
        let ir_method_id = *self.inner.read().unwrap().method_id_to_ir_method_id.get(&method_id).unwrap();
        self.ir.run_method(ir_method_id, &java_stack.inner, match location {
            JavaStackPosition::Frame { frame_pointer } => frame_pointer,
            JavaStackPosition::Top => java_stack.inner.mmaped_top
        })
    }

    pub fn lookup_ir_method_id(&self, opaque_or_not: OpaqueFrameIdOrMethodID) -> IRMethodID {
        match opaque_or_not {
            OpaqueFrameIdOrMethodID::Opaque { opaque_id } => {
                self.ir.lookup_opaque_ir_method_id(opaque_id)
            }
            OpaqueFrameIdOrMethodID::Method { method_id } => {
                let read_guard = self.inner.read().unwrap();
                *read_guard.method_id_to_ir_method_id.get(&(method_id as usize)).unwrap()
            }
        }
    }
}

pub mod compiler;
pub mod java_stack;
pub mod vm_exit_abi;