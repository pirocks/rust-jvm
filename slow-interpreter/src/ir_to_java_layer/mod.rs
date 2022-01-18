use std::collections::HashMap;
use std::ffi::c_void;
use std::hash::Hash;
use std::marker::PhantomData;
use std::mem::{size_of, transmute};
use std::ops::Deref;
use std::ptr::{NonNull, null_mut};
use std::sync::{Arc, RwLock};

use bimap::BiHashMap;
use iced_x86::CC_b::c;
use iced_x86::CC_np::po;
use iced_x86::ConditionCode::{o, s};
use itertools::Itertools;
use libc::read;

use another_jit_vm::{SavedRegistersWithIP, SavedRegistersWithIPDiff, SavedRegistersWithoutIP, SavedRegistersWithoutIPDiff, VMExitAction};
use another_jit_vm_ir::{ExitHandlerType, IRInstructIndex, IRMethodID, IRVMExitAction, IRVMExitEvent, IRVMState};
use another_jit_vm_ir::compiler::{IRInstr, RestartPointID};
use another_jit_vm_ir::ir_stack::{FRAME_HEADER_END_OFFSET, IRStackMut};
use another_jit_vm_ir::vm_exit_abi::{IRVMExitType, RuntimeVMExitInput, VMExitTypeWithArgs};
use rust_jvm_common::compressed_classfile::code::{CompressedCode, CompressedInstruction, CompressedInstructionInfo};
use rust_jvm_common::compressed_classfile::CPDType;
use rust_jvm_common::MethodId;
use rust_jvm_common::runtime_type::RuntimeType;

use crate::{check_initing_or_inited_class, check_loaded_class_force_loader, InterpreterStateGuard, JavaValue, JVMState};
use crate::class_loading::assert_inited_or_initing_class;
use crate::instructions::invoke::native::run_native_method;
use crate::interpreter::FrameToRunOn;
use crate::interpreter_state::FramePushGuard;
use crate::ir_to_java_layer::compiler::{ByteCodeIndex, compile_to_ir, JavaCompilerMethodAndFrameData};
use crate::ir_to_java_layer::java_stack::{JavaStackPosition, OpaqueFrameIdOrMethodID, OwnedJavaStack};
use crate::java::lang::int::Int;
use crate::java_values::{GcManagedObject, NativeJavaValue, StackNativeJavaValue};
use crate::jit::{ByteCodeOffset, MethodResolver, ToIR};
use crate::jit::state::{Labeler, NaiveStackframeLayout, runtime_class_to_allocated_object_type};
use crate::jit_common::java_stack::JavaStack;
use crate::runtime_class::RuntimeClass;
use crate::stack_entry::StackEntryMut;
use crate::utils::run_static_or_virtual;

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub struct ExitNumber(u64);

