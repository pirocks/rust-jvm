use std::ffi::c_void;
use std::mem::{size_of, transmute};
use std::num::NonZeroU8;
use std::ops::Deref;
use std::ptr::NonNull;
use itertools::Itertools;
use libc::memset;
use another_jit_vm::saved_registers_utils::{SavedRegistersWithIPDiff, SavedRegistersWithoutIPDiff};
use another_jit_vm_ir::compiler::RestartPointID;
use another_jit_vm_ir::IRVMExitAction;
use another_jit_vm_ir::vm_exit_abi::register_structs::InvokeVirtualResolve;
use gc_memory_layout_common::memory_regions::AllocatedObjectType;
use jvmti_jni_bindings::{jint, jlong};
use perf_metrics::VMExitGuard;
use rust_jvm_common::compressed_classfile::{CompressedParsedDescriptorType, CompressedParsedRefType, CPDType};
use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName};
use rust_jvm_common::cpdtype_table::CPDTypeID;
use rust_jvm_common::method_shape::{MethodShape, MethodShapeID};
use rust_jvm_common::{ByteCodeOffset, FieldId, MethodId, NativeJavaValue};
use rust_jvm_common::compressed_classfile::code::CompressedExceptionTableElem;
use sketch_jvm_version_of_utf8::wtf8_pool::CompressedWtf8String;
use crate::{check_initing_or_inited_class, InterpreterStateGuard, JavaValueCommon, JString, JVMState, MethodResolver, NewAsObjectOrJavaValue, NewJavaValue, NewJavaValueHandle, WasException};
use crate::class_loading::assert_inited_or_initing_class;
use crate::instructions::fields::get_static_impl;
use crate::instructions::invoke::native::run_native_method;
use crate::instructions::invoke::virtual_::virtual_method_lookup;
use crate::instructions::special::{instance_of_exit_impl, instance_of_exit_impl_impl};
use crate::ir_to_java_layer::dump_frame::dump_frame_contents;
use crate::ir_to_java_layer::java_stack::OpaqueFrameIdOrMethodID;
use crate::java::lang::class::JClass;
use crate::java_values::{default_value, native_to_new_java_value};
use crate::jit::{NotCompiledYet, ResolvedInvokeVirtual};
use crate::jit::state::runtime_class_to_allocated_object_type;
use crate::runtime_class::static_vars;
use crate::utils::lookup_method_parsed;

