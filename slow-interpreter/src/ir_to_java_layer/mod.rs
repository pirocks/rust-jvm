use std::collections::HashMap;
use std::ffi::c_void;
use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::Deref;
use std::ptr::{NonNull, null_mut};
use std::sync::RwLock;

use bimap::BiHashMap;
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
use crate::instructions::invoke::native::run_native_method;
use crate::ir_to_java_layer::compiler::{compile_to_ir, JavaCompilerMethodAndFrameData};
use crate::ir_to_java_layer::java_stack::{JavaStackPosition, OpaqueFrameIdOrMethodID, OwnedJavaStack};
use crate::ir_to_java_layer::vm_exit_abi::{AllocateVMExit, RuntimeVMExitInput, VMExitTypeWithArgs};
use crate::java::lang::int::Int;
use crate::java_values::{GcManagedObject, NativeJavaValue, StackNativeJavaValue};
use crate::jit::{ByteCodeOffset, MethodResolver};
use crate::jit::ir::IRInstr;
use crate::jit::state::{Labeler, NaiveStackframeLayout};
use crate::jit_common::java_stack::JavaStack;
use crate::method_table::MethodId;
use crate::native_to_ir_layer::{IRFrameMut, IRFrameRef, IRMethodID, IRVMExitEvent, IRVMState, IRVMStateInner, OwnedIRStack};
use crate::runtime_class::RuntimeClass;
use crate::utils::run_static_or_virtual;

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub struct ExitNumber(u64);

pub struct JavaVMStateWrapperInner<'gc_life> {
    method_id_to_ir_method_id: BiHashMap<MethodId, IRMethodID>,
    max_exit_number: ExitNumber,
    // exit_types: HashMap<ExitNumber, VMExitTypeWithArgs>,
    method_exit_handlers: HashMap<ExitNumber, Box<dyn for<'l> Fn(&'gc_life JVMState<'gc_life>, &mut InterpreterStateGuard<'l ,'gc_life>, MethodId, &VMExitTypeWithArgs) -> JavaExitAction>>,
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
    fn handle_vm_exit(&self, jvm: &'gc_life JVMState<'gc_life>, int_state: &mut InterpreterStateGuard<'gc_life,'l>, method_id: MethodId, vm_exit_type: &RuntimeVMExitInput) -> VMExitAction<u64> {
        match vm_exit_type {
            RuntimeVMExitInput::Allocate { type_, return_to_ptr } => {
                todo!()
            }
            RuntimeVMExitInput::LoadClassAndRecompile { .. } => todo!(),
            RuntimeVMExitInput::RunStaticNative { method_id, arg_start, num_args, return_to_ptr } => {
                let (rc, method_i) = jvm.method_table.read().unwrap().try_lookup(*method_id).unwrap();
                let mut args_jv = vec![];
                let class_view = rc.view();
                let method_view = class_view.method_view_i(method_i);
                let arg_types = &method_view.desc().arg_types;
                unsafe {
                    for (i,cpdtype) in (0..*num_args).zip(arg_types.iter()) {
                        let arg_ptr = arg_start.offset(-(i as isize)) as *const u64;//stack grows down
                        let native_jv = NativeJavaValue { as_u64: arg_ptr.read() };
                        args_jv.push(native_jv.to_java_value(cpdtype, jvm))
                    }
                }
                run_native_method(jvm, int_state, rc, method_i, args_jv).unwrap();
                todo!()
            }
            RuntimeVMExitInput::TopLevelReturn => todo!()
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
                // exit_types: Default::default(),
                method_exit_handlers: Default::default(),
            }),
            labeler: Labeler::new(),
        }
    }

    pub fn add_method(&'vm_life self, jvm: &'vm_life JVMState<'vm_life>, resolver: &MethodResolver<'vm_life>, method_id: MethodId) {
        let mut java_function_frame_guard = jvm.java_function_frame_data.write().unwrap();
        let java_frame_data = &java_function_frame_guard.entry(method_id).or_insert_with(|| JavaCompilerMethodAndFrameData::new(jvm, method_id));
        let ir_instructions = compile_to_ir(resolver, &self.labeler, java_frame_data);
        let ir_exit_handler = box move |ir_vm_exit_event: &IRVMExitEvent, int_state: &mut InterpreterStateGuard<'vm_life, '_>| {
            let frame_ptr = ir_vm_exit_event.inner.saved_guest_registers.saved_registers_without_ip.rbp;
            let ir_num = ExitNumber(ir_vm_exit_event.inner.saved_guest_registers.saved_registers_without_ip.rax as u64);
            let read_guard = self.inner.read().unwrap();
            let ir_method_id = ir_vm_exit_event.ir_method;
            let method_id = *read_guard.method_id_to_ir_method_id.get_by_right(&ir_method_id).unwrap();
            // let vm_exit_type = read_guard.exit_types.get(&ir_num).unwrap();
            read_guard.handle_vm_exit(jvm, int_state, method_id, &ir_vm_exit_event.exit_type) as VMExitAction<u64>
        };
        let ir_method_id = self.ir.add_function(ir_instructions, ir_exit_handler);
        let mut write_guard = self.inner.write().unwrap();
        write_guard.method_id_to_ir_method_id.insert(method_id, ir_method_id);
    }

    pub fn run_method(&'vm_life self, jvm: &'vm_life JVMState<'vm_life>, int_state: &mut InterpreterStateGuard<'vm_life,'_>, method_id: MethodId, location: JavaStackPosition) -> u64 {
        let ir_method_id = *self.inner.read().unwrap().method_id_to_ir_method_id.get_by_left(&method_id).unwrap();
        let mmapped_top = int_state.java_stack().inner.mmaped_top;
        self.ir.run_method(ir_method_id, int_state, match location {
            JavaStackPosition::Frame { frame_pointer } => frame_pointer,
            JavaStackPosition::Top => mmapped_top
        })
    }

    pub fn lookup_ir_method_id(&self, opaque_or_not: OpaqueFrameIdOrMethodID) -> IRMethodID {
        match opaque_or_not {
            OpaqueFrameIdOrMethodID::Opaque { opaque_id } => {
                self.ir.lookup_opaque_ir_method_id(opaque_id)
            }
            OpaqueFrameIdOrMethodID::Method { method_id } => {
                let read_guard = self.inner.read().unwrap();
                *read_guard.method_id_to_ir_method_id.get_by_left(&(method_id as usize)).unwrap()
            }
        }
    }
}

pub mod compiler;
pub mod java_stack;
pub mod vm_exit_abi;