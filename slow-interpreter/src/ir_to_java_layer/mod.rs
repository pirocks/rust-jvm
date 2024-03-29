use std::collections::HashMap;
use std::ffi::c_void;
use std::hash::Hash;

use another_jit_vm::IRMethodID;
use another_jit_vm::saved_registers_utils::{SavedRegistersWithIPDiff, SavedRegistersWithoutIPDiff};
use another_jit_vm_ir::{IRInstructIndex, IRVMExitAction};
use another_jit_vm_ir::compiler::RestartPointID;
use another_jit_vm_ir::vm_exit_abi::runtime_input::RuntimeVMExitInput;
use runtime_class_stuff::method_numbers::MethodNumber;
use rust_jvm_common::{ByteCodeOffset, MethodId};

use crate::{JavaValue, JVMState, PushableFrame, WasException};
use crate::better_java_stack::exit_frame::JavaExitFrame;
use crate::better_java_stack::frames::HasFrame;
use crate::interpreter::run_function_interpreted;
use crate::ir_to_java_layer::exit_impls::multi_allocate_array::multi_allocate_array;
use crate::ir_to_java_layer::exit_impls::new_run_native::{run_native_special_new, run_native_static_new};
use crate::ir_to_java_layer::exit_impls::throw_impl;
use crate::new_java_values::owned_casts::OwnedCastAble;
use crate::stdlib::java::lang::class_cast_exception::ClassCastException;
use crate::stdlib::java::lang::null_pointer_exception::NullPointerException;
use crate::stdlib::java::NewAsObjectOrJavaValue;

pub mod java_stack;

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub struct ExitNumber(u64);

pub struct ByteCodeIRMapping {
    ir_index_to_bytecode_pc: HashMap<IRInstructIndex, ByteCodeOffset>,
    bytecode_pc_to_start_ir_index: HashMap<ByteCodeOffset, IRInstructIndex>,
}

pub struct JavaVMStateMethod {
    restart_points: HashMap<RestartPointID, IRInstructIndex>,
    byte_code_ir_mapping: Option<ByteCodeIRMapping>,
    associated_method_id: MethodId,
}

pub struct JavaVMStateWrapperInner {
    most_up_to_date_ir_method_id_for_method_id: HashMap<MethodId, IRMethodID>,
    methods: HashMap<IRMethodID, JavaVMStateMethod>,
}

impl JavaVMStateWrapperInner {
    pub fn java_method_for_ir_method_id(&self, ir_method_id: IRMethodID) -> &JavaVMStateMethod {
        self.methods.get(&ir_method_id).unwrap()
    }

    pub fn associated_method_id(&self, ir_method_id: IRMethodID) -> MethodId {
        self.java_method_for_ir_method_id(ir_method_id).associated_method_id
    }

    pub fn restart_location(&self, ir_method_id: IRMethodID, restart_point: RestartPointID) -> IRInstructIndex {
        let restart_points = &self.methods.get(&ir_method_id).unwrap().restart_points;
        *restart_points.get(&restart_point).unwrap()
    }
}


pub enum JavaExitAction {}

pub enum VMExitEvent<'vm> {
    Allocate { size: usize, return_to: *mut c_void },
    TopLevelExitEvent {
        //todo when this stuff is registers can't have gc.
        _return: JavaValue<'vm>
    },
}