pub fn multi_allocate_array<'gc>(jvm: &'gc JVMState<'gc>, int_state: &mut InterpreterStateGuard<'gc, '_>, elem_type: CPDTypeID, num_arrays: u8, len_start: *const i64, return_to_ptr: *const c_void, res_address: *mut NonNull<c_void>) -> IRVMExitAction {
    let elem_type = *jvm.cpdtype_table.read().unwrap().get_cpdtype(elem_type);
    let elem_type = elem_type.unwrap_non_array();
    let array_type = CPDType::Array { base_type: elem_type, num_nested_arrs: NonZeroU8::new(num_arrays).unwrap() };
    let mut lens = vec![];
    unsafe {
        for len_index in 0..num_arrays {
            let offsetted_ptr = len_start.sub(len_index as usize);
            lens.push(offsetted_ptr.cast::<i32>().read());
        }
    }
    assert_inited_or_initing_class(jvm, elem_type.to_cpdtype());
    let mut current_value = default_value(&elem_type.to_cpdtype()).as_njv().to_native();
    //iterates from innermost to outermost
    for (depth, len) in lens.into_iter().rev().enumerate() {
        let rc = match NonZeroU8::new(depth as u8 + 1) {
            None => {
                panic!()
            }
            Some(depth) => {
                assert_inited_or_initing_class(jvm, CPDType::Array { base_type: elem_type, num_nested_arrs: depth })
            }
        };
        let array = runtime_class_to_allocated_object_type(rc.as_ref(), int_state.current_loader(jvm), Some(len as usize));
        let mut memory_region_guard = jvm.gc.memory_region.lock().unwrap();
        let array_size = array.size();
        let region_data = memory_region_guard.find_or_new_region_for(array);
        let allocated_object = region_data.get_allocation();
        unsafe { allocated_object.as_ptr().cast::<jlong>().write(len as jlong); }
        for i in 0..len {
            unsafe { allocated_object.as_ptr().cast::<NativeJavaValue<'gc>>().offset((i + 1) as isize).write(current_value) };
        }
        current_value = NativeJavaValue { object: allocated_object.as_ptr() };
    }

    unsafe { res_address.cast::<NativeJavaValue<'gc>>().write(current_value) }
    IRVMExitAction::RestartAtPtr { ptr: return_to_ptr }
}

pub fn throw_exit<'gc>(jvm: &'gc JVMState<'gc>, int_state: &mut InterpreterStateGuard<'gc, '_>, exception_obj_ptr: *const c_void) -> IRVMExitAction{
    let throw = jvm.perf_metrics.vm_exit_throw();
    if jvm.exit_trace_options.tracing_enabled() {
        eprintln!("Throw");
    }
    eprintln!("THROW AT:");
    int_state.debug_print_stack_trace(jvm);
    let exception_obj_native_value = unsafe { (exception_obj_ptr).cast::<NativeJavaValue<'gc>>().read() };
    let exception_obj_handle = native_to_new_java_value(exception_obj_native_value,&CClassName::object().into(), jvm);
    throw_impl(&jvm, int_state, exception_obj_handle)
}

pub fn invoke_interface_resolve<'gc>(jvm: &'gc JVMState<'gc>, int_state: &mut InterpreterStateGuard<'gc, '_>, exit_guard: VMExitGuard, return_to_ptr: *const c_void, native_method_restart_point: RestartPointID, native_method_res: *mut c_void, object_ref: *const c_void, target_method_id: MethodId) -> IRVMExitAction {
    if jvm.exit_trace_options.tracing_enabled() {
        eprintln!("InvokeInterfaceResolve");
    }
    let caller_method_id = int_state.current_frame().frame_view.ir_ref.method_id().unwrap();
    let obj_native_jv = unsafe { (object_ref).cast::<NativeJavaValue>().read() };
    let obj_jv_handle = native_to_new_java_value(obj_native_jv,&CPDType::object(), jvm);
    let obj_rc = obj_jv_handle.unwrap_object_nonnull().runtime_class(jvm);
    let (target_rc, target_method_i) = jvm.method_table.read().unwrap().try_lookup(target_method_id).unwrap();
    let class_view = target_rc.view();
    let method_view = class_view.method_view_i(target_method_i);
    let method_name = method_view.name();
    let method_desc = method_view.desc();
    let (resolved_method_i, resolved_rc) = lookup_method_parsed(jvm, obj_rc, method_name, method_desc).unwrap();
    let resolved_method_id = jvm.method_table.write().unwrap().get_method_id(resolved_rc.clone(), resolved_method_i);
    if jvm.is_native_by_method_id(resolved_method_id) {
        let args_jv_handle = virtual_args_extract(jvm, method_desc.arg_types.as_slice(), object_ref);
        match run_native_method(jvm, int_state, resolved_rc, resolved_method_i, args_jv_handle.iter().map(|handle| handle.as_njv()).collect_vec()) {
            Ok(res) => {
                if let Some(res) = res {
                    unsafe { ((native_method_res) as *mut NativeJavaValue).write(res.as_njv().to_native()) }
                };
                if !jvm.instruction_trace_options.partial_tracing() {
                    // jvm.java_vm_state.assertion_state.lock().unwrap().current_before.pop().unwrap();
                }
                let restart_address = jvm.java_vm_state.lookup_restart_point(caller_method_id, native_method_restart_point);
                return IRVMExitAction::RestartAtPtr { ptr: restart_address };
            }
            Err(WasException {}) => {
                todo!()
            }
        }
    } else {
        let resolver = MethodResolver { jvm, loader: int_state.current_loader(jvm) };
        jvm.java_vm_state.add_method_if_needed(jvm, &resolver, resolved_method_id);
        let new_frame_size = resolver.lookup_method_layout(resolved_method_id).full_frame_size();
        let ir_method_id = jvm.java_vm_state.lookup_method_ir_method_id(resolved_method_id);
        let address = jvm.java_vm_state.ir.lookup_ir_method_id_pointer(ir_method_id);
        let mut start_diff = SavedRegistersWithoutIPDiff::no_change();
        start_diff.add_change(InvokeVirtualResolve::ADDRESS_RES, address as *mut c_void);
        start_diff.add_change(InvokeVirtualResolve::IR_METHOD_ID_RES, ir_method_id.0 as *mut c_void);
        start_diff.add_change(InvokeVirtualResolve::METHOD_ID_RES, resolved_method_id as *mut c_void);
        start_diff.add_change(InvokeVirtualResolve::NEW_FRAME_SIZE_RES, new_frame_size as *mut c_void);
        drop(exit_guard);
        IRVMExitAction::RestartWithRegisterState {
            diff: SavedRegistersWithIPDiff {
                rip: Some(return_to_ptr),
                saved_registers_without_ip: start_diff,
            }
        }
    }
}

