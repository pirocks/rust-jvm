use std::collections::HashMap;
use std::ffi::c_void;
use std::marker::PhantomData;
use std::mem::{size_of, transmute};
use std::ops::Deref;
use std::ptr::{NonNull, null_mut};
use std::sync::RwLock;

use bimap::BiHashMap;
use iced_x86::CC_b::c;
use iced_x86::CC_np::po;
use iced_x86::ConditionCode::{o, s};
use itertools::Itertools;
use libc::read;

use another_jit_vm::{SavedRegistersWithIP, SavedRegistersWithIPDiff, SavedRegistersWithoutIP, SavedRegistersWithoutIPDiff, VMExitAction};
use rust_jvm_common::compressed_classfile::code::{CompressedCode, CompressedInstruction, CompressedInstructionInfo};
use rust_jvm_common::compressed_classfile::CPDType;
use rust_jvm_common::runtime_type::RuntimeType;

use crate::{check_initing_or_inited_class, check_loaded_class_force_loader, InterpreterStateGuard, JavaValue, JVMState};
use crate::class_loading::assert_inited_or_initing_class;
use crate::gc_memory_layout_common::{AllocatedObjectType, FramePointerOffset, StackframeMemoryLayout};
use crate::instructions::invoke::native::run_native_method;
use crate::interpreter::FrameToRunOn;
use crate::ir_to_java_layer::compiler::{ByteCodeIndex, compile_to_ir, JavaCompilerMethodAndFrameData};
use crate::ir_to_java_layer::java_stack::{JavaStackPosition, OpaqueFrameIdOrMethodID, OwnedJavaStack};
use crate::ir_to_java_layer::vm_exit_abi::{AllocateVMExit, IRVMExitType, RestartPointID, RuntimeVMExitInput, VMExitTypeWithArgs};
use crate::java::lang::int::Int;
use crate::java_values::{GcManagedObject, NativeJavaValue, StackNativeJavaValue};
use crate::jit::{ByteCodeOffset, MethodResolver, ToIR};
use crate::jit::ir::IRInstr;
use crate::jit::state::{Labeler, NaiveStackframeLayout, runtime_class_to_allocated_object_type};
use crate::jit_common::java_stack::JavaStack;
use crate::method_table::MethodId;
use crate::native_to_ir_layer::{IRFrameMut, IRFrameRef, IRInstructIndex, IRMethodID, IRVMExitEvent, IRVMState, IRVMStateInner, OwnedIRStack};
use crate::runtime_class::RuntimeClass;
use crate::utils::run_static_or_virtual;

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub struct ExitNumber(u64);