pub struct JavaVMStateWrapperInner<'gc_life> {
    most_up_to_date_ir_method_id_for_method_id: HashMap<MethodId, IRMethodID>,
    ir_method_id_to_method_id: HashMap<IRMethodID, MethodId>,
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
    fn handle_vm_exit(jvm: &'gc_life JVMState<'gc_life>, int_state: &mut InterpreterStateGuard<'gc_life, 'l>, method_id: MethodId, vm_exit_type: &RuntimeVMExitInput) -> IRVMExitAction {
        match dbg!(vm_exit_type) {
            RuntimeVMExitInput::AllocateObjectArray { type_, len, return_to_ptr, res_address } => {
                eprintln!("AllocateObjectArray");
                let type_ = jvm.cpdtype_table.read().unwrap().get_cpdtype(*type_).unwrap_ref_type().clone();
                assert!(*len > 0);
                let rc = assert_inited_or_initing_class(jvm, CPDType::Ref(type_.clone()));
                let object_array = runtime_class_to_allocated_object_type(rc.as_ref(), int_state.current_loader(jvm), Some(*len as usize), int_state.thread().java_tid);
                let mut memory_region_guard = jvm.gc.memory_region.lock().unwrap();
                let allocated_object = memory_region_guard.find_or_new_region_for(object_array).get_allocation();
                unsafe { res_address.write(allocated_object) }
                IRVMExitAction::RestartAtIndex { index: todo!() }
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
                    unsafe { (*res_ptr as *mut NativeJavaValue<'static>).write(transmute::<NativeJavaValue<'_>, NativeJavaValue<'static>>(res.to_native())) }
                };
                IRVMExitAction::RestartAtIndex { index: todo!() }
            }
            RuntimeVMExitInput::TopLevelReturn { return_value } => {
                eprintln!("TopLevelReturn");
                IRVMExitAction::ExitVMCompletely { return_data: *return_value }
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
                IRVMExitAction::RestartAtIndex { index: todo!() }
            }
            RuntimeVMExitInput::PutStatic { field_id, value_ptr, return_to_ptr } => {
                eprintln!("PutStatic");
                let (rc, field_i) = jvm.field_table.read().unwrap().lookup(*field_id);
                let view = rc.view();
                let field_view = view.field(field_i as usize);
                let mut static_vars_guard = rc.static_vars();
                let static_var = static_vars_guard.get_mut(&field_view.field_name()).unwrap();
                let jv = unsafe { (*value_ptr as *mut NativeJavaValue<'gc_life>).as_ref() }.unwrap().to_java_value(&field_view.field_type(), jvm);
                *static_var = jv;
                IRVMExitAction::RestartAtIndex { index: todo!() }
            }
            RuntimeVMExitInput::InitClassAndRecompile { class_type, current_method_id, restart_point, rbp } => {
                eprintln!("InitClassAndRecompile");
                dbg!(rbp);
                unsafe { dbg!((rbp.offset(-0x38) as *const u64).read()) };
                let cpdtype = jvm.cpdtype_table.read().unwrap().get_cpdtype(*class_type).clone();
                // dbg!(int_state.int_state.as_ref().unwrap().current_stack_position);
                let inited = check_initing_or_inited_class(jvm, int_state, cpdtype).unwrap();
                // dbg!(int_state.int_state.as_ref().unwrap().current_stack_position);
                let method_resolver = MethodResolver { jvm, loader: int_state.current_loader(jvm) };
                jvm.java_vm_state.add_method(jvm, &method_resolver, *current_method_id);
                let restart_point = jvm.java_vm_state.lookup_restart_point(*current_method_id, *restart_point);
                IRVMExitAction::RestartAtIndex { index: todo!() }
            }
            RuntimeVMExitInput::AllocatePrimitiveArray { .. } => todo!()
        }
    }
}

pub struct JavaVMStateWrapper<'vm_life> {
    pub ir: IRVMState<'vm_life, ()>,
    pub inner: RwLock<JavaVMStateWrapperInner<'vm_life>>,
    // should be per thread
    labeler: Labeler,
}

impl<'vm_life> JavaVMStateWrapper<'vm_life> {
    pub fn new() -> Self {
        let mut res = Self {
            ir: IRVMState::new(),
            inner: RwLock::new(JavaVMStateWrapperInner {
                most_up_to_date_ir_method_id_for_method_id: Default::default(),
                ir_method_id_to_method_id: Default::default(),
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
        //&IRVMExitEvent, IRStackMut, &IRVMState<'vm_life, ExtraData>, &mut ExtraData
        let (ir_method_id, restart_points) = self.ir.add_function(vec![IRInstr::VMExit2 { exit_type: IRVMExitType::TopLevelReturn {} }], FRAME_HEADER_END_OFFSET, box |event, ir_stack_mut, ir_vm_state, extra| {
            match &event.exit_type {
                RuntimeVMExitInput::TopLevelReturn { return_value } => IRVMExitAction::ExitVMCompletely { return_data: *return_value },
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
        // &IRVMExitEvent, IRStackMut, &IRVMState<'vm_life, ExtraData>, &mut ExtraData
        let ir_exit_handler: ExitHandlerType<'vm_life, ()> = box move |ir_vm_exit_event: &IRVMExitEvent, ir_stack_mut: IRStackMut, ir_vm_state: &IRVMState<'vm_life, ()>, extra| {
            let ir_stack_mut: IRStackMut = ir_stack_mut;
            let mut int_state = InterpreterStateGuard::LocalInterpreterState {
                int_state: ir_stack_mut,
                thread: jvm.thread_state.get_current_thread(),
                registered: false,
                jvm,
            };
            int_state.register_interpreter_state_guard(jvm);
            let frame_ptr = ir_vm_exit_event.inner.saved_guest_registers.saved_registers_without_ip.rbp;
            let ir_num = ExitNumber(ir_vm_exit_event.inner.saved_guest_registers.saved_registers_without_ip.rax as u64);
            let read_guard = self.inner.read().unwrap();
            let ir_method_id = ir_vm_exit_event.ir_method;
            let method_id = *read_guard.ir_method_id_to_method_id.get(&ir_method_id).unwrap();
            drop(read_guard);
            JavaVMStateWrapperInner::handle_vm_exit(jvm, &mut int_state, method_id, &ir_vm_exit_event.exit_type)
        };
        let (ir_method_id, restart_points) = self.ir.add_function(ir_instructions, java_frame_data.full_frame_size(), ir_exit_handler);
        let mut write_guard = self.inner.write().unwrap();
        write_guard.most_up_to_date_ir_method_id_for_method_id.insert(method_id, ir_method_id);
        write_guard.ir_method_id_to_method_id.insert(ir_method_id, method_id);
        write_guard.restart_points.insert(ir_method_id, restart_points);
    }

    pub fn run_method(&'vm_life self, jvm: &'vm_life JVMState<'vm_life>, int_state: &'_ mut InterpreterStateGuard<'vm_life, 'l>, method_id: MethodId) -> u64 {
        let ir_method_id = *self.inner.read().unwrap().most_up_to_date_ir_method_id_for_method_id.get(&method_id).unwrap();
        let mut frame_to_run_on = int_state.current_frame_mut();
        let frame_ir_method_id = frame_to_run_on.frame_view.ir_mut.downgrade().ir_method_id().unwrap();
        assert_eq!(self.inner.read().unwrap().ir_method_id_to_method_id.get(&frame_ir_method_id).unwrap(), &method_id);
        if frame_ir_method_id != ir_method_id {
            frame_to_run_on.frame_view.ir_mut.set_ir_method_id(ir_method_id);
        }
        assert!(jvm.thread_state.int_state_guard_valid.with(|refcell| { *refcell.borrow() }));

        self.ir.run_method(ir_method_id, &mut frame_to_run_on.frame_view.ir_mut, &mut ())
    }

    pub fn lookup_ir_method_id(&self, opaque_or_not: OpaqueFrameIdOrMethodID) -> IRMethodID {
        self.try_lookup_ir_method_id(opaque_or_not).unwrap()
    }

    pub fn lookup_method_ir_method_id(&self, method_id: MethodId) -> IRMethodID {
        let inner = self.inner.read().unwrap();
        *inner.most_up_to_date_ir_method_id_for_method_id.get(&method_id).unwrap()
    }

    pub fn try_lookup_ir_method_id(&self, opaque_or_not: OpaqueFrameIdOrMethodID) -> Option<IRMethodID> {
        match opaque_or_not {
            OpaqueFrameIdOrMethodID::Opaque { opaque_id } => {
                Some(self.ir.lookup_opaque_ir_method_id(opaque_id))
            }
            OpaqueFrameIdOrMethodID::Method { method_id } => {
                let read_guard = self.inner.read().unwrap();
                read_guard.most_up_to_date_ir_method_id_for_method_id.get(&(method_id as usize)).cloned()
            }
        }
    }

    pub fn lookup_restart_point(&self, method_id: MethodId, restart_point_id: RestartPointID) -> *const c_void {
        let read_guard = self.inner.read().unwrap();
        let ir_method_id = *read_guard.most_up_to_date_ir_method_id_for_method_id.get(&method_id).unwrap();
        let restart_points = read_guard.restart_points.get(&ir_method_id).unwrap();
        let ir_instruct_index = *restart_points.get(&restart_point_id).unwrap();
        drop(read_guard);
        self.ir.lookup_location_of_ir_instruct(ir_method_id, ir_instruct_index).0
    }
}

pub mod compiler;
pub mod java_stack;
pub mod vm_exit_abi;