pub fn run_native_special<'gc>(jvm: &'gc JVMState<'gc>, int_state: &mut InterpreterStateGuard<'gc, '_>, exit_guard: &VMExitGuard, res_ptr: *mut c_void, arg_start: *const c_void, method_id: MethodId, return_to_ptr: *const c_void) -> IRVMExitAction {
    if jvm.exit_trace_options.tracing_enabled() {
        eprintln!("RunNativeSpecial");
    }
    let (rc, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
    let class_view = rc.view();
    let method_view = class_view.method_view_i(method_i);
    let arg_types = &method_view.desc().arg_types;
    let arg_start: *const c_void = arg_start;
    let args_jv_handle = virtual_args_extract(jvm, arg_types, arg_start);
    let args_new_jv: Vec<NewJavaValue> = args_jv_handle.iter().map(|handle| handle.as_njv()).collect();
    args_new_jv[0].unwrap_object_alloc().unwrap();//nonnull this
    drop(exit_guard);
    let res = match run_native_method(jvm, int_state, rc, method_i, args_new_jv) {
        Ok(x) => x,
        Err(WasException {}) => {
            let expception_obj_handle = int_state.throw().unwrap().duplicate_discouraged();
            int_state.set_throw(None);
            return throw_impl(jvm, int_state, expception_obj_handle.new_java_handle());
        }
    };
    if let Some(res) = res {
        unsafe { ((res_ptr) as *mut NativeJavaValue).write(res.as_njv().to_native()) }
    };
    if !jvm.instruction_trace_options.partial_tracing() {
        // jvm.java_vm_state.assertion_state.lock().unwrap().current_before.pop().unwrap();
    }
    IRVMExitAction::RestartAtPtr { ptr: return_to_ptr }
}

pub fn check_cast<'gc>(jvm: &'gc JVMState<'gc>, int_state: &mut InterpreterStateGuard<'gc, '_>, exit_guard: &VMExitGuard, value: &*const c_void, cpdtype_id: &CPDTypeID, return_to_ptr: &*const c_void) -> IRVMExitAction {
    let checkcast = jvm.perf_metrics.vm_exit_checkcast();
    if jvm.exit_trace_options.tracing_enabled() {
        eprintln!("CheckCast");
    }
    let cpdtype = jvm.cpdtype_table.read().unwrap().get_cpdtype(*cpdtype_id).clone();
    //todo just use region data from pointer to cache the result of this checkast and then havee a restart point
    /*runtime_class_to_allocated_object_type(&rc, LoaderName::BootstrapLoader, todo!());
    todo!();*/
    let value = unsafe { (*value).cast::<NativeJavaValue>().read() };
    let value = native_to_new_java_value(value,&CClassName::object().into(), jvm);
    let value = value.unwrap_object();
    if let Some(handle) = value {
        let res_int = instance_of_exit_impl(jvm, cpdtype, Some(handle.unwrap_normal_object_ref()));
        if res_int == 0 {
            dbg!(cpdtype.jvm_representation(&jvm.string_pool));
            dbg!(handle.runtime_class(jvm).cpdtype().jvm_representation(&jvm.string_pool));
            int_state.debug_print_stack_trace(jvm);
            todo!()
        }

        let base_address_and_mask = jvm.gc.memory_region.lock().unwrap().find_object_base_address_and_mask(handle.ptr());
        jvm.known_addresses.sink_known_address(cpdtype, base_address_and_mask)
    }
    drop(exit_guard);
    drop(checkcast);
    IRVMExitAction::RestartAtPtr { ptr: *return_to_ptr }
}

pub fn instance_of<'gc>(jvm: &'gc JVMState<'gc>, int_state: &mut InterpreterStateGuard<'gc, '_>, exit_guard: &VMExitGuard, res: &*mut c_void, value: &*const c_void, cpdtype_id: &CPDTypeID, return_to_ptr: &*const c_void) -> IRVMExitAction {
    if jvm.exit_trace_options.tracing_enabled() {
        eprintln!("InstanceOf");
    }
    let cpdtype = *jvm.cpdtype_table.read().unwrap().get_cpdtype(*cpdtype_id);
    let value = unsafe { (*value).cast::<NativeJavaValue>().read() };
    let value = native_to_new_java_value(value,&CClassName::object().into(), jvm);
    let value = value.unwrap_object();
    check_initing_or_inited_class(jvm, int_state, cpdtype).unwrap();
    let res_int = instance_of_exit_impl(jvm, cpdtype, value.as_ref().map(|handle| handle.unwrap_normal_object_ref()));
    unsafe { (*((*res) as *mut NativeJavaValue)).int = res_int };
    drop(exit_guard);
    IRVMExitAction::RestartAtPtr { ptr: *return_to_ptr }
}

pub fn get_static<'gc>(jvm: &'gc JVMState<'gc>, int_state: &mut InterpreterStateGuard<'gc, '_>, exit_guard: &VMExitGuard, value_ptr: *mut c_void, field_name: FieldName, cpdtype_id: CPDTypeID, return_to_ptr: *const c_void) -> IRVMExitAction {
    let get_static = jvm.perf_metrics.vm_exit_get_static();
    if jvm.exit_trace_options.tracing_enabled() {
        eprintln!("GetStatic");
    }
    let cpd_type = jvm.cpdtype_table.read().unwrap().get_cpdtype(cpdtype_id).clone();
    let name = cpd_type.unwrap_class_type();
    let static_var = get_static_impl(jvm, int_state, name, field_name).unwrap().unwrap();
    // let static_var = static_vars_guard.get(field_name);
    // todo doesn't handle interfaces and the like
    // int_state.debug_print_stack_trace(jvm);
    unsafe { (value_ptr).cast::<NativeJavaValue>().write(static_var.as_njv().to_native()); }
    drop(exit_guard);
    drop(get_static);
    IRVMExitAction::RestartAtPtr { ptr: return_to_ptr }
}