pub struct JavaVMStateWrapperInner<'gc_life> {
    method_id_to_ir_method_id: BiHashMap<MethodId, IRMethodID>,
    restart_points: HashMap<IRMethodID, HashMap<RestartPointID, IRInstructIndex>>,
    max_exit_number: ExitNumber,
    method_exit_handlers: HashMap<ExitNumber, Box<dyn for<'l> Fn(&'gc_life JVMState<'gc_life>, &mut InterpreterStateGuard<'l, 'gc_life>, MethodId, &VMExitTypeWithArgs) -> JavaExitAction>>,
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
    fn handle_vm_exit(jvm: &'gc_life JVMState<'gc_life>, int_state: &mut InterpreterStateGuard<'gc_life, 'l>, method_id: MethodId, vm_exit_type: &RuntimeVMExitInput) -> VMExitAction<u64> {
        match dbg!(vm_exit_type) {
            RuntimeVMExitInput::AllocateObjectArray { type_, len, return_to_ptr, res_address } => {
                eprintln!("AllocateObjectArray");
                let type_ = jvm.cpdtype_table.read().unwrap().get_cpdtype(*type_).unwrap_ref_type().clone();
                assert!(*len > 0);
                let rc = assert_inited_or_initing_class(jvm, CPDType::Ref(type_.clone()));
                let object_array = runtime_class_to_allocated_object_type(rc.as_ref(), int_state.current_loader(jvm), Some(*len as usize), int_state.thread.java_tid);
                let mut memory_region_guard = jvm.gc.memory_region.lock().unwrap();
                let allocated_object = memory_region_guard.find_or_new_region_for(object_array).get_allocation();
                unsafe { res_address.write(allocated_object) }
                VMExitAction::ReturnTo { return_register_state: SavedRegistersWithIPDiff { rip: Some(*return_to_ptr), saved_registers_without_ip: None } }
            }
            RuntimeVMExitInput::LoadClassAndRecompile { .. } => todo!(),
            RuntimeVMExitInput::RunStaticNative { method_id, arg_start, num_args, res_ptr, return_to_ptr } => {
                eprintln!("RunStaticNative");
                let (rc, method_i) = jvm.method_table.read().unwrap().try_lookup(*method_id).unwrap();
                let mut args_jv = vec![];
                let class_view = rc.view();
                let method_view = class_view.method_view_i(method_i);
                let arg_types = &method_view.desc().arg_types;
                unsafe {
                    for (i, cpdtype) in (0..*num_args).zip(arg_types.iter()) {
                        let arg_ptr = arg_start.offset(-(i as isize)) as *const u64;//stack grows down
                        let native_jv = NativeJavaValue { as_u64: arg_ptr.read() };
                        args_jv.push(native_jv.to_java_value(cpdtype, jvm))
                    }
                }
                assert!(jvm.thread_state.int_state_guard_valid.with(|refcell| { *refcell.borrow() }));
                let res = run_native_method(jvm, int_state, rc, method_i, args_jv).unwrap();
                if let Some(res) = res {
                    unsafe { (*res_ptr).write(transmute::<NativeJavaValue<'_>, NativeJavaValue<'static>>(res.to_native())) }
                };
                VMExitAction::ReturnTo { return_register_state: SavedRegistersWithIPDiff { rip: Some(*return_to_ptr), saved_registers_without_ip: None } }
            }
            RuntimeVMExitInput::TopLevelReturn { return_value } => {
                eprintln!("TopLevelReturn");
                VMExitAction::ExitVMCompletely { return_data: *return_value }
            }
            RuntimeVMExitInput::CompileFunctionAndRecompileCurrent {
                current_method_id,
                to_recompile,
                restart_point
            } => {
                eprintln!("CompileFunctionAndRecompileCurrent");
                let method_resolver = MethodResolver { jvm, loader: int_state.current_loader(jvm) };
                jvm.java_vm_state.add_method(jvm, &method_resolver, *to_recompile);
                jvm.java_vm_state.add_method(jvm, &method_resolver, *current_method_id);
                let restart_point = jvm.java_vm_state.lookup_restart_point(*current_method_id, *restart_point);
                VMExitAction::ReturnTo { return_register_state: SavedRegistersWithIPDiff { rip: Some(restart_point), saved_registers_without_ip: None } }
            }
            RuntimeVMExitInput::PutStatic { field_id, value, return_to_ptr } => {
                eprintln!("PutStatic");
                let (rc, field_i) = jvm.field_table.read().unwrap().lookup(*field_id);
                let view = rc.view();
                let field_view = view.field(field_i as usize);
                let mut static_vars_guard = rc.static_vars();
                let static_var = static_vars_guard.get_mut(&field_view.field_name()).unwrap();
                let jv = unsafe { value.as_ref() }.unwrap().to_java_value(&field_view.field_type(), jvm);
                *static_var = jv;
                VMExitAction::ReturnTo { return_register_state: SavedRegistersWithIPDiff { rip: Some(*return_to_ptr), saved_registers_without_ip: None } }
            }
            RuntimeVMExitInput::InitClassAndRecompile { class_type, current_method_id, restart_point, rbp } => {
                eprintln!("InitClassAndRecompile");
                dbg!(rbp);
                unsafe { dbg!((rbp.offset(-0x38) as *const u64).read()) };
                let cpdtype = jvm.cpdtype_table.read().unwrap().get_cpdtype(*class_type).clone();
                dbg!(int_state.int_state.as_ref().unwrap().current_stack_position);
                let inited = check_initing_or_inited_class(jvm, int_state, cpdtype).unwrap();
                dbg!(int_state.int_state.as_ref().unwrap().current_stack_position);
                let method_resolver = MethodResolver { jvm, loader: int_state.current_loader(jvm) };
                jvm.java_vm_state.add_method(jvm, &method_resolver, *current_method_id);
                let restart_point = jvm.java_vm_state.lookup_restart_point(*current_method_id, *restart_point);
                VMExitAction::ReturnTo {
                    return_register_state: SavedRegistersWithIPDiff {
                        rip: Some(restart_point),
                        saved_registers_without_ip: None,
                    }
                }
            }
            RuntimeVMExitInput::AllocatePrimitiveArray { .. } => todo!()
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
        let mut res = Self {
            ir: IRVMState::new(),
            inner: RwLock::new(JavaVMStateWrapperInner {
                method_id_to_ir_method_id: Default::default(),
                restart_points: Default::default(),
                max_exit_number: ExitNumber(0),
                // exit_types: Default::default(),
                method_exit_handlers: Default::default(),
            }),
            labeler: Labeler::new(),
        };
        res
    }

    pub fn add_top_level_vm_exit(&'vm_life self) {
        let (ir_method_id, restart_points) = self.ir.add_function(vec![IRInstr::VMExit2 { exit_type: IRVMExitType::TopLevelReturn {} }], 0, box |event, _int_state| {
            match &event.exit_type {
                RuntimeVMExitInput::TopLevelReturn { return_value } => VMExitAction::ExitVMCompletely { return_data: *return_value },
                _ => panic!()
            }
        });
        assert!(restart_points.is_empty());
        self.ir.init_top_level_exit_id(ir_method_id)
    }

    pub fn add_method(&'vm_life self, jvm: &'vm_life JVMState<'vm_life>, resolver: &MethodResolver<'vm_life>, method_id: MethodId) {
        eprintln!("Re/Compile: {}", method_id);
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
            drop(read_guard);
            JavaVMStateWrapperInner::handle_vm_exit(jvm, int_state, method_id, &ir_vm_exit_event.exit_type) as VMExitAction<u64>
        };
        let (ir_method_id, restart_points) = self.ir.add_function(ir_instructions, java_frame_data.full_frame_size(), ir_exit_handler);
        let mut write_guard = self.inner.write().unwrap();
        write_guard.method_id_to_ir_method_id.insert(method_id, ir_method_id);
        write_guard.restart_points.insert(ir_method_id, restart_points);
    }

    pub fn run_method(&'vm_life self, jvm: &'vm_life JVMState<'vm_life>, int_state: &'_ mut InterpreterStateGuard<'vm_life, 'l>, method_id: MethodId, frame_to_run_on: FrameToRunOn) -> u64 {
        let ir_method_id = *self.inner.read().unwrap().method_id_to_ir_method_id.get_by_left(&method_id).unwrap();
        let mmapped_top = int_state.java_stack().inner.mmaped_top;
        assert!(jvm.thread_state.int_state_guard_valid.with(|refcell| { *refcell.borrow() }));
        //todo calculate stack pointer based of frame size
        let frame_pointer = match frame_to_run_on.frame_pointer {
            JavaStackPosition::Frame { frame_pointer } => frame_pointer,
            JavaStackPosition::Top => mmapped_top
        };
        let stack_pointer = unsafe { frame_pointer.sub(frame_to_run_on.size) };
        self.ir.run_method(ir_method_id, int_state, frame_pointer, stack_pointer)
    }

    pub fn lookup_ir_method_id(&self, opaque_or_not: OpaqueFrameIdOrMethodID) -> IRMethodID {
        self.try_lookup_ir_method_id(opaque_or_not).unwrap()
    }

    pub fn try_lookup_ir_method_id(&self, opaque_or_not: OpaqueFrameIdOrMethodID) -> Option<IRMethodID> {
        match opaque_or_not {
            OpaqueFrameIdOrMethodID::Opaque { opaque_id } => {
                Some(self.ir.lookup_opaque_ir_method_id(opaque_id))
            }
            OpaqueFrameIdOrMethodID::Method { method_id } => {
                let read_guard = self.inner.read().unwrap();
                read_guard.method_id_to_ir_method_id.get_by_left(&(method_id as usize)).cloned()
            }
        }
    }

    pub fn lookup_restart_point(&self, method_id: MethodId, restart_point_id: RestartPointID) -> *const c_void {
        let read_guard = self.inner.read().unwrap();
        let ir_method_id = *read_guard.method_id_to_ir_method_id.get_by_left(&method_id).unwrap();
        let restart_points = read_guard.restart_points.get(&ir_method_id).unwrap();
        let ir_instruct_index = *restart_points.get(&restart_point_id).unwrap();
        drop(read_guard);
        self.ir.lookup_location_of_ir_instruct(ir_method_id, ir_instruct_index).0
    }
}

pub mod compiler;
pub mod java_stack;
pub mod vm_exit_abi;