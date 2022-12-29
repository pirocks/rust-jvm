use std::ffi::c_void;
use std::mem::size_of;
use std::ops::Deref;
use std::ptr::NonNull;

use libc::memset;

use another_jit_vm::Register;
use another_jit_vm::saved_registers_utils::{SavedRegistersWithIPDiff, SavedRegistersWithoutIPDiff};
use another_jit_vm_ir::compiler::RestartPointID;
use another_jit_vm_ir::ir_stack::{IsOpaque, read_frame_ir_header};
use another_jit_vm_ir::IRVMExitAction;
use another_jit_vm_ir::vm_exit_abi::register_structs::InvokeVirtualResolve;
use gc_memory_layout_common::allocated_object_types::AllocatedObjectType;
use interface_vtable::{InterfaceVTableEntry, ITable, ResolvedInterfaceVTableEntry};
use jvmti_jni_bindings::{jint, jlong};
use method_table::interface_table::InterfaceID;
use runtime_class_stuff::method_numbers::MethodNumber;
use rust_jvm_common::{ByteCodeOffset, FieldId, MethodId, MethodTableIndex, StackNativeJavaValue};
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::code::CompressedExceptionTableElem;
use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CompressedParsedDescriptorType, CompressedParsedRefType};
use rust_jvm_common::compressed_classfile::method_names::MethodName;


use rust_jvm_common::cpdtype_table::CPDTypeID;
use rust_jvm_common::loading::LoaderName;
use rust_jvm_common::method_shape::{MethodShape, MethodShapeID};
use rust_jvm_common::runtime_type::RuntimeType;
use sketch_jvm_version_of_utf8::wtf8_pool::CompressedWtf8String;
use compiler_common::MethodResolver;
use gc_memory_layout_common::memory_regions::MemoryRegions;
use vtable::{RawNativeVTable, ResolvedVTableEntry, VTable, VTableEntry};

use crate::{check_initing_or_inited_class, JavaValueCommon, JString, JVMState, MethodResolverImpl, NewAsObjectOrJavaValue, NewJavaValueHandle};
use crate::better_java_stack::exit_frame::JavaExitFrame;
use crate::better_java_stack::frames::{HasFrame, PushableFrame};
use crate::class_loading::assert_inited_or_initing_class;
use crate::interpreter::common::invoke::virtual_::virtual_method_lookup;
use crate::interpreter::common::special::{instance_of_exit_impl, instance_of_exit_impl_impl};
use crate::ir_to_java_layer::dump_frame::dump_frame_contents;
use crate::ir_to_java_layer::java_stack::OpaqueFrameIdOrMethodID;
use crate::java_values::{native_to_new_java_value_rtype};
use crate::jit::{NotCompiledYet, ResolvedInvokeVirtual};
use crate::jit::state::runtime_class_to_allocated_object_type;
use crate::new_java_values::owned_casts::OwnedCastAble;
use crate::static_vars::static_vars;
use crate::stdlib::java::lang::array_out_of_bounds_exception::ArrayOutOfBoundsException;
use crate::stdlib::java::lang::class::JClass;
use crate::stdlib::java::lang::throwable::Throwable;
use crate::utils::{lookup_method_parsed};

pub mod multi_allocate_array;
pub mod new_run_native;

#[inline(never)]
pub fn array_out_of_bounds<'gc, 'k>(jvm: &'gc JVMState<'gc>, int_state: &mut JavaExitFrame<'gc, 'k>, index: i32) -> IRVMExitAction {
    if jvm.exit_tracing_options.tracing_enabled() {
        eprintln!("ArrayOutOfBounds");
    }
    let array_out_of_bounds = ArrayOutOfBoundsException::new(jvm, int_state, index).unwrap();
    let throwable = array_out_of_bounds.object().cast_throwable();
    throw_impl(&jvm, int_state, throwable, false)
}