pub fn monitor_exit<'gc>(jvm: &'gc JVMState<'gc>, int_state: &mut InterpreterStateGuard<'gc, '_>, exit_guard: &VMExitGuard, obj_ptr: &*const c_void, return_to_ptr: &*const c_void) -> IRVMExitAction {
    if jvm.exit_trace_options.tracing_enabled() {
        eprintln!("MonitorExit");
    }
    let monitor = jvm.monitor_for(*obj_ptr);
    monitor.unlock(jvm, int_state).unwrap();
    drop(exit_guard);
    IRVMExitAction::RestartAtPtr { ptr: *return_to_ptr }
}

pub fn monitor_enter<'gc>(jvm: &'gc JVMState<'gc>, int_state: &mut InterpreterStateGuard<'gc, '_>, exit_guard: &VMExitGuard, obj_ptr: &*const c_void, return_to_ptr: &*const c_void) -> IRVMExitAction {
    if jvm.exit_trace_options.tracing_enabled() {
        eprintln!("MonitorEnter");
    }
    let monitor = jvm.monitor_for(*obj_ptr);
    monitor.lock(jvm, int_state).unwrap();
    drop(exit_guard);
    IRVMExitAction::RestartAtPtr { ptr: *return_to_ptr }
}

pub fn invoke_virtual_resolve<'gc>(
    jvm: &'gc JVMState<'gc>,
    int_state: &mut InterpreterStateGuard<'gc, '_>,
    exit_guard: &VMExitGuard,
    return_to_ptr: *const c_void,
    object_ref_ptr: *const c_void,
    method_shape_id: MethodShapeID,
    native_method_restart_point: RestartPointID,
    native_method_res: *mut c_void
) -> IRVMExitAction {
    if jvm.exit_trace_options.tracing_enabled() {
        eprintln!("InvokeVirtualResolve");
    }
    let caller_method_id = int_state.current_frame().frame_view.ir_ref.method_id().unwrap();
    let MethodShape { name, desc } = jvm.method_shapes.lookup_method_shape(method_shape_id);
    //todo this is probably wrong what if there's a class with a same name private method?
    // like surely I need to start at the classname specified in the bytecode
    let mut memory_region_guard = jvm.gc.memory_region.lock().unwrap();
    let maybe_non_null = NonNull::new(unsafe { (object_ref_ptr as *const *mut c_void).read() });
    let allocated_type = memory_region_guard.find_object_allocated_type(maybe_non_null.unwrap()).clone();
    let allocated_type_id = memory_region_guard.lookup_or_add_type(&allocated_type);
    drop(memory_region_guard);
    let rc = match allocated_type {
        AllocatedObjectType::Class { name, .. } => {
            assert_inited_or_initing_class(jvm, (name).into())
        }
        AllocatedObjectType::PrimitiveArray { .. } |
        AllocatedObjectType::ObjectArray { .. } => {
            assert_inited_or_initing_class(jvm, CClassName::object().into())
        }
    };
    let (resolved_rc, method_i) = virtual_method_lookup(jvm, int_state, name, &desc, rc).unwrap();
    let method_id = jvm.method_table.write().unwrap().get_method_id(resolved_rc, method_i);
    let method_resolver = MethodResolver { jvm, loader: int_state.current_loader(jvm) };
    if jvm.java_vm_state.try_lookup_ir_method_id(OpaqueFrameIdOrMethodID::Method { method_id: method_id as u64 }).is_none() {
        if !jvm.is_native_by_method_id(method_id) {
            jvm.java_vm_state.add_method_if_needed(jvm, &method_resolver, method_id);
        } else {
            //is native should run native method
            //todo duplicated
            if jvm.exit_trace_options.tracing_enabled() {
                eprintln!("RunNativeVirtual");
            }
            let (rc, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
            let class_view = rc.view();
            let method_view = class_view.method_view_i(method_i);
            let arg_types = &method_view.desc().arg_types;
            let arg_start: *const c_void = object_ref_ptr;
            let args_jv_handle = virtual_args_extract(jvm, arg_types, arg_start);
            let args_new_jv = args_jv_handle.iter().map(|handle| handle.as_njv()).collect();
            drop(exit_guard);
            let res = match run_native_method(jvm, int_state, rc, method_i, args_new_jv) {
                Ok(x) => x,
                Err(WasException {}) => {
                    let allocate_obj = int_state.throw().unwrap().duplicate_discouraged();
                    int_state.set_throw(None);
                    dbg!(allocate_obj.cast_throwable().to_string(jvm, int_state).unwrap().unwrap().to_rust_string(jvm));
                    todo!()
                }
            };
            if let Some(res) = res {
                unsafe { ((native_method_res) as *mut NativeJavaValue).write(res.as_njv().to_native()) }
            };
            if !jvm.instruction_trace_options.partial_tracing() {
                /*jvm.java_vm_state.assertion_state.lock().unwrap().current_before.pop().unwrap();*/
            }
            let restart_address = jvm.java_vm_state.lookup_restart_point(caller_method_id, native_method_restart_point);
            return IRVMExitAction::RestartAtPtr { ptr: restart_address };
        }
    }

    let ResolvedInvokeVirtual {
        address,
        ir_method_id,
        method_id,
        new_frame_size
    } = match jvm.java_vm_state.lookup_resolved_invoke_virtual(method_id, &method_resolver) {
        Ok(resolved) => {
            resolved
        }
        Err(NotCompiledYet { needs_compiling }) => {
            // let rc = assert_loaded_class(jvm, allocated_type.as_cpdtype());
            jvm.java_vm_state.add_method_if_needed(jvm, &method_resolver, needs_compiling);
            // jvm.java_vm_state.add_method(jvm, &method_resolver, *debug_method_id);
            // jvm.java_vm_state.add_method(jvm, &method_resolver, method_id);
            dbg!(needs_compiling);
            todo!()
            // jvm.vtables.read().unwrap().lookup_resolved(allocated_type_id, *inheritance_id).unwrap()
        }
    };
    let mut start_diff = SavedRegistersWithoutIPDiff::no_change();
    start_diff.add_change(InvokeVirtualResolve::ADDRESS_RES, address as *mut c_void);
    start_diff.add_change(InvokeVirtualResolve::IR_METHOD_ID_RES, ir_method_id.0 as *mut c_void);
    start_diff.add_change(InvokeVirtualResolve::METHOD_ID_RES, method_id as *mut c_void);
    start_diff.add_change(InvokeVirtualResolve::NEW_FRAME_SIZE_RES, new_frame_size as *mut c_void);
    drop(exit_guard);
    IRVMExitAction::RestartWithRegisterState {
        diff: SavedRegistersWithIPDiff {
            rip: Some(return_to_ptr),
            saved_registers_without_ip: start_diff,
        }
    }
}

pub fn new_class<'gc>(jvm: &'gc JVMState<'gc>, int_state: &mut InterpreterStateGuard<'gc, '_>, exit_guard: &VMExitGuard, type_: CPDTypeID, res: *mut c_void, return_to_ptr: *const c_void) -> IRVMExitAction {
    if jvm.exit_trace_options.tracing_enabled() {
        eprintln!("NewClass");
    }
    let cpdtype = jvm.cpdtype_table.write().unwrap().get_cpdtype(type_).clone();
    let jclass = JClass::from_type(jvm, int_state, cpdtype).unwrap();
    let jv_new_handle = jclass.new_java_value_handle();
    unsafe {
        let raw_64 = jv_new_handle.as_njv().to_native().as_u64;
        (res as *mut u64).write(raw_64);
    };
    drop(exit_guard);
    IRVMExitAction::RestartAtPtr { ptr: return_to_ptr }
}