impl JavaVMStateWrapperInner {
    #[inline(never)]
    fn handle_vm_exit<'l, 'gc>(jvm: &'gc JVMState<'gc>, int_state: Option<&mut JavaExitFrame<'gc, 'l>>, vm_exit_type: &RuntimeVMExitInput) -> IRVMExitAction {
        // let exit_guard = jvm.perf_metrics.vm_exit_start();
        match vm_exit_type {
            RuntimeVMExitInput::AllocateObjectArray { type_, len, return_to_ptr, res_address, pc: _ } => {
                return exit_impls::allocate_object_array(jvm, int_state.unwrap(), *type_, *len, *return_to_ptr, *res_address);
            }
            RuntimeVMExitInput::LoadClassAndRecompile { .. } => todo!(),
            RuntimeVMExitInput::RunStaticNative { method_id, arg_start, num_args, res_ptr, return_to_ptr, pc: _ } => {
                todo!()
                // return exit_impls::run_static_native(jvm, todo!()/*int_state.unwrap()*/, *method_id, *arg_start, *num_args, *res_ptr, *return_to_ptr);
            }
            RuntimeVMExitInput::RunNativeVirtual { res_ptr, arg_start, method_id, return_to_ptr, pc: _ } => {
                todo!()
            }
            RuntimeVMExitInput::TopLevelReturn { return_value } => {
                return exit_impls::top_level_return(jvm, *return_value);
            }
            RuntimeVMExitInput::CompileFunctionAndRecompileCurrent {
                current_method_id,
                to_recompile,
                restart_point, pc: _
            } => {
                exit_impls::compile_function_and_recompile_current(jvm, int_state.unwrap().current_loader(jvm), *current_method_id, *to_recompile, *restart_point)
            }
            RuntimeVMExitInput::PutStatic { field_id, value_ptr, return_to_ptr, pc: _ } => {
                exit_impls::put_static(jvm, field_id, value_ptr, return_to_ptr)
            }
            RuntimeVMExitInput::InitClassAndRecompile { class_type, current_method_id, restart_point, pc: _ } => {
                exit_impls::init_class_and_recompile(jvm, int_state.unwrap(), *class_type, *current_method_id, *restart_point)
            }
            RuntimeVMExitInput::AllocatePrimitiveArray { .. } => todo!(),
            RuntimeVMExitInput::LogFramePointerOffsetValue { value, return_to_ptr, pc: _ } => {
                exit_impls::log_frame_pointer_offset_value(jvm, *value, *return_to_ptr)
            }
            RuntimeVMExitInput::LogWholeFrame { return_to_ptr, pc: _ } => {
                exit_impls::log_whole_frame(&jvm, todo!()/*int_state.unwrap()*/, *return_to_ptr)
            }
            RuntimeVMExitInput::TraceInstructionBefore { method_id, return_to_ptr, bytecode_offset, pc: _ } => {
                exit_impls::trace_instruction_before(&jvm, *method_id, *return_to_ptr, *bytecode_offset)
            }
            RuntimeVMExitInput::TraceInstructionAfter { method_id, return_to_ptr, bytecode_offset, pc: _ } => {
                exit_impls::trace_instruction_after(&jvm, int_state.unwrap(), *method_id, *return_to_ptr, *bytecode_offset)
            }
            RuntimeVMExitInput::NPE { .. } => {
                let int_state = int_state.unwrap();
                let npe = NullPointerException::new(jvm, int_state).expect("exception while creating exception?");
                return throw_impl(jvm, int_state, npe.new_java_value_handle().cast_throwable(), false);
            }
            RuntimeVMExitInput::CheckCastFailure { pc } => {
                let int_state = int_state.unwrap();
                int_state.debug_print_stack_trace(jvm);
                let npe = ClassCastException::new(jvm, int_state).expect("exception while creating exception?");
                return throw_impl(jvm, int_state, npe.new_java_value_handle().cast_throwable(), false);
            }
            RuntimeVMExitInput::AllocateObject { type_, return_to_ptr, res_address, pc: _ } => {
                exit_impls::allocate_object(jvm, int_state.unwrap().current_loader(jvm), type_, *return_to_ptr, res_address)
            }
            RuntimeVMExitInput::NewString { return_to_ptr, res, compressed_wtf8, pc: _ } => {
                exit_impls::new_string(&jvm, int_state.unwrap(), *return_to_ptr, *res, *compressed_wtf8)
            }
            RuntimeVMExitInput::NewClass { type_, res, return_to_ptr, pc: _ } => {
                exit_impls::new_class(jvm, int_state.unwrap(), *type_, *res, *return_to_ptr)
            }
            RuntimeVMExitInput::NewClassRegister { return_to_ptr, res, type_, pc } => {
                exit_impls::new_class_register(jvm, int_state.unwrap(), *type_, *res, *return_to_ptr)
            }
            RuntimeVMExitInput::InvokeVirtualResolve { return_to_ptr, object_ref_ptr, method_shape_id, method_number, native_method_restart_point, native_method_res, pc: _ } => {
                exit_impls::invoke_virtual_resolve(jvm, int_state.unwrap(), *return_to_ptr, *object_ref_ptr, *method_shape_id, MethodNumber(*method_number), *native_method_restart_point, *native_method_res)
            }
            RuntimeVMExitInput::MonitorEnter { obj_ptr, return_to_ptr, pc: _ } => {
                exit_impls::monitor_enter(jvm, int_state.unwrap(), *obj_ptr, *return_to_ptr)
            }
            RuntimeVMExitInput::MonitorExit { obj_ptr, return_to_ptr, pc: _ } => {
                exit_impls::monitor_exit(jvm, int_state.unwrap(), *obj_ptr, *return_to_ptr)
            }
            RuntimeVMExitInput::MonitorEnterRegister { obj_ptr, return_to_ptr, pc } => {
                exit_impls::monitor_enter(jvm, int_state.unwrap(), *obj_ptr, *return_to_ptr)
            }
            RuntimeVMExitInput::MonitorExitRegister { obj_ptr, return_to_ptr, pc } => {
                exit_impls::monitor_exit(jvm, int_state.unwrap(), *obj_ptr, *return_to_ptr)
            }
            RuntimeVMExitInput::GetStatic { res_value_ptr: value_ptr, field_name, cpdtype_id, return_to_ptr, pc: _ } => {
                exit_impls::get_static(jvm, todo!()/*int_state.unwrap()*/, *value_ptr, *field_name, *cpdtype_id, *return_to_ptr)
            }
            RuntimeVMExitInput::InstanceOf { res, value, cpdtype_id, return_to_ptr, pc: _ } => {
                exit_impls::instance_of(jvm, int_state.unwrap(), res, value, cpdtype_id, return_to_ptr)
            }
            RuntimeVMExitInput::CheckCast { value, cpdtype_id, return_to_ptr, pc: _ } => {
                exit_impls::check_cast(&jvm, int_state.unwrap(), value, cpdtype_id, return_to_ptr)
            }
            RuntimeVMExitInput::RunNativeSpecial { res_ptr, arg_start, method_id, return_to_ptr, pc: _ } => {
                todo!()
                // exit_impls::run_native_special(jvm, todo!()/*int_state.unwrap()*/, *res_ptr, *arg_start, *method_id, *return_to_ptr)
            }
            RuntimeVMExitInput::InvokeInterfaceResolve { return_to_ptr, native_method_restart_point, native_method_res, object_ref, method_shape_id, method_number, interface_id, pc: _ } => {
                exit_impls::invoke_interface_resolve(jvm, int_state.unwrap(), *return_to_ptr, *native_method_restart_point, *native_method_res, *object_ref, *method_shape_id, *interface_id, *method_number)
            }
            RuntimeVMExitInput::Throw { exception_obj_ptr, pc: _ } => {
                exit_impls::throw_exit(&jvm, int_state.unwrap(), *exception_obj_ptr)
            }
            RuntimeVMExitInput::MultiAllocateArray {
                elem_type,
                num_arrays,
                len_start,
                return_to_ptr,
                res_address,
                pc: _
            } => {
                multi_allocate_array(jvm, int_state.unwrap(), *elem_type, *num_arrays, *len_start, *return_to_ptr, *res_address)
            }
            RuntimeVMExitInput::RunNativeSpecialNew { method_id, return_to_ptr } => {
                run_native_special_new(jvm, int_state, *method_id, *return_to_ptr)
            }
            RuntimeVMExitInput::RunNativeStaticNew { method_id, return_to_ptr } => {
                run_native_static_new(jvm, int_state, *method_id, *return_to_ptr)
            }
            RuntimeVMExitInput::RunInterpreted { method_id, return_to_ptr } => {
                let int_state = int_state.unwrap();
                let expected_method_id = int_state.frame_ref().method_id();
                assert_eq!(expected_method_id, Ok(*method_id));
                return match int_state.to_interpreter_frame(|java_interpreter_frame| {
                    let res = run_function_interpreted(jvm, java_interpreter_frame)?;
                    let mut saved_registers_without_ipdiff = SavedRegistersWithoutIPDiff::no_change();
                    saved_registers_without_ipdiff.rax = res.map(|res| res.to_interpreter_jv().to_raw());
                    let diff = SavedRegistersWithIPDiff { rip: Some(*return_to_ptr), saved_registers_without_ip: saved_registers_without_ipdiff };
                    Ok(IRVMExitAction::RestartWithRegisterState { diff })
                }) {
                    Ok(ir_vm_exit_action) => {
                        ir_vm_exit_action
                    }
                    Err(WasException { exception_obj }) => {
                        throw_impl(jvm, int_state, exception_obj, true)
                    }
                };
            }
            RuntimeVMExitInput::AssertInstanceOf { res, value, cpdtype_id, return_to_ptr, pc, expected } => {
                exit_impls::assert_instance_of(jvm, todo!()/*int_state.unwrap()*/, res, value, cpdtype_id, return_to_ptr, *expected)
            }
            RuntimeVMExitInput::ArrayOutOfBounds { pc, index } => {
                let index = unsafe { index.cast::<i32>().as_ptr().read() };
                exit_impls::array_out_of_bounds(&jvm, int_state.unwrap(), index)
            }
            RuntimeVMExitInput::Todo { pc, todo_case } => {
                dbg!(todo_case);
                int_state.unwrap().debug_print_stack_trace(jvm);
                todo!()
            }
            RuntimeVMExitInput::AllocateObjectArrayIntrinsic { type_, len, return_to_ptr, res_address } => {
                return exit_impls::allocate_object_array(jvm, int_state.unwrap(), *type_, *len, *return_to_ptr, *res_address);
            }
        }
    }
}

pub mod exit_impls;
pub mod dump_frame;
pub mod java_vm_state;