#[inline(never)]
pub fn throw_exit<'gc, 'k>(jvm: &'gc JVMState<'gc>, int_state: &mut JavaExitFrame<'gc, 'k>, exception_obj_ptr: *const c_void) -> IRVMExitAction {
    let throw = jvm.perf_metrics.vm_exit_throw();
    if jvm.exit_tracing_options.tracing_enabled() {
        eprintln!("Throw");
    }
    let exception_obj_native_value = unsafe { (exception_obj_ptr).cast::<StackNativeJavaValue<'gc>>().read() };
    let exception_obj_handle = native_to_new_java_value_rtype(exception_obj_native_value, CClassName::object().into(), jvm);
    let throwable = exception_obj_handle.cast_throwable();
    let res = throw_impl(&jvm, int_state, throwable, false);
    drop(throw);
    res
}

#[inline(never)]
pub fn invoke_interface_resolve<'gc>(
    jvm: &'gc JVMState<'gc>,
    int_state: &mut JavaExitFrame<'gc, '_>,
    return_to_ptr: *const c_void,
    _native_method_restart_point: RestartPointID,
    _native_method_res: *mut c_void,
    object_ref: *const c_void,
    target_method_shape_id: MethodShapeID,
    interface_id: InterfaceID,
    method_number: MethodNumber,
) -> IRVMExitAction {
    if jvm.exit_tracing_options.tracing_enabled() {
        eprintln!("InvokeInterfaceResolve");
    }
    let _caller_method_id = int_state.frame_ref().method_id().unwrap();
    let obj_native_jv = unsafe { (object_ref).cast::<StackNativeJavaValue>().read() };
    let obj_jv_handle = native_to_new_java_value_rtype(obj_native_jv, RuntimeType::object(), jvm);
    let obj_rc = obj_jv_handle.unwrap_object_nonnull().runtime_class(jvm);
    let target_rc = jvm.interface_table.lookup(interface_id);
    let method_shape = jvm.method_shapes.lookup_method_shape(target_method_shape_id);
    let resolver = MethodResolverImpl { jvm, loader: int_state.current_loader(jvm) };
    let read_guard = jvm.invoke_interface_lookup_cache.read().unwrap();
    let itable = jvm.itables.lock().unwrap().lookup_or_new_itable(&jvm.interface_table, obj_rc.clone());
    let method_number_check = *target_rc.unwrap_class_class().method_numbers.get(&method_shape).unwrap();
    assert_eq!(method_number, method_number_check);
    let res = match ITable::lookup(itable, interface_id, method_number)/*read_guard.lookup(obj_rc.clone(), method_name, method_desc.clone())*/ {
        None => {
            let (resolved_method_i, resolved_rc) = lookup_method_parsed(jvm, obj_rc.clone(), method_shape.name, &method_shape.desc).unwrap();
            let resolved_method_id = jvm.method_table.write().unwrap().get_method_id(resolved_rc.clone(), resolved_method_i);
            drop(read_guard);
            jvm.java_vm_state.add_method_if_needed(jvm, &resolver, resolved_method_id, false);
            let resolved = resolved_entry_from_method_id(jvm, resolver, resolved_method_id);
            jvm.itables.lock().unwrap().set_entry(obj_rc, interface_id, method_number, resolved.address);
            InterfaceVTableEntry { address: Some(resolved.address) }
        }
        Some(resolved) => {
            dbg!("already resolved");
            resolved
        }
    };
    let InterfaceVTableEntry { address } = res;
    let mut start_diff = SavedRegistersWithoutIPDiff::no_change();
    start_diff.add_change(InvokeVirtualResolve::ADDRESS_RES, address.unwrap().as_ptr() as u64);
    IRVMExitAction::RestartWithRegisterState {
        diff: SavedRegistersWithIPDiff {
            rip: Some(return_to_ptr),
            saved_registers_without_ip: start_diff,
        }
    }
}

fn resolved_entry_from_method_id(jvm: &JVMState, resolver: MethodResolverImpl, resolved_method_id: MethodTableIndex) -> ResolvedInterfaceVTableEntry {
    let new_frame_size = if resolver.is_native(resolved_method_id) {
        resolver.lookup_native_method_layout(resolved_method_id).full_frame_size()
    } else {
        resolver.lookup_partial_method_layout(resolved_method_id).full_frame_size()
    };
    let ir_method_id = jvm.java_vm_state.lookup_method_ir_method_id(resolved_method_id);
    let address = jvm.java_vm_state.ir.lookup_ir_method_id_pointer(ir_method_id);
    let resolved = ResolvedInterfaceVTableEntry {
        address,
        ir_method_id,
        method_id: resolved_method_id,
        new_frame_size,
    };
    resolved
}