pub fn new_string<'gc>(jvm: &'gc JVMState<'gc>, int_state: &mut InterpreterStateGuard<'gc, '_>, exit_guard: &VMExitGuard, return_to_ptr: *const c_void, res: *mut c_void, compressed_wtf8: &CompressedWtf8String) -> IRVMExitAction {
    if jvm.exit_trace_options.tracing_enabled() {
        eprintln!("NewString");
    }
    let wtf8buf = compressed_wtf8.to_wtf8(&jvm.wtf8_pool);
    drop(exit_guard);
    let jstring = JString::from_rust(jvm, int_state, wtf8buf).expect("todo exceptions").intern(jvm, int_state).unwrap();
    let jv = jstring.new_java_value_handle();
    unsafe {
        let raw_64 = jv.to_native().as_u64;
        (res as *mut u64).write(raw_64);
    }
    IRVMExitAction::RestartAtPtr { ptr: return_to_ptr }
}

pub fn allocate_object<'gc>(jvm: &'gc JVMState<'gc>, int_state: &mut InterpreterStateGuard<'gc, '_>, exit_guard: &VMExitGuard, type_: &CPDTypeID, return_to_ptr: *const c_void, res_address: &*mut NonNull<c_void>) -> IRVMExitAction {
    let guard = jvm.perf_metrics.vm_exit_allocate_obj();
    if jvm.exit_trace_options.tracing_enabled() {
        eprintln!("AllocateObject");
    }
    let type_ = jvm.cpdtype_table.read().unwrap().get_cpdtype(*type_).unwrap_ref_type().clone();
    let rc = assert_inited_or_initing_class(jvm, type_.to_cpdtype());
    let object_type = runtime_class_to_allocated_object_type(rc.as_ref(), int_state.current_loader(jvm), None);
    let mut memory_region_guard = jvm.gc.memory_region.lock().unwrap();
    let object_size = object_type.size();
    let allocated_object = memory_region_guard.find_or_new_region_for(object_type).get_allocation();
    unsafe {
        libc::memset(allocated_object.as_ptr(), 0, object_size);
    }//todo do correct initing of fields
    unsafe { res_address.write(allocated_object) }
    drop(exit_guard);
    drop(guard);
    IRVMExitAction::RestartAtPtr { ptr: return_to_ptr }
}

