use std::ffi::c_void;
use std::ptr::null_mut;
use itertools::Itertools;
use another_jit_vm::Register;
use another_jit_vm::saved_registers_utils::{SavedRegistersWithIPDiff, SavedRegistersWithoutIPDiff};
use another_jit_vm_ir::{IRVMExitAction, WasException};
use classfile_view::view::HasAccessFlags;
use gc_memory_layout_common::layout::NativeStackframeMemoryLayout;
use rust_jvm_common::compressed_classfile::names::CClassName;
use rust_jvm_common::MethodId;
use rust_jvm_common::runtime_type::{RuntimeRefType, RuntimeType};
use crate::{InterpreterStateGuard, JavaValueCommon, JVMState};
use crate::instructions::invoke::native::run_native_method;
use crate::ir_to_java_layer::exit_impls::throw_impl;
use crate::java_values::native_to_new_java_value_rtype;

#[inline(never)]
pub fn run_native_special_new<'vm>(jvm: &'vm JVMState<'vm>, int_state: Option<&mut InterpreterStateGuard<'vm, '_>>, method_id: MethodId, return_to_ptr: *const c_void) -> IRVMExitAction {
    let int_state = int_state.unwrap();
    let (rc, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
    let view = rc.view();
    let method_view = view.method_view_i(method_i);
    let mut args = vec![];
    let current_frame = int_state.current_frame();
    //todo dup
    let memory_layout = NativeStackframeMemoryLayout { num_locals: jvm.num_local_vars_native(method_id) };
    let nth_local = current_frame.frame_view.read_target(memory_layout.local_var_entry(0));
    let rtype: RuntimeType = RuntimeType::Ref(RuntimeRefType::Class(CClassName::object()));
    args.push(native_to_new_java_value_rtype(nth_local, rtype, jvm));
    let mut i = 0;
    for arg_type in method_view.desc().arg_types.iter() {
        let nth_local = current_frame.frame_view.read_target(memory_layout.local_var_entry((i + 1) as u16));
        let rtype: RuntimeType = arg_type.to_runtime_type().unwrap();
        if arg_type.is_double_or_long() {
            i += 1;
        }
        args.push(native_to_new_java_value_rtype(nth_local, rtype, jvm));
        i += 1;
    }
    let res = match run_native_method(jvm, int_state, rc, method_i, args.iter().map(|handle| handle.as_njv()).collect_vec()) {
        Ok(x) => x,
        Err(WasException{}) => {
            let throw_obj = int_state.throw().as_ref().unwrap().duplicate_discouraged().new_java_handle();
            int_state.set_throw(None);//todo should move this into throw impl
            return throw_impl(jvm, int_state, throw_obj);
        },
    };
    let mut diff = SavedRegistersWithoutIPDiff::no_change();
    diff.add_change(Register(0), res.map(|handle| unsafe { handle.to_native().object }).unwrap_or(null_mut()));
    IRVMExitAction::RestartWithRegisterState {
        diff: SavedRegistersWithIPDiff {
            rip: Some(return_to_ptr),
            saved_registers_without_ip: diff,
        }
    }
}

#[inline(never)]
pub fn run_native_static_new<'vm>(jvm: &'vm JVMState<'vm>, int_state: Option<&mut InterpreterStateGuard<'vm, '_>>, method_id: MethodId, return_to_ptr: *const c_void) -> IRVMExitAction {
    let int_state = int_state.unwrap();
    let (rc, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
    let view = rc.view();
    let method_view = view.method_view_i(method_i);
    assert!(method_view.is_static());
    let mut args = vec![];
    let memory_layout = NativeStackframeMemoryLayout { num_locals: jvm.num_local_vars_native(method_id) };
    let current_frame = int_state.current_frame();
    let mut i = 0;
    for arg_type in method_view.desc().arg_types.iter() {
        let nth_local = current_frame.frame_view.read_target(memory_layout.local_var_entry(i as u16));
        let rtype: RuntimeType = arg_type.to_runtime_type().unwrap();
        args.push(native_to_new_java_value_rtype(nth_local, rtype, jvm));
        if arg_type.is_double_or_long() {
            i += 1
        }
        i += 1;
    }
    let res = match run_native_method(jvm, int_state, rc, method_i, args.iter().map(|handle| handle.as_njv()).collect_vec()) {
        Ok(x) => x,
        Err(WasException{}) => {
            let expception_obj_handle = int_state.throw().unwrap().duplicate_discouraged();
            int_state.set_throw(None);
            return throw_impl(jvm, int_state, expception_obj_handle.new_java_handle());
        },
    };
    let mut diff = SavedRegistersWithoutIPDiff::no_change();
    diff.add_change(Register(0), res.map(|handle| unsafe { handle.to_native().object }).unwrap_or(null_mut()));
    IRVMExitAction::RestartWithRegisterState {
        diff: SavedRegistersWithIPDiff {
            rip: Some(return_to_ptr),
            saved_registers_without_ip: diff,
        }
    }
}