#[inline(never)]
pub fn check_cast<'gc, 'k>(jvm: &'gc JVMState<'gc>, _int_state: &mut JavaExitFrame<'gc, 'k>, value: &*const c_void, cpdtype_id: &CPDTypeID, return_to_ptr: &*const c_void) -> IRVMExitAction {
    let checkcast = jvm.perf_metrics.vm_exit_checkcast();
    if jvm.exit_tracing_options.tracing_enabled() {
        eprintln!("CheckCast");
    }
    let cpdtype = jvm.cpdtype_table.read().unwrap().get_cpdtype(*cpdtype_id).clone();
    let value = unsafe { (*value).cast::<StackNativeJavaValue>().read() };
    let value = native_to_new_java_value_rtype(value, CClassName::object().into(), jvm);
    let value = value.unwrap_object();
    if let Some(handle) = value {
        let res_int = instance_of_exit_impl(jvm, cpdtype, Some(&handle));
        if res_int == 0 {
            dbg!(cpdtype.jvm_representation(&jvm.string_pool));
            dbg!(handle.runtime_class(jvm).cpdtype().jvm_representation(&jvm.string_pool));
            /*int_state.debug_print_stack_trace(jvm);*/
            todo!()
        }
    }
    drop(checkcast);
    IRVMExitAction::RestartAtPtr { ptr: *return_to_ptr }
}

#[inline(never)]
pub fn instance_of<'gc, 'l, 'k>(jvm: &'gc JVMState<'gc>, int_state: &mut JavaExitFrame<'gc, 'k>, res: &*mut c_void, value: &*const c_void, cpdtype_id: &CPDTypeID, return_to_ptr: &*const c_void) -> IRVMExitAction {
    if jvm.exit_tracing_options.tracing_enabled() {
        eprintln!("InstanceOf");
    }
    let cpdtype = *jvm.cpdtype_table.read().unwrap().get_cpdtype(*cpdtype_id);
    let value = unsafe { (*value).cast::<StackNativeJavaValue>().read() };
    let value = native_to_new_java_value_rtype(value, CClassName::object().into(), jvm);
    let value = value.unwrap_object();
    check_initing_or_inited_class(jvm, int_state, cpdtype).unwrap();
    let res_int = instance_of_exit_impl(jvm, cpdtype, value.as_ref());
    unsafe { (*((*res) as *mut StackNativeJavaValue)).int = res_int };
    IRVMExitAction::RestartAtPtr { ptr: *return_to_ptr }
}


#[inline(never)]
pub fn assert_instance_of<'gc, 'l, 'k>(jvm: &'gc JVMState<'gc>, int_state: &mut JavaExitFrame<'gc, 'k>, res: &*mut c_void, value: &*const c_void, cpdtype_id: &CPDTypeID, return_to_ptr: &*const c_void, expected: bool) -> IRVMExitAction {
    if jvm.exit_tracing_options.tracing_enabled() {
        eprintln!("InstanceOf");
    }
    let cpdtype = *jvm.cpdtype_table.read().unwrap().get_cpdtype(*cpdtype_id);
    let value = unsafe { (*value).cast::<StackNativeJavaValue>().read() };
    let value = native_to_new_java_value_rtype(value, CClassName::object().into(), jvm);
    let value = value.unwrap_object();
    let _initied = check_initing_or_inited_class(jvm, int_state, cpdtype).unwrap();
    let res_int = instance_of_exit_impl(jvm, cpdtype, value.as_ref());
    dbg!(&value.as_ref().unwrap().runtime_class(jvm).cpdtype().jvm_representation(&jvm.string_pool));
    dbg!(cpdtype.jvm_representation(&jvm.string_pool));
    assert_eq!(res_int, if expected { 1 } else { 0 });
    unsafe { (*((*res) as *mut StackNativeJavaValue)).int = res_int };
    IRVMExitAction::RestartAtPtr { ptr: *return_to_ptr }
}

#[inline(never)]
pub fn monitor_exit<'gc, 'k>(jvm: &'gc JVMState<'gc>, int_state: &mut JavaExitFrame<'gc, 'k>, obj_ptr: *const c_void, return_to_ptr: *const c_void) -> IRVMExitAction {
    if jvm.exit_tracing_options.tracing_enabled() {
        eprintln!("MonitorExit");
    }
    let monitor = jvm.monitor_for(obj_ptr);
    int_state.to_interpreter_frame(|interpreter_frame| {
        monitor.unlock(jvm, interpreter_frame).unwrap();
    });
    IRVMExitAction::RestartAtPtr { ptr: return_to_ptr }
}

#[inline(never)]
pub fn monitor_enter<'gc, 'k>(jvm: &'gc JVMState<'gc>, int_state: &mut JavaExitFrame<'gc, 'k>, obj_ptr: *const c_void, return_to_ptr: *const c_void) -> IRVMExitAction {
    if jvm.exit_tracing_options.tracing_enabled() {
        eprintln!("MonitorEnter");
    }
    let monitor = jvm.monitor_for(obj_ptr);
    int_state.to_interpreter_frame(|interpreter_frame| {
        monitor.lock(jvm, interpreter_frame).unwrap();
    });
    IRVMExitAction::RestartAtPtr { ptr: return_to_ptr }
}