pub fn trace_instruction_after<'gc>(jvm: &'gc JVMState<'gc>, int_state: &mut InterpreterStateGuard<'gc, '_>, exit_guard: &VMExitGuard, method_id: MethodId, return_to_ptr: *const c_void, bytecode_offset: ByteCodeOffset) -> IRVMExitAction {
    assert_eq!(Some(method_id), int_state.current_frame().frame_view.ir_ref.method_id());
    let (rc, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
    let view = rc.view();
    let method_view = view.method_view_i(method_i);
    let code = method_view.code_attribute().unwrap();
    let instr = code.instructions.get(&bytecode_offset).unwrap();
    eprintln!("After:{}/{:?}", jvm.method_table.read().unwrap().lookup_method_string(method_id, &jvm.string_pool), instr.info.better_debug_string(&jvm.string_pool));
    if !jvm.instruction_trace_options.partial_tracing() {
        // jvm.java_vm_state.assertion_state.lock().unwrap().handle_trace_after(jvm, instr, int_state);
    }
    dump_frame_contents(jvm, int_state);
    drop(exit_guard);
    IRVMExitAction::RestartAtPtr { ptr: return_to_ptr }
}

pub fn trace_instruction_before(jvm: &JVMState, exit_guard: &VMExitGuard, method_id: MethodId, return_to_ptr: *const c_void, bytecode_offset: ByteCodeOffset) -> IRVMExitAction {
    let (rc, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
    let view = rc.view();
    let method_view = view.method_view_i(method_i);
    let code = method_view.code_attribute().unwrap();
    let instr = code.instructions.get(&bytecode_offset).unwrap();
    eprintln!("Before:{:?} {}", instr.info.better_debug_string(&jvm.string_pool), bytecode_offset.0);
    if !jvm.instruction_trace_options.partial_tracing() {
        // jvm.java_vm_state.assertion_state.lock().unwrap().handle_trace_before(jvm, instr, int_state);
    }
    drop(exit_guard);
    IRVMExitAction::RestartAtPtr { ptr: return_to_ptr }
}

pub fn log_whole_frame<'gc>(jvm: &'gc JVMState<'gc>, int_state: &mut InterpreterStateGuard<'gc, '_>, exit_guard: &VMExitGuard, return_to_ptr: *const c_void) -> IRVMExitAction {
    if jvm.exit_trace_options.tracing_enabled() {
        eprintln!("LogWholeFrame");
    }
    let current_frame = int_state.current_frame();
    dbg!(current_frame.pc);
    let method_id = current_frame.frame_view.ir_ref.method_id().unwrap();
    let (rc, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
    let view = rc.view();
    let method_view = view.method_view_i(method_i);
    dbg!(method_view.name().0.to_str(&jvm.string_pool));
    dbg!(view.name().unwrap_name().0.to_str(&jvm.string_pool));
    dbg!(method_view.desc_str().to_str(&jvm.string_pool));
    current_frame.ir_stack_entry_debug_print();
    dump_frame_contents(jvm, int_state);
    drop(exit_guard);
    IRVMExitAction::RestartAtPtr { ptr: return_to_ptr }
}

pub fn log_frame_pointer_offset_value(jvm: &JVMState, value: u64, return_to_ptr: *const c_void) -> IRVMExitAction {
    if jvm.exit_trace_options.tracing_enabled() {
        eprintln!("value:{}", value);
    }
    IRVMExitAction::RestartAtPtr { ptr: return_to_ptr }
}

pub fn init_class_and_recompile<'gc>(jvm: &'gc JVMState<'gc>, int_state: &mut InterpreterStateGuard<'gc, '_>, exit_guard: &VMExitGuard, class_type: CPDTypeID, current_method_id: MethodId, restart_point: RestartPointID) -> IRVMExitAction {
    if jvm.exit_trace_options.tracing_enabled() {
        eprintln!("InitClassAndRecompile");
    }
    let cpdtype = jvm.cpdtype_table.read().unwrap().get_cpdtype(class_type).clone();
    drop(exit_guard);
    let inited = check_initing_or_inited_class(jvm, int_state, cpdtype).unwrap();
    let method_resolver = MethodResolver { jvm, loader: int_state.current_loader(jvm) };
    jvm.java_vm_state.add_method_if_needed(jvm, &method_resolver, current_method_id);
    let restart_point = jvm.java_vm_state.lookup_restart_point(current_method_id, restart_point);
    if jvm.exit_trace_options.tracing_enabled() {
        eprintln!("InitClassAndRecompile done");
    }
    IRVMExitAction::RestartAtPtr { ptr: restart_point }
}

