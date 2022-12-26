use std::ffi::c_void;

use itertools::Itertools;
use nonnull_const::NonNullConst;

use another_jit_vm::Register;
use another_jit_vm::saved_registers_utils::{SavedRegistersWithIPDiff, SavedRegistersWithoutIPDiff};
use another_jit_vm_ir::IRVMExitAction;
use classfile_view::view::HasAccessFlags;
use gc_memory_layout_common::frame_layout::NativeStackframeMemoryLayout;
use rust_jvm_common::compressed_classfile::class_names::CClassName;

use rust_jvm_common::MethodId;
use rust_jvm_common::runtime_type::{RuntimeRefType, RuntimeType};

use crate::{JavaValueCommon, JVMState, WasException};
use crate::better_java_stack::exit_frame::JavaExitFrame;
use crate::interpreter::common::invoke::native::run_native_method;
use crate::ir_to_java_layer::exit_impls::throw_impl;
use crate::java_values::native_to_new_java_value_rtype;

#[inline(never)]
pub fn run_native_special_new<'vm, 'k>(jvm: &'vm JVMState<'vm>, int_state: Option<&mut JavaExitFrame<'vm, 'k>>, method_id: MethodId, return_to_ptr: *const c_void, resolved_fn_ptr: NonNullConst<c_void>) -> IRVMExitAction {
    //todo need to use this frame instead of pushing a new one
    let int_state = int_state.unwrap();
    let (rc, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
    let view = rc.view();
    let method_view = view.method_view_i(method_i);
    let mut args = vec![];
    //todo dup
    let memory_layout = NativeStackframeMemoryLayout { num_locals: jvm.num_local_vars_native(method_id) };
    let nth_local = int_state.read_target(memory_layout.local_var_entry(0));
    let rtype: RuntimeType = RuntimeType::Ref(RuntimeRefType::Class(CClassName::object()));
    args.push(native_to_new_java_value_rtype(nth_local, rtype, jvm));
    let mut i = 0;
    for arg_type in method_view.desc().arg_types.iter() {
        let nth_local = int_state.read_target(memory_layout.local_var_entry((i + 1) as u16));
        let rtype: RuntimeType = arg_type.to_runtime_type().unwrap();
        if arg_type.is_double_or_long() {
            i += 1;
        }
        args.push(native_to_new_java_value_rtype(nth_local, rtype, jvm));
        i += 1;
    }
    let res = match run_native_method(jvm, int_state, rc, method_i, args.iter().map(|handle| handle.as_njv()).collect_vec(), Some(resolved_fn_ptr)) {
        Ok(x) => x,
        Err(WasException { exception_obj }) => {
            return throw_impl(jvm, int_state, exception_obj, true);
        }
    };
    let mut diff = SavedRegistersWithoutIPDiff::no_change();
    diff.add_change(Register(0), res.map(|handle| unsafe { handle.to_stack_native().as_u64 }).unwrap_or(0));
    //todo what if gc after this function returns such that the handle gets collected
    IRVMExitAction::RestartWithRegisterState {
        diff: SavedRegistersWithIPDiff {
            rip: Some(return_to_ptr),
            saved_registers_without_ip: diff,
        }
    }
}

#[inline(never)]
pub fn run_native_static_new<'vm, 'k>(jvm: &'vm JVMState<'vm>, int_state: Option<&mut JavaExitFrame<'vm, 'k>>, method_id: MethodId, return_to_ptr: *const c_void, resolved_fn_ptr: NonNullConst<c_void>) -> IRVMExitAction {
    let int_state = int_state.unwrap();
    let (rc, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
    let view = rc.view();
    let method_view = view.method_view_i(method_i);
    assert!(method_view.is_static());
    let mut args = vec![];
    let memory_layout = NativeStackframeMemoryLayout { num_locals: jvm.num_local_vars_native(method_id) };
    let mut i = 0;
    for arg_type in method_view.desc().arg_types.iter() {
        let nth_local = int_state.read_target(memory_layout.local_var_entry(i as u16));
        let rtype: RuntimeType = arg_type.to_runtime_type().unwrap();
        args.push(native_to_new_java_value_rtype(nth_local, rtype, jvm));
        if arg_type.is_double_or_long() {
            i += 1
        }
        i += 1;
    }
    let res = match run_native_method(jvm, int_state, rc, method_i, args.iter().map(|handle| handle.as_njv()).collect_vec(), Some(resolved_fn_ptr)) {
        Ok(x) => x,
        Err(WasException { exception_obj }) => {
            return throw_impl(jvm, int_state, exception_obj, true);
        }
    };
    let mut diff = SavedRegistersWithoutIPDiff::no_change();
    diff.add_change(Register(0), res.map(|handle| unsafe { handle.to_stack_native().as_u64 }).unwrap_or(0));
    IRVMExitAction::RestartWithRegisterState {
        diff: SavedRegistersWithIPDiff {
            rip: Some(return_to_ptr),
            saved_registers_without_ip: diff,
        }
    }
}