#[inline(never)]
pub fn invoke_virtual_resolve<'gc, 'k>(
    jvm: &'gc JVMState<'gc>,
    int_state: &mut JavaExitFrame<'gc, 'k>,
    return_to_ptr: *const c_void,
    object_ref_ptr: *const c_void,
    method_shape_id: MethodShapeID,
    method_number: MethodNumber,
) -> IRVMExitAction {
    if jvm.exit_tracing_options.tracing_enabled() {
        eprintln!("InvokeVirtualResolve");
    }
    //todo this is probably wrong what if there's a class with a same name private method?
    // like surely I need to start at the classname specified in the bytecode
    let memory_region_guard = jvm.gc.memory_region.lock().unwrap();
    let maybe_non_null = NonNull::new(unsafe { (object_ref_ptr as *const *mut c_void).read() });
    let vtable = MemoryRegions::find_type_vtable(maybe_non_null.unwrap()).unwrap();
    let vtable_lookup_res = VTable::lookup(vtable, method_number);
    //todo actually use vtable lookup res
    let res = match vtable_lookup_res {
        None => {
            let allocated_type = memory_region_guard.find_object_allocated_type(maybe_non_null.unwrap()).clone();
            // let allocated_type_id = memory_region_guard.lookup_or_add_type(&allocated_type);
            drop(vtable);//make sure vtable is always guarded by memory region lock
            drop(memory_region_guard);

            let MethodShape { name, desc } = jvm.method_shapes.lookup_method_shape(method_shape_id);
            let res = invoke_virtual_full(
                jvm,
                int_state,
                method_number,
                name,
                &desc,
                allocated_type,
                vtable,
            );
            res
        }
        Some(res) => {
            res
        }
    };

    assert!(VTable::lookup(vtable, method_number).is_some());

    let ResolvedVTableEntry {
        address,
    } = res;

    let mut start_diff = SavedRegistersWithoutIPDiff::no_change();
    start_diff.add_change(InvokeVirtualResolve::ADDRESS_RES, address.as_ptr() as u64);
    IRVMExitAction::RestartWithRegisterState {
        diff: SavedRegistersWithIPDiff {
            rip: Some(return_to_ptr),
            saved_registers_without_ip: start_diff,
        }
    }
}

fn invoke_virtual_full<'gc>(
    jvm: &'gc JVMState<'gc>,
    int_state: &mut impl PushableFrame<'gc>,
    method_number: MethodNumber,
    name: MethodName,
    desc: &CMethodDescriptor,
    allocated_type: AllocatedObjectType,
    vtable_from_region: NonNull<RawNativeVTable>,
) -> ResolvedVTableEntry {
    let rc = match allocated_type {
        AllocatedObjectType::Class { name, vtable, .. } => {
            assert_eq!(vtable_from_region, vtable);
            assert_inited_or_initing_class(jvm, (name).into())
        }
        AllocatedObjectType::PrimitiveArray { .. } |
        AllocatedObjectType::ObjectArray { .. } => {
            assert_inited_or_initing_class(jvm, CClassName::object().into())
        }
        AllocatedObjectType::RawConstantSize { .. } => {
            panic!()
        }
    };
    let (resolved_rc, method_i) = virtual_method_lookup(jvm, int_state, name, &desc, rc.clone()).unwrap();
    let method_id = jvm.method_table.write().unwrap().get_method_id(resolved_rc.clone(), method_i);
    let method_resolver = MethodResolverImpl { jvm, loader: int_state.current_loader(jvm) };
    if jvm.java_vm_state.try_lookup_ir_method_id(OpaqueFrameIdOrMethodID::Method { method_id: method_id as u64 }).is_none() {
        //todo needs way to exit to interpreter
        jvm.java_vm_state.add_method_if_needed(jvm, &method_resolver, method_id, false);
    }

    let ResolvedInvokeVirtual {
        address,
        ..
    } = match jvm.java_vm_state.lookup_resolved_invoke_virtual(method_id, &method_resolver) {
        Ok(resolved) => {
            resolved
        }
        Err(NotCompiledYet { needs_compiling }) => {
            // let rc = assert_loaded_class(jvm, allocated_type.as_cpdtype());
            jvm.java_vm_state.add_method_if_needed(jvm, &method_resolver, needs_compiling, false);
            // jvm.java_vm_state.add_method(jvm, &method_resolver, *debug_method_id);
            // jvm.java_vm_state.add_method(jvm, &method_resolver, method_id);
            dbg!(needs_compiling);
            todo!()
            // jvm.vtables.read().unwrap().lookup_resolved(allocated_type_id, *inheritance_id).unwrap()
        }
    };
    let resolved_vtable_entry = VTableEntry {
        address: Some(NonNull::new(address as *mut c_void).unwrap()),
    };
    jvm.vtables.lock().unwrap().vtable_register_entry(rc, method_number, resolved_vtable_entry);
    // jvm.vtable.lock().unwrap().vtable_register_entry(resolved_rc, method_number, resolved_vtable_entry,vtable_from_region);

    resolved_vtable_entry.resolved().unwrap()
}