pub fn put_static<'gc>(jvm: &'gc JVMState<'gc>, exit_guard: &VMExitGuard, field_id: &FieldId, value_ptr: &*mut c_void, return_to_ptr: &*const c_void) -> IRVMExitAction {
    if jvm.exit_trace_options.tracing_enabled() {
        eprintln!("PutStatic");
    }
    let (rc, field_i) = jvm.field_table.read().unwrap().lookup(*field_id);
    let view = rc.view();
    let field_view = view.field(field_i as usize);
    let mut static_vars_guard = static_vars(rc.deref(),jvm);
    let field_name = field_view.field_name();
    let native_jv = *unsafe { (*value_ptr as *mut NativeJavaValue<'gc>).as_ref() }.unwrap();
    let njv = native_to_new_java_value(native_jv,&field_view.field_type(), jvm);
    if let NewJavaValue::AllocObject(alloc) = njv.as_njv() {
        let rc = alloc.unwrap_normal_object().runtime_class(jvm);
        if instance_of_exit_impl(jvm, field_view.field_type(), Some(alloc.unwrap_normal_object())) == 0 {
            panic!()
        }
    }
    static_vars_guard.set(field_name, njv);
    drop(exit_guard);
    IRVMExitAction::RestartAtPtr { ptr: *return_to_ptr }
}

pub fn compile_function_and_recompile_current<'gc>(jvm: &'gc JVMState<'gc>, int_state: &mut InterpreterStateGuard<'gc, '_>, current_method_id: MethodId, to_recompile: MethodId, restart_point: RestartPointID) -> IRVMExitAction {
    if jvm.exit_trace_options.tracing_enabled() {
        eprintln!("CompileFunctionAndRecompileCurrent");
    }
    let method_resolver = MethodResolver { jvm, loader: int_state.current_loader(jvm) };
    jvm.java_vm_state.add_method_if_needed(jvm, &method_resolver, to_recompile);
    jvm.java_vm_state.add_method_if_needed(jvm, &method_resolver, current_method_id);
    let restart_point = jvm.java_vm_state.lookup_restart_point(current_method_id, restart_point);
    IRVMExitAction::RestartAtPtr { ptr: restart_point }
}

pub fn top_level_return(jvm: &JVMState, return_value: u64) -> IRVMExitAction {
    if jvm.exit_trace_options.tracing_enabled() {
        eprintln!("TopLevelReturn");
    }
    IRVMExitAction::ExitVMCompletely { return_data: return_value }
}

pub fn run_static_native<'gc>(jvm: &'gc JVMState<'gc>, int_state: &mut InterpreterStateGuard<'gc, '_>, method_id: MethodId, arg_start: *mut c_void, num_args: u16, res_ptr: *mut c_void, return_to_ptr: *mut c_void) -> IRVMExitAction {
    if jvm.exit_trace_options.tracing_enabled() {
        eprintln!("RunStaticNative");
    }
    let (rc, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
    let mut args_jv_handle = vec![];
    let class_view = rc.view();
    let method_view = class_view.method_view_i(method_i);
    let arg_types = &method_view.desc().arg_types;
    unsafe {
        for (i, cpdtype) in (0..num_args).zip(arg_types.iter()) {
            let arg_ptr = arg_start.offset(-(i as isize) * size_of::<jlong>() as isize) as *const u64;//stack grows down
            let native_jv = NativeJavaValue { as_u64: arg_ptr.read() };
            args_jv_handle.push(native_to_new_java_value(native_jv,cpdtype, jvm))
        }
    }
    assert!(jvm.thread_state.int_state_guard_valid.get().borrow().clone());
    let args_new_jv = args_jv_handle.iter().map(|handle| handle.as_njv()).collect();
    let res = match run_native_method(jvm, int_state, rc, method_i, args_new_jv) {
        Ok(x) => x,
        Err(WasException {}) => {
            let expception_obj_handle = int_state.throw().unwrap().duplicate_discouraged();
            int_state.set_throw(None);
            return throw_impl(jvm, int_state, expception_obj_handle.new_java_handle());
        }
    };
    assert!(int_state.throw().is_none());
    if let Some(res) = res {
        unsafe { (res_ptr as *mut NativeJavaValue<'static>).write(transmute::<NativeJavaValue<'_>, NativeJavaValue<'static>>(res.to_native())) }
    };
    if !jvm.instruction_trace_options.partial_tracing() {
        // jvm.java_vm_state.assertion_state.lock().unwrap().current_before.pop().unwrap();
    }
    IRVMExitAction::RestartAtPtr { ptr: return_to_ptr }
}

pub fn allocate_object_array<'gc>(jvm: &'gc JVMState<'gc>, int_state: &mut InterpreterStateGuard<'gc, '_>, type_: CPDTypeID, len: i32, return_to_ptr: *const c_void, res_address: *mut NonNull<c_void>) -> IRVMExitAction {
    if jvm.exit_trace_options.tracing_enabled() {
        eprintln!("AllocateObjectArray");
    }
    let type_ = jvm.cpdtype_table.read().unwrap().get_cpdtype(type_).unwrap_ref_type().clone();
    assert!(len >= 0);
    let rc = assert_inited_or_initing_class(jvm, type_.to_cpdtype());
    let object_array = runtime_class_to_allocated_object_type(rc.as_ref(), int_state.current_loader(jvm), Some(len as usize));
    let mut memory_region_guard = jvm.gc.memory_region.lock().unwrap();
    let array_size = object_array.size();
    let region_data = memory_region_guard.find_or_new_region_for(object_array);
    let allocated_object = region_data.get_allocation();
    unsafe { res_address.write(allocated_object) }
    unsafe {
        memset(allocated_object.as_ptr(), 0, array_size);
    }//todo init this properly according to type
    unsafe { *allocated_object.cast::<jint>().as_mut() = len }//init the length
    IRVMExitAction::RestartAtPtr { ptr: return_to_ptr }
}

pub fn throw_impl<'gc>(jvm: &'gc JVMState<'gc>, int_state: &mut InterpreterStateGuard<'gc, '_>, exception_obj_handle: NewJavaValueHandle<'gc>) -> IRVMExitAction {
    let exception_object_handle = exception_obj_handle.unwrap_object_nonnull();
    let throwable = exception_object_handle.cast_throwable();
    let exception_as_string = throwable.to_string(jvm, int_state).unwrap().unwrap();
    dbg!(exception_as_string.to_rust_string(jvm));
    let exception_object_handle = throwable.normal_object;
    let exception_obj = &exception_object_handle;
    let exception_obj_rc = exception_obj.runtime_class(jvm);
    for current_frame in int_state.frame_iter() {
        let rc = match current_frame.try_class_pointer(jvm) {
            None => continue,
            Some(rc) => rc
        };
        let view = rc.view();
        let method_i = current_frame.method_i(jvm);
        let method_view = view.method_view_i(method_i);
        if let Some(code) = method_view.code_attribute() {
            let current_pc = current_frame.pc(jvm);
            for CompressedExceptionTableElem {
                start_pc,
                end_pc,
                handler_pc,
                catch_type
            } in &code.exception_table {
                let matches_class = match catch_type {
                    None => true,
                    Some(class_name) => {
                        instance_of_exit_impl_impl(jvm, CompressedParsedRefType::Class(*class_name), exception_obj) == 1
                    }
                };
                if *start_pc <= current_pc && current_pc < *end_pc && matches_class {
                    let ir_method_id = current_frame.frame_view.ir_ref.ir_method_id().unwrap();
                    let handler_address = jvm.java_vm_state.lookup_byte_code_offset(ir_method_id, *handler_pc);
                    let handler_rbp = current_frame.frame_view.ir_ref.frame_ptr();
                    let frame_size = current_frame.frame_view.ir_ref.frame_size(&jvm.java_vm_state.ir);
                    let handler_rsp = unsafe { handler_rbp.sub(frame_size) };
                    //todo need to set caught exception in stack
                    let mut start_diff = SavedRegistersWithIPDiff::no_change();
                    start_diff.saved_registers_without_ip.rbp = Some(handler_rbp);
                    start_diff.saved_registers_without_ip.rsp = Some(handler_rsp);
                    start_diff.rip = Some(handler_address);
                    return IRVMExitAction::RestartWithRegisterState {
                        diff: start_diff
                    };
                }
            }
        }
    }
    jvm.perf_metrics.display();
    todo!()
}

pub fn virtual_args_extract<'gc>(jvm: &'gc JVMState<'gc>, arg_types: &[CompressedParsedDescriptorType], mut arg_start: *const c_void) -> Vec<NewJavaValueHandle<'gc>> {
    let obj_ref_native = unsafe { arg_start.cast::<NativeJavaValue>().read() };
    let obj_ref = native_to_new_java_value(obj_ref_native,&CClassName::object().into(), jvm);
    let mut args_jv_handle = vec![];
    args_jv_handle.push(obj_ref);
    unsafe {
        arg_start = arg_start.sub(size_of::<NativeJavaValue>());
        for (i, cpdtype) in (0..arg_types.len()).zip(arg_types.iter()) {
            let arg_ptr = arg_start.sub(i * size_of::<jlong>()) as *const u64;
            let native_jv = NativeJavaValue { as_u64: arg_ptr.read() };
            args_jv_handle.push(native_to_new_java_value(native_jv,cpdtype, jvm))
        }
    }
    args_jv_handle
}