fn new_class_impl<'gc, 'k>(jvm: &'gc JVMState<'gc>, int_state: &mut JavaExitFrame<'gc, 'k>, type_: CPDTypeID) -> NewJavaValueHandle<'gc> {
    let cpdtype = jvm.cpdtype_table.write().unwrap().get_cpdtype(type_).clone();
    let jclass = JClass::from_type(jvm, int_state, cpdtype).unwrap();
    jclass.new_java_value_handle()
}

#[inline(never)]
pub fn new_class<'gc, 'k>(jvm: &'gc JVMState<'gc>, int_state: &mut JavaExitFrame<'gc, 'k>, type_: CPDTypeID, res: *mut c_void, return_to_ptr: *const c_void) -> IRVMExitAction {
    if jvm.exit_tracing_options.tracing_enabled() {
        eprintln!("NewClass");
    }
    let jv_new_handle = new_class_impl(jvm, int_state, type_);
    unsafe {
        let raw_64 = jv_new_handle.as_njv().to_stack_native().as_u64;
        (res as *mut u64).write(raw_64);
    };
    IRVMExitAction::RestartAtPtr { ptr: return_to_ptr }
}

#[inline(never)]
pub fn new_class_register<'gc, 'k>(jvm: &'gc JVMState<'gc>, int_state: &mut JavaExitFrame<'gc, 'k>, type_: CPDTypeID, _res: Register, return_to_ptr: *const c_void) -> IRVMExitAction {
    if jvm.exit_tracing_options.tracing_enabled() {
        eprintln!("NewClassRegister");
    }
    let jv_new_handle = new_class_impl(jvm, int_state, type_);
    let mut diff = SavedRegistersWithIPDiff::no_change();
    unsafe {
        let raw_64 = jv_new_handle.as_njv().to_stack_native().as_u64;
        diff.saved_registers_without_ip.rbx = Some(raw_64);
    };
    diff.rip = Some(return_to_ptr);
    IRVMExitAction::RestartWithRegisterState { diff }
}

#[inline(never)]
pub fn new_string<'gc, 'k>(jvm: &'gc JVMState<'gc>, int_state: &mut JavaExitFrame<'gc, 'k>, return_to_ptr: *const c_void, res: *mut c_void, compressed_wtf8: CompressedWtf8String) -> IRVMExitAction {
    if jvm.exit_tracing_options.tracing_enabled() {
        eprintln!("NewString");
    }
    let read_guard = jvm.string_exit_cache.read().unwrap();
    let native = match read_guard.lookup(compressed_wtf8) {
        None => {
            drop(read_guard);
            let wtf8buf = compressed_wtf8.to_wtf8(&jvm.wtf8_pool);
            let jstring = JString::from_rust(jvm, int_state, wtf8buf).expect("todo exceptions").intern(jvm, int_state).unwrap();
            jvm.string_exit_cache.write().unwrap().register_entry(compressed_wtf8, jstring.clone());
            let jv = jstring.new_java_value();
            jv.to_stack_native()
        }
        Some(jstring) => {
            let jv = jstring.new_java_value();
            jv.to_stack_native()
        }
    };
    unsafe {
        let raw_u64 = native.as_u64;
        (res as *mut u64).write(raw_u64);
    }
    IRVMExitAction::RestartAtPtr { ptr: return_to_ptr }
}


#[inline(never)]
pub fn allocate_object<'gc>(jvm: &'gc JVMState<'gc>, current_loader: LoaderName, type_: &CPDTypeID, return_to_ptr: *const c_void, res_address: &*mut NonNull<c_void>) -> IRVMExitAction {
    let guard = jvm.perf_metrics.vm_exit_allocate_obj();
    if jvm.exit_tracing_options.tracing_enabled() {
        eprintln!("AllocateObject");
    }
    let type_ = jvm.cpdtype_table.read().unwrap().get_cpdtype(*type_).unwrap_ref_type().clone();
    let rc = assert_inited_or_initing_class(jvm, type_.to_cpdtype());
    let object_type = runtime_class_to_allocated_object_type(jvm, rc.clone(), current_loader, None);
    let mut memory_region_guard = jvm.gc.memory_region.lock().unwrap();
    let (allocated_object, object_size) = memory_region_guard.allocate_with_size(&object_type);
    unsafe {
        memset(allocated_object.as_ptr(), 0, object_size.get());
    }//todo do correct initing of fields
    unsafe { res_address.write(allocated_object) }
    drop(guard);
    IRVMExitAction::RestartAtPtr { ptr: return_to_ptr }
}

pub fn trace_instruction_after<'gc, 'k>(jvm: &'gc JVMState<'gc>, int_state: &mut JavaExitFrame<'gc, 'k>, method_id: MethodId, return_to_ptr: *const c_void, bytecode_offset: ByteCodeOffset) -> IRVMExitAction {
    let (rc, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
    let view = rc.view();
    let method_view = view.method_view_i(method_i);
    let code = method_view.code_attribute().unwrap();
    let instr = code.instructions.get(&bytecode_offset).unwrap();
    eprintln!("After:{}/{:?}", jvm.method_table.read().unwrap().lookup_method_string(method_id, &jvm.string_pool), instr.info.better_debug_string(&jvm.string_pool));
    if !jvm.instruction_tracing_options.partial_tracing() {
        // jvm.java_vm_state.assertion_state.lock().unwrap().handle_trace_after(jvm, instr, int_state);
    }
    dump_frame_contents(jvm, int_state);
    IRVMExitAction::RestartAtPtr { ptr: return_to_ptr }
}


pub fn trace_instruction_before(jvm: &JVMState, method_id: MethodId, return_to_ptr: *const c_void, bytecode_offset: ByteCodeOffset) -> IRVMExitAction {
    let (rc, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
    let view = rc.view();
    let method_view = view.method_view_i(method_i);
    let code = method_view.code_attribute().unwrap();
    let instr = code.instructions.get(&bytecode_offset).unwrap();
    eprintln!("Before:{:?} {}", instr.info.better_debug_string(&jvm.string_pool), bytecode_offset.0);
    if !jvm.instruction_tracing_options.partial_tracing() {
        // jvm.java_vm_state.assertion_state.lock().unwrap().handle_trace_before(jvm, instr, int_state);
    }
    IRVMExitAction::RestartAtPtr { ptr: return_to_ptr }
}

pub fn log_whole_frame<'gc, 'k>(jvm: &'gc JVMState<'gc>, _int_state: &mut JavaExitFrame<'gc, 'k>, _return_to_ptr: *const c_void) -> IRVMExitAction {
    if jvm.exit_tracing_options.tracing_enabled() {
        eprintln!("LogWholeFrame");
    }
    todo!()/*let current_frame = int_state.current_frame();
    dbg!(current_frame.pc);
    let method_id = current_frame.frame_view.ir_ref.method_id().unwrap();
    let (rc, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
    let view = rc.view();
    let method_view = view.method_view_i(method_i);
    dbg!(method_view.name().0.to_str(&jvm.string_pool));
    dbg!(view.name().unwrap_name().0.to_str(&jvm.string_pool));
    dbg!(method_view.desc_str().to_str(&jvm.string_pool));
    current_frame.ir_stack_entry_debug_print();
    dump_frame_contents(jvm, int_state);*/
    // IRVMExitAction::RestartAtPtr { ptr: return_to_ptr }
}

pub fn log_frame_pointer_offset_value(jvm: &JVMState, value: u64, return_to_ptr: *const c_void) -> IRVMExitAction {
    if jvm.exit_tracing_options.tracing_enabled() {
        eprintln!("value:{}", value);
    }
    IRVMExitAction::RestartAtPtr { ptr: return_to_ptr }
}

#[inline(never)]
pub fn init_class_and_recompile<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &mut JavaExitFrame<'gc, 'l>, class_type: CPDTypeID, current_method_id: MethodId, restart_point: RestartPointID) -> IRVMExitAction {
    if jvm.exit_tracing_options.tracing_enabled() {
        eprintln!("InitClassAndRecompile");
    }
    let cpdtype = jvm.cpdtype_table.read().unwrap().get_cpdtype(class_type).clone();
    let _ = check_initing_or_inited_class(jvm, int_state, cpdtype).unwrap();
    assert!(jvm.classes.read().unwrap().is_inited_or_initing(&cpdtype).is_some());
    let method_resolver = MethodResolverImpl { jvm, loader: int_state.current_loader(jvm) };
    jvm.java_vm_state.add_method_if_needed(jvm, &method_resolver, current_method_id, false);
    let restart_point = jvm.java_vm_state.lookup_restart_point(
        current_method_id,
        restart_point,
    );
    if jvm.exit_tracing_options.tracing_enabled() {
        eprintln!("InitClassAndRecompile done");
    }
    IRVMExitAction::RestartAtPtr { ptr: restart_point }
}

#[inline(never)]
pub fn put_static<'gc>(jvm: &'gc JVMState<'gc>, field_id: &FieldId, value_ptr: &*mut c_void, return_to_ptr: &*const c_void) -> IRVMExitAction {
    if jvm.exit_tracing_options.tracing_enabled() {
        eprintln!("PutStatic");
    }
    let (rc, field_i) = jvm.field_table.read().unwrap().lookup(*field_id);
    let view = rc.view();
    let field_view = view.field(field_i as usize);
    let mut static_vars_guard = static_vars(rc.deref(), jvm);
    let field_name = field_view.field_name();
    let native_jv = *unsafe { (*value_ptr as *mut StackNativeJavaValue<'gc>).as_ref() }.unwrap();
    let njv = native_to_new_java_value_rtype(native_jv, field_view.field_type().to_runtime_type().unwrap(), jvm);
    static_vars_guard.set(field_name, njv);
    IRVMExitAction::RestartAtPtr { ptr: *return_to_ptr }
}

#[inline(never)]
pub fn compile_function_and_recompile_current<'gc>(jvm: &'gc JVMState<'gc>, current_loader: LoaderName, current_method_id: MethodId, to_recompile: MethodId, restart_point: RestartPointID) -> IRVMExitAction {
    if jvm.exit_tracing_options.tracing_enabled() {
        eprintln!("CompileFunctionAndRecompileCurrent");
    }
    let method_resolver = MethodResolverImpl { jvm, loader: current_loader };
    jvm.java_vm_state.add_method_if_needed(jvm, &method_resolver, to_recompile, false);
    jvm.java_vm_state.add_method_if_needed(jvm, &method_resolver, current_method_id, false);
    let restart_point = jvm.java_vm_state.lookup_restart_point(current_method_id, restart_point);
    IRVMExitAction::RestartAtPtr { ptr: restart_point }
}

#[inline(never)]
pub fn top_level_return(jvm: &JVMState, return_value: u64) -> IRVMExitAction {
    if jvm.exit_tracing_options.tracing_enabled() {
        eprintln!("TopLevelReturn");
    }
    IRVMExitAction::ExitVMCompletely { return_data: return_value }
}

#[inline(never)]
pub fn allocate_object_array<'gc, 'k>(jvm: &'gc JVMState<'gc>, int_state: &mut JavaExitFrame<'gc, 'k>, type_: CPDTypeID, len: jint, return_to_ptr: *const c_void, res_address: *mut NonNull<c_void>) -> IRVMExitAction {
    if jvm.exit_tracing_options.tracing_enabled() {
        eprintln!("AllocateObjectArray");
    }
    let type_ = jvm.cpdtype_table.read().unwrap().get_cpdtype(type_).unwrap_ref_type().clone();
    if !type_.is_array(){
        dbg!(type_.jvm_representation(&jvm.string_pool));
        dbg!(type_);
        todo!();
    }
    assert!(len >= 0);
    // int_state.debug_print_stack_trace(jvm);
    let rc = check_initing_or_inited_class(jvm,int_state, type_.to_cpdtype()).expect("exception initing an array object but those don't have an initializer?");
    //todo fix current_loader
    let object_array = runtime_class_to_allocated_object_type(jvm, rc.clone(), int_state.current_loader(jvm), Some(len));
    let mut memory_region_guard = jvm.gc.memory_region.lock().unwrap();
    let allocated_object = memory_region_guard.allocate(&object_array);
    unsafe { res_address.write(allocated_object) }
    unsafe {
        memset(allocated_object.as_ptr(), 0, object_array.size.get());
    }//todo init this properly according to type
    unsafe { *allocated_object.cast::<jint>().as_mut() = len }//init the length
    assert!(memory_region_guard.find_object_allocated_type(allocated_object).as_cpdtype().is_array());
    IRVMExitAction::RestartAtPtr { ptr: return_to_ptr }
}

pub fn throw_impl<'gc, 'k>(jvm: &'gc JVMState<'gc>, int_state: &mut JavaExitFrame<'gc, 'k>, throwable: Throwable<'gc>, ignore_this_frame: bool) -> IRVMExitAction {
    // let exception_as_string = throwable.to_string(jvm, int_state).unwrap().unwrap();
    // dbg!(exception_as_string.to_rust_string(jvm));
    // throwable.print_stack_trace(jvm,int_state).unwrap();
    // let _exception_obj_rc = throwable.normal_object.runtime_class(jvm);
    let mut this_frame = true;
    for current_frame in int_state.frame_iter() {
        if this_frame && ignore_this_frame {
            this_frame = false;
            continue;
        }
        let rc = match current_frame.try_class_pointer(jvm) {
            Err(IsOpaque{}) => {
                continue;
            }
            Ok(rc) => rc
        };
        let view = rc.view();
        let method_i = current_frame.method_i();
        let method_view = view.method_view_i(method_i);
        if let Some(code) = method_view.code_attribute() {
            let current_pc = match current_frame.try_pc() {
                None => {
                    return IRVMExitAction::Exception { throwable: throwable.normal_object.ptr };
                }
                Some(current_pc) => current_pc
            };
            if current_frame.is_interpreted() {
                return IRVMExitAction::Exception { throwable: throwable.normal_object.ptr };
            }
            for CompressedExceptionTableElem {
                start_pc,
                end_pc,
                handler_pc,
                catch_type
            } in &code.exception_table {
                let matches_class = match catch_type {
                    None => true,
                    Some(class_name) => {
                        instance_of_exit_impl_impl(jvm, CompressedParsedRefType::Class(*class_name), &throwable.clone().full_object()) == 1
                    }
                };
                if *start_pc <= current_pc && current_pc < *end_pc && matches_class {
                    // int_state.debug_print_stack_trace(jvm);
                    // eprintln!("Unwind to: {}/{}/{}", view.name().unwrap_name().0.to_str(&jvm.string_pool), method_view.name().0.to_str(&jvm.string_pool), method_view.desc().jvm_representation(&jvm.string_pool));
                    let ir_method_id = current_frame.frame_ref().ir_method_id().unwrap();
                    let method_id = current_frame.frame_ref().method_id().unwrap();
                    let handler_address = jvm.java_vm_state.lookup_byte_code_offset(ir_method_id, *handler_pc);
                    let handler_rbp = current_frame.frame_ref().frame_ptr();
                    let frame_size = current_frame.frame_ref().frame_size(&jvm.java_vm_state.ir);
                    let handler_rsp = unsafe { handler_rbp.as_ptr().sub(frame_size) };
                    let loader = LoaderName::BootstrapLoader;//todo
                    let method_resolver = MethodResolverImpl { jvm, loader };
                    let frame_layout = method_resolver.lookup_method_layout(method_id);
                    let to_write_offset = frame_layout.operand_stack_start();
                    unsafe { (handler_rbp.as_ptr().sub(to_write_offset.0) as *mut StackNativeJavaValue).cast::<StackNativeJavaValue>().write(throwable.new_java_value().to_stack_native()); }
                    unsafe { read_frame_ir_header(handler_rbp); }
                    let mut start_diff = SavedRegistersWithIPDiff::no_change();
                    start_diff.saved_registers_without_ip.rbp = Some(handler_rbp.as_ptr() as u64);
                    start_diff.saved_registers_without_ip.rsp = Some(handler_rsp as u64);
                    start_diff.rip = Some(handler_address);
                    return IRVMExitAction::RestartWithRegisterState {
                        diff: start_diff
                    };
                }
            }
        } else {
            return IRVMExitAction::Exception { throwable: throwable.normal_object.ptr };
        }
        this_frame = false;
    }
    jvm.perf_metrics.display();
    todo!()
}

pub fn virtual_args_extract<'gc>(jvm: &'gc JVMState<'gc>, arg_types: &[CompressedParsedDescriptorType], mut arg_start: *const c_void) -> Vec<NewJavaValueHandle<'gc>> {
    let obj_ref_native = unsafe { arg_start.cast::<StackNativeJavaValue>().read() };
    let obj_ref = native_to_new_java_value_rtype(obj_ref_native, CClassName::object().into(), jvm);
    let mut args_jv_handle = vec![];
    args_jv_handle.push(obj_ref);
    unsafe {
        arg_start = arg_start.sub(size_of::<StackNativeJavaValue>());
        for (i, cpdtype) in (0..arg_types.len()).zip(arg_types.iter()) {
            let arg_ptr = arg_start.sub(i * size_of::<jlong>()) as *const u64;
            let native_jv = StackNativeJavaValue { as_u64: arg_ptr.read() };
            args_jv_handle.push(native_to_new_java_value_rtype(native_jv, cpdtype.to_runtime_type().unwrap(), jvm));
        }
    }
    args_jv_handle
}
