use std::collections::HashMap;
use std::ffi::c_void;
use std::hash::Hash;
use std::mem::{size_of, transmute};
use std::num::NonZeroU8;
use std::ptr::{NonNull};
use std::sync::{Arc, Mutex, RwLock};

use itertools::{Itertools};
use libc::{memset};

use another_jit_vm::saved_registers_utils::{SavedRegistersWithIPDiff, SavedRegistersWithoutIPDiff};
use another_jit_vm_ir::{ExitHandlerType, IRInstructIndex, IRMethodID, IRVMExitAction, IRVMExitEvent, IRVMState};
use another_jit_vm_ir::compiler::{IRInstr, RestartPointID};
use another_jit_vm_ir::ir_stack::{FRAME_HEADER_END_OFFSET, IRStackMut};
use another_jit_vm_ir::vm_exit_abi::{InvokeVirtualResolve, IRVMExitType, RuntimeVMExitInput, VMExitTypeWithArgs};
use gc_memory_layout_common::{AllocatedObjectType};
use jvmti_jni_bindings::{jint, jlong};
use rust_jvm_common::{ByteCodeOffset, MethodId};
use rust_jvm_common::compressed_classfile::{CompressedParsedDescriptorType, CompressedParsedRefType, CPDType};
use rust_jvm_common::compressed_classfile::code::{CompressedExceptionTableElem};
use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName};
use rust_jvm_common::cpdtype_table::CPDTypeID;
use rust_jvm_common::method_shape::MethodShape;
use rust_jvm_common::runtime_type::{RuntimeType};

use crate::{check_initing_or_inited_class, InterpreterStateGuard, JavaValue, JString, JVMState, NewJavaValue, WasException};
use crate::class_loading::{assert_inited_or_initing_class};
use crate::inheritance_vtable::{NotCompiledYet, ResolvedInvokeVirtual};
use crate::instructions::fields::get_static_impl;
use crate::instructions::invoke::native::run_native_method;
use crate::instructions::invoke::virtual_::virtual_method_lookup;
use crate::instructions::special::{instance_of_exit_impl, instance_of_exit_impl_impl};
use crate::ir_to_java_layer::compiler::{compile_to_ir, JavaCompilerMethodAndFrameData};
use crate::ir_to_java_layer::instruction_correctness_assertions::AssertionState;
use crate::ir_to_java_layer::java_stack::{OpaqueFrameIdOrMethodID};
use crate::java::lang::class::JClass;
use crate::java::NewAsObjectOrJavaValue;
use crate::java_values::{ByAddressAllocatedObject, default_value, NativeJavaValue};
use crate::jit::{MethodResolver};
use crate::jit::state::{Labeler, runtime_class_to_allocated_object_type};
use crate::new_java_values::{AllocatedObjectHandle, NewJavaValueHandle};
use crate::utils::{lookup_method_parsed};

pub mod compiler;
pub mod java_stack;
pub mod vm_exit_abi;

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub struct ExitNumber(u64);

pub struct JavaVMStateMethod {
    restart_points: HashMap<RestartPointID, IRInstructIndex>,
    ir_index_to_bytecode_pc: HashMap<IRInstructIndex, ByteCodeOffset>,
    bytecode_pc_to_start_ir_index: HashMap<ByteCodeOffset, IRInstructIndex>,
    associated_method_id: MethodId,
}

pub struct JavaVMStateWrapperInner<'gc> {
    most_up_to_date_ir_method_id_for_method_id: HashMap<MethodId, IRMethodID>,
    methods: HashMap<IRMethodID, JavaVMStateMethod>,
    method_exit_handlers: HashMap<ExitNumber, Box<dyn for<'l> Fn(&'gc JVMState<'gc>, &mut InterpreterStateGuard<'l, 'gc>, MethodId, &VMExitTypeWithArgs) -> JavaExitAction>>,
}

impl<'gc> JavaVMStateWrapperInner<'gc> {
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

pub enum VMExitEvent<'vm_life> {
    Allocate { size: usize, return_to: *mut c_void },
    TopLevelExitEvent {
        //todo when this stuff is registers can't have gc.
        _return: JavaValue<'vm_life>
    },
}

impl<'gc> JavaVMStateWrapperInner<'gc> {
    fn handle_vm_exit<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut InterpreterStateGuard<'gc, 'l>, method_id: MethodId, vm_exit_type: &RuntimeVMExitInput, exiting_pc: ByteCodeOffset) -> IRVMExitAction {
        // let current_frame = int_state.current_frame();
        // let (rc, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
        // let view = rc.view();
        // let method_view = view.method_view_i(method_i);
        // let code = method_view.code_attribute().unwrap();
        // drop(current_frame);
        let exit_guard = jvm.perf_metrics.vm_exit_start();
        match vm_exit_type {
            RuntimeVMExitInput::AllocateObjectArray { type_, len, return_to_ptr, res_address } => {
                return Self::allocate_object_array(jvm, int_state, *type_, *len, *return_to_ptr, *res_address);
            }
            RuntimeVMExitInput::LoadClassAndRecompile { .. } => todo!(),
            RuntimeVMExitInput::RunStaticNative { method_id, arg_start, num_args, res_ptr, return_to_ptr } => {
                return Self::run_static_native(jvm, int_state, *method_id, *arg_start, *num_args, *res_ptr, *return_to_ptr);
            }
            RuntimeVMExitInput::RunNativeVirtual { res_ptr, arg_start, method_id, return_to_ptr } => {
                todo!()
            }
            RuntimeVMExitInput::TopLevelReturn { return_value } => {
                return Self::top_level_return(jvm, *return_value);
            }
            RuntimeVMExitInput::CompileFunctionAndRecompileCurrent {
                current_method_id,
                to_recompile,
                restart_point
            } => {
                Self::compile_function_and_recompile_current(jvm, int_state, *current_method_id, *to_recompile, *restart_point)
            }
            RuntimeVMExitInput::PutStatic { field_id, value_ptr, return_to_ptr } => {
                if jvm.exit_trace_options.tracing_enabled() {
                    eprintln!("PutStatic");
                }
                let (rc, field_i) = jvm.field_table.read().unwrap().lookup(*field_id);
                let view = rc.view();
                let field_view = view.field(field_i as usize);
                let mut static_vars_guard = rc.static_vars(jvm);
                let field_name = field_view.field_name();
                let njv = unsafe { (*value_ptr as *mut NativeJavaValue<'gc>).as_ref() }.unwrap().to_new_java_value(&field_view.field_type(), jvm);
                if let NewJavaValue::AllocObject(alloc) = njv.as_njv() {
                    let rc = alloc.runtime_class(jvm);
                    if instance_of_exit_impl(jvm, &field_view.field_type(), Some(alloc)) == 0 {
                        panic!()
                    }
                }
                static_vars_guard.set(field_name, njv);
                drop(exit_guard);
                IRVMExitAction::RestartAtPtr { ptr: *return_to_ptr }
            }
            RuntimeVMExitInput::InitClassAndRecompile { class_type, current_method_id, restart_point, rbp } => {
                if jvm.exit_trace_options.tracing_enabled() {
                    eprintln!("InitClassAndRecompile");
                }
                let cpdtype = jvm.cpdtype_table.read().unwrap().get_cpdtype(*class_type).clone();
                drop(exit_guard);
                let inited = check_initing_or_inited_class(jvm, int_state, cpdtype).unwrap();
                let method_resolver = MethodResolver { jvm, loader: int_state.current_loader(jvm) };
                jvm.java_vm_state.add_method_if_needed(jvm, &method_resolver, *current_method_id);
                let restart_point = jvm.java_vm_state.lookup_restart_point(*current_method_id, *restart_point);
                if jvm.exit_trace_options.tracing_enabled() {
                    eprintln!("InitClassAndRecompile done");
                }
                IRVMExitAction::RestartAtPtr { ptr: restart_point }
            }
            RuntimeVMExitInput::AllocatePrimitiveArray { .. } => todo!(),
            RuntimeVMExitInput::LogFramePointerOffsetValue { value, return_to_ptr } => {
                if jvm.exit_trace_options.tracing_enabled() {
                    eprintln!("value:{}", value);
                }
                IRVMExitAction::RestartAtPtr { ptr: *return_to_ptr }
            }
            RuntimeVMExitInput::LogWholeFrame { return_to_ptr } => {
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
                IRVMExitAction::RestartAtPtr { ptr: *return_to_ptr }
            }
            RuntimeVMExitInput::TraceInstructionBefore { method_id, return_to_ptr, bytecode_offset } => {
                let (rc, method_i) = jvm.method_table.read().unwrap().try_lookup(*method_id).unwrap();
                let view = rc.view();
                let method_view = view.method_view_i(method_i);
                let code = method_view.code_attribute().unwrap();
                let instr = code.instructions.get(bytecode_offset).unwrap();
                eprintln!("Before:{:?} {}", instr.info.better_debug_string(&jvm.string_pool), bytecode_offset.0);
                if jvm.static_breakpoints.should_break(view.name().unwrap_name(), method_view.name(), method_view.desc().clone(), *bytecode_offset) {
                    eprintln!("here");
                }
                if !jvm.instruction_trace_options.partial_tracing() {
                    // jvm.java_vm_state.assertion_state.lock().unwrap().handle_trace_before(jvm, instr, int_state);
                }
                drop(exit_guard);
                IRVMExitAction::RestartAtPtr { ptr: *return_to_ptr }
            }
            RuntimeVMExitInput::TraceInstructionAfter { method_id, return_to_ptr, bytecode_offset } => {
                assert_eq!(Some(*method_id), int_state.current_frame().frame_view.ir_ref.method_id());
                let (rc, method_i) = jvm.method_table.read().unwrap().try_lookup(*method_id).unwrap();
                let view = rc.view();
                let method_view = view.method_view_i(method_i);
                let code = method_view.code_attribute().unwrap();
                let instr = code.instructions.get(bytecode_offset).unwrap();
                eprintln!("After:{}/{:?}", jvm.method_table.read().unwrap().lookup_method_string(*method_id, &jvm.string_pool), instr.info.better_debug_string(&jvm.string_pool));
                if !jvm.instruction_trace_options.partial_tracing() {
                    // jvm.java_vm_state.assertion_state.lock().unwrap().handle_trace_after(jvm, instr, int_state);
                }
                dump_frame_contents(jvm, int_state);
                drop(exit_guard);
                IRVMExitAction::RestartAtPtr { ptr: *return_to_ptr }
            }
            RuntimeVMExitInput::NPE { .. } => {
                int_state.debug_print_stack_trace(jvm);
                todo!()
            }
            RuntimeVMExitInput::AllocateObject { type_, return_to_ptr, res_address } => {
                let guard = jvm.perf_metrics.vm_exit_allocate_obj();
                if jvm.exit_trace_options.tracing_enabled() {
                    eprintln!("AllocateObject");
                }
                let type_ = jvm.cpdtype_table.read().unwrap().get_cpdtype(*type_).unwrap_ref_type().clone();
                let rc = assert_inited_or_initing_class(jvm, CPDType::Ref(type_.clone()));
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
                IRVMExitAction::RestartAtPtr { ptr: *return_to_ptr }
            }
            RuntimeVMExitInput::NewString { return_to_ptr, res, compressed_wtf8 } => {
                if jvm.exit_trace_options.tracing_enabled() {
                    eprintln!("NewString");
                }
                let wtf8buf = compressed_wtf8.to_wtf8(&jvm.wtf8_pool);
                drop(exit_guard);
                let jstring = JString::from_rust(jvm, int_state, wtf8buf).expect("todo exceptions").intern(jvm, int_state).unwrap();
                let jv = jstring.new_java_value_handle();
                unsafe {
                    let raw_64 = jv.as_njv().to_native().as_u64;
                    (*res as *mut u64).write(raw_64);
                }
                IRVMExitAction::RestartAtPtr { ptr: *return_to_ptr }
            }
            RuntimeVMExitInput::NewClass { type_, res, return_to_ptr } => {
                if jvm.exit_trace_options.tracing_enabled() {
                    eprintln!("NewClass");
                }
                let cpdtype = jvm.cpdtype_table.write().unwrap().get_cpdtype(*type_).clone();
                let jclass = JClass::from_type(jvm, int_state, cpdtype).unwrap();
                let jv_new_handle = jclass.new_java_value_handle();
                unsafe {
                    let raw_64 = jv_new_handle.as_njv().to_native().as_u64;
                    (*res as *mut u64).write(raw_64);
                };
                drop(exit_guard);
                IRVMExitAction::RestartAtPtr { ptr: *return_to_ptr }
            }
            RuntimeVMExitInput::InvokeVirtualResolve { return_to_ptr, object_ref_ptr, method_shape_id, native_method_restart_point, native_method_res } => {
                if jvm.exit_trace_options.tracing_enabled() {
                    eprintln!("InvokeVirtualResolve");
                }
                let caller_method_id = int_state.current_frame().frame_view.ir_ref.method_id().unwrap();
                let MethodShape { name, desc } = jvm.method_shapes.lookup_method_shape(*method_shape_id);
                //todo this is probably wrong what if there's a class with a same name private method?
                // like surely I need to start at the classname specified in the bytecode
                let mut memory_region_guard = jvm.gc.memory_region.lock().unwrap();
                let allocated_type = memory_region_guard.find_object_allocated_type(NonNull::new(unsafe { (*object_ref_ptr as *const *mut c_void).read() }).unwrap()).clone();
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
                        let arg_start: *const c_void = *object_ref_ptr;
                        let args_jv_handle = Self::virtual_args_extract(jvm, arg_types, arg_start);
                        let args_new_jv = args_jv_handle.iter().map(|handle| handle.as_njv()).collect();
                        drop(exit_guard);
                        let res = match run_native_method(jvm, int_state, rc, method_i, args_new_jv) {
                            Ok(x) => x,
                            Err(WasException {}) => {
                                let allocate_obj = int_state.throw().unwrap().handle.duplicate_discouraged();
                                int_state.set_throw(None);
                                dbg!(allocate_obj.cast_throwable().to_string(jvm, int_state).unwrap().unwrap().to_rust_string(jvm));
                                todo!()
                            }
                        };
                        if let Some(res) = res {
                            unsafe { ((*native_method_res) as *mut NativeJavaValue).write(res.as_njv().to_native()) }
                        };
                        if !jvm.instruction_trace_options.partial_tracing() {
                            /*jvm.java_vm_state.assertion_state.lock().unwrap().current_before.pop().unwrap();*/
                        }
                        let restart_address = jvm.java_vm_state.lookup_restart_point(caller_method_id, *native_method_restart_point);
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
                        rip: Some(*return_to_ptr),
                        saved_registers_without_ip: start_diff,
                    }
                }
            }
            RuntimeVMExitInput::MonitorEnter { obj_ptr, return_to_ptr } => {
                if jvm.exit_trace_options.tracing_enabled() {
                    eprintln!("MonitorEnter");
                }
                let monitor = jvm.monitor_for(*obj_ptr);
                monitor.lock(jvm, int_state).unwrap();
                drop(exit_guard);
                IRVMExitAction::RestartAtPtr { ptr: *return_to_ptr }
            }
            RuntimeVMExitInput::MonitorExit { obj_ptr, return_to_ptr } => {
                if jvm.exit_trace_options.tracing_enabled() {
                    eprintln!("MonitorExit");
                }
                let monitor = jvm.monitor_for(*obj_ptr);
                monitor.unlock(jvm, int_state).unwrap();
                drop(exit_guard);
                IRVMExitAction::RestartAtPtr { ptr: *return_to_ptr }
            }
            RuntimeVMExitInput::GetStatic { res_value_ptr: value_ptr, field_name, cpdtype_id, return_to_ptr } => {
                let get_static = jvm.perf_metrics.vm_exit_get_static();
                if jvm.exit_trace_options.tracing_enabled() {
                    eprintln!("GetStatic");
                }
                let cpd_type = jvm.cpdtype_table.read().unwrap().get_cpdtype(*cpdtype_id).clone();
                let name = cpd_type.unwrap_class_type();
                let static_var = get_static_impl(jvm, int_state, name, *field_name).unwrap().unwrap();
                // let static_var = static_vars_guard.get(field_name);
                // todo doesn't handle interfaces and the like
                // int_state.debug_print_stack_trace(jvm);
                unsafe { (*value_ptr).cast::<NativeJavaValue>().write(static_var.as_njv().to_native()); }
                drop(exit_guard);
                drop(get_static);
                IRVMExitAction::RestartAtPtr { ptr: *return_to_ptr }
            }

            RuntimeVMExitInput::InstanceOf { res, value, cpdtype_id, return_to_ptr } => {
                if jvm.exit_trace_options.tracing_enabled() {
                    eprintln!("InstanceOf");
                }
                let cpdtype = *jvm.cpdtype_table.read().unwrap().get_cpdtype(*cpdtype_id);
                let value = unsafe { (*value).cast::<NativeJavaValue>().read() };
                let value = value.to_new_java_value(&CClassName::object().into(), jvm);
                let value = value.unwrap_object();
                check_initing_or_inited_class(jvm, int_state, cpdtype).unwrap();
                let res_int = instance_of_exit_impl(jvm, &cpdtype, value.as_ref().map(|handle| handle.as_allocated_obj()));
                unsafe { (*((*res) as *mut NativeJavaValue)).int = res_int };
                drop(exit_guard);
                IRVMExitAction::RestartAtPtr { ptr: *return_to_ptr }
            }
            RuntimeVMExitInput::CheckCast { value, cpdtype_id, return_to_ptr } => {
                let checkcast = jvm.perf_metrics.vm_exit_checkcast();
                if jvm.exit_trace_options.tracing_enabled() {
                    eprintln!("CheckCast");
                }
                let cpdtype = jvm.cpdtype_table.read().unwrap().get_cpdtype(*cpdtype_id).clone();
                //todo just use region data from pointer to cache the result of this checkast and then havee a restart point
                /*runtime_class_to_allocated_object_type(&rc, LoaderName::BootstrapLoader, todo!());
                todo!();*/
                let value = unsafe { (*value).cast::<NativeJavaValue>().read() };
                let value = value.to_new_java_value(&CClassName::object().into(), jvm);
                let value = value.unwrap_object();
                if let Some(handle) = value {
                    let res_int = instance_of_exit_impl(jvm, &cpdtype, Some(handle.as_allocated_obj()));
                    if res_int == 0 {
                        dbg!(cpdtype.jvm_representation(&jvm.string_pool));
                        dbg!(handle.as_allocated_obj().runtime_class(jvm).cpdtype().jvm_representation(&jvm.string_pool));
                        int_state.debug_print_stack_trace(jvm);
                        todo!()
                    }

                    let base_address_and_mask = jvm.gc.memory_region.lock().unwrap().find_object_base_address_and_mask(handle.ptr);
                    jvm.known_addresses.sink_known_address(cpdtype, base_address_and_mask)
                }
                drop(exit_guard);
                drop(checkcast);
                IRVMExitAction::RestartAtPtr { ptr: *return_to_ptr }
            }
            RuntimeVMExitInput::RunNativeSpecial { res_ptr, arg_start, method_id, return_to_ptr } => {
                if jvm.exit_trace_options.tracing_enabled() {
                    eprintln!("RunNativeSpecial");
                }
                let (rc, method_i) = jvm.method_table.read().unwrap().try_lookup(*method_id).unwrap();
                let class_view = rc.view();
                let method_view = class_view.method_view_i(method_i);
                let arg_types = &method_view.desc().arg_types;
                let arg_start: *const c_void = *arg_start;
                let args_jv_handle = Self::virtual_args_extract(jvm, arg_types, arg_start);
                let args_new_jv: Vec<NewJavaValue> = args_jv_handle.iter().map(|handle| handle.as_njv()).collect();
                args_new_jv[0].unwrap_object_alloc().unwrap();//nonnull this
                drop(exit_guard);
                let res = match run_native_method(jvm, int_state, rc, method_i, args_new_jv) {
                    Ok(x) => x,
                    Err(WasException {}) => {
                        let expception_obj_handle = int_state.throw().unwrap().handle.duplicate_discouraged();
                        int_state.set_throw(None);
                        return Self::throw_impl(jvm, int_state, NewJavaValueHandle::Object(expception_obj_handle));
                    }
                };
                if let Some(res) = res {
                    unsafe { ((*res_ptr) as *mut NativeJavaValue).write(res.as_njv().to_native()) }
                };
                if !jvm.instruction_trace_options.partial_tracing() {
                    // jvm.java_vm_state.assertion_state.lock().unwrap().current_before.pop().unwrap();
                }
                IRVMExitAction::RestartAtPtr { ptr: *return_to_ptr }
            }
            RuntimeVMExitInput::InvokeInterfaceResolve { return_to_ptr, native_method_restart_point, native_method_res, object_ref, target_method_id } => {
                if jvm.exit_trace_options.tracing_enabled() {
                    eprintln!("InvokeInterfaceResolve");
                }
                let caller_method_id = int_state.current_frame().frame_view.ir_ref.method_id().unwrap();
                let obj_jv_handle = unsafe { (*object_ref).cast::<NativeJavaValue>().read() }.to_new_java_value(&CPDType::object(), jvm);
                let obj_rc = obj_jv_handle.unwrap_object_nonnull().as_allocated_obj().runtime_class(jvm);
                let (target_rc, target_method_i) = jvm.method_table.read().unwrap().try_lookup(*target_method_id).unwrap();
                let class_view = target_rc.view();
                let method_view = class_view.method_view_i(target_method_i);
                let method_name = method_view.name();
                let method_desc = method_view.desc();
                let (resolved_method_i, resolved_rc) = lookup_method_parsed(jvm, obj_rc, method_name, method_desc).unwrap();
                let resolved_method_id = jvm.method_table.write().unwrap().get_method_id(resolved_rc.clone(), resolved_method_i);
                if jvm.is_native_by_method_id(resolved_method_id) {
                    let args_jv_handle = Self::virtual_args_extract(jvm, method_desc.arg_types.as_slice(), *object_ref);
                    match run_native_method(jvm, int_state, resolved_rc, resolved_method_i, args_jv_handle.iter().map(|handle| handle.as_njv()).collect_vec()) {
                        Ok(res) => {
                            if let Some(res) = res {
                                unsafe { ((*native_method_res) as *mut NativeJavaValue).write(res.as_njv().to_native()) }
                            };
                            if !jvm.instruction_trace_options.partial_tracing() {
                                // jvm.java_vm_state.assertion_state.lock().unwrap().current_before.pop().unwrap();
                            }
                            let restart_address = jvm.java_vm_state.lookup_restart_point(caller_method_id, *native_method_restart_point);
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
                            rip: Some(*return_to_ptr),
                            saved_registers_without_ip: start_diff,
                        }
                    }
                }
            }
            RuntimeVMExitInput::Throw { exception_obj_ptr } => {
                let throw = jvm.perf_metrics.vm_exit_throw();
                if jvm.exit_trace_options.tracing_enabled() {
                    eprintln!("Throw");
                }
                eprintln!("THROW AT:");
                int_state.debug_print_stack_trace(jvm);
                let exception_obj_native_value = unsafe { (*exception_obj_ptr).cast::<NativeJavaValue<'gc>>().read() };
                let exception_obj_handle = exception_obj_native_value.to_new_java_value(&CClassName::object().into(), jvm);
                return Self::throw_impl(&jvm, int_state, exception_obj_handle);
            }
            RuntimeVMExitInput::MultiAllocateArray {
                elem_type,
                num_arrays,
                len_start,
                return_to_ptr,
                res_address
            } => {
                let elem_type = *jvm.cpdtype_table.read().unwrap().get_cpdtype(*elem_type);
                let elem_type = elem_type.to_non_array();
                let array_type = CPDType::Ref(CompressedParsedRefType::Array { base_type: elem_type, num_nested_arrs: NonZeroU8::new(*num_arrays).unwrap() });
                let mut lens = vec![];
                unsafe {
                    for len_index in 0..*num_arrays {
                        let offsetted_ptr = len_start.sub(len_index as usize);
                        lens.push(offsetted_ptr.cast::<i32>().read());
                    }
                }
                let mut current_value = NativeJavaValue { as_u64: u64::MAX };
                assert_inited_or_initing_class(jvm, elem_type.to_cpdtype());
                current_value = default_value(&elem_type.to_cpdtype()).as_njv().to_native();
                //iterates from innermost to outermost
                for (depth, len) in lens.into_iter().rev().enumerate() {
                    let rc = match NonZeroU8::new(depth as u8 + 1) {
                        None => {
                            panic!()
                        }
                        Some(depth) => {
                            assert_inited_or_initing_class(jvm, CPDType::Ref(CompressedParsedRefType::Array { base_type: elem_type, num_nested_arrs: depth }))
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
                IRVMExitAction::RestartAtPtr { ptr: *return_to_ptr }
            }
        }
    }

    fn compile_function_and_recompile_current(jvm: &'gc JVMState<'gc>, int_state: &mut InterpreterStateGuard<'gc, '_>, current_method_id: MethodId, to_recompile: MethodId, restart_point: RestartPointID) -> IRVMExitAction {
        if jvm.exit_trace_options.tracing_enabled() {
            eprintln!("CompileFunctionAndRecompileCurrent");
        }
        let method_resolver = MethodResolver { jvm, loader: int_state.current_loader(jvm) };
        jvm.java_vm_state.add_method_if_needed(jvm, &method_resolver, to_recompile);
        jvm.java_vm_state.add_method_if_needed(jvm, &method_resolver, current_method_id);
        let restart_point = jvm.java_vm_state.lookup_restart_point(current_method_id, restart_point);
        IRVMExitAction::RestartAtPtr { ptr: restart_point }
    }

    fn top_level_return(jvm: &JVMState, return_value: u64) -> IRVMExitAction {
        if jvm.exit_trace_options.tracing_enabled() {
            eprintln!("TopLevelReturn");
        }
        IRVMExitAction::ExitVMCompletely { return_data: return_value }
    }

    fn run_static_native(jvm: &'gc JVMState<'gc>, int_state: &mut InterpreterStateGuard<'gc, '_>, method_id: MethodId, arg_start: *mut c_void, num_args: u16, res_ptr: *mut c_void, return_to_ptr: *mut c_void) -> IRVMExitAction {
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
                args_jv_handle.push(native_jv.to_new_java_value(cpdtype, jvm))
            }
        }
        assert!(jvm.thread_state.int_state_guard_valid.get().borrow().clone());
        let args_new_jv = args_jv_handle.iter().map(|handle| handle.as_njv()).collect();
        let res = match run_native_method(jvm, int_state, rc, method_i, args_new_jv) {
            Ok(x) => x,
            Err(WasException {}) => {
                let expception_obj_handle = int_state.throw().unwrap().handle.duplicate_discouraged();
                int_state.set_throw(None);
                return Self::throw_impl(jvm, int_state, NewJavaValueHandle::Object(expception_obj_handle));
            }
        };
        assert!(int_state.throw().is_none());
        if let Some(res) = res {
            unsafe { (res_ptr as *mut NativeJavaValue<'static>).write(transmute::<NativeJavaValue<'_>, NativeJavaValue<'static>>(res.as_njv().to_native())) }
        };
        if !jvm.instruction_trace_options.partial_tracing() {
            // jvm.java_vm_state.assertion_state.lock().unwrap().current_before.pop().unwrap();
        }
        IRVMExitAction::RestartAtPtr { ptr: return_to_ptr }
    }

    fn allocate_object_array(jvm: &'gc JVMState<'gc>, int_state: &mut InterpreterStateGuard<'gc, '_>, type_: CPDTypeID, len: i32, return_to_ptr: *const c_void, res_address: *mut NonNull<c_void>) -> IRVMExitAction {
        if jvm.exit_trace_options.tracing_enabled() {
            eprintln!("AllocateObjectArray");
        }
        let type_ = jvm.cpdtype_table.read().unwrap().get_cpdtype(type_).unwrap_ref_type().clone();
        assert!(len >= 0);
        let rc = assert_inited_or_initing_class(jvm, CPDType::Ref(type_.clone()));
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

    fn throw_impl(jvm: &'gc JVMState<'gc>, int_state: &mut InterpreterStateGuard<'gc, '_>, exception_obj_handle: NewJavaValueHandle<'gc>) -> IRVMExitAction {
        let exception_object_handle = exception_obj_handle.unwrap_object_nonnull();
        let throwable = exception_object_handle.cast_throwable();
        let exception_as_string = throwable.to_string(jvm, int_state).unwrap().unwrap();
        dbg!(exception_as_string.to_rust_string(jvm));
        let exception_object_handle = throwable.normal_object;
        let exception_obj = exception_object_handle.as_allocated_obj();
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
                            instance_of_exit_impl_impl(jvm, CompressedParsedRefType::Class(*class_name), exception_obj.clone()) == 1
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

    fn virtual_args_extract(jvm: &'gc JVMState<'gc>, arg_types: &[CompressedParsedDescriptorType], mut arg_start: *const c_void) -> Vec<NewJavaValueHandle<'gc>> {
        let obj_ref_native = unsafe { arg_start.cast::<NativeJavaValue>().read() };
        let obj_ref = obj_ref_native.to_new_java_value(&CClassName::object().into(), jvm);
        let mut args_jv_handle = vec![];
        args_jv_handle.push(obj_ref);
        unsafe {
            arg_start = arg_start.sub(size_of::<NativeJavaValue>());
            for (i, cpdtype) in (0..arg_types.len()).zip(arg_types.iter()) {
                let arg_ptr = arg_start.sub(i * size_of::<jlong>()) as *const u64;
                let native_jv = NativeJavaValue { as_u64: arg_ptr.read() };
                args_jv_handle.push(native_jv.to_new_java_value(cpdtype, jvm))
            }
        }
        args_jv_handle
    }
}

pub fn dump_frame_contents<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &mut InterpreterStateGuard<'gc, 'l>) {
    unsafe {
        if !IN_TO_STRING {
            dump_frame_contents_impl(jvm, int_state)
        }
    }
}

pub fn dump_frame_contents_impl<'gc>(jvm: &'gc JVMState<'gc>, int_state: &mut InterpreterStateGuard<'gc, '_>) {
    if !int_state.current_frame().full_frame_available(jvm) {
        let current_frame = int_state.current_frame();
        let local_vars = current_frame.local_var_simplified_types(jvm);
        eprint!("Local Vars:");
        unsafe {
            for (i, local_var_type) in local_vars.into_iter().enumerate() {
                let jv = current_frame.local_vars(jvm).raw_get(i as u16);
                eprint!("#{}: {:?}\t", i, jv as *const c_void)
            }
        }
        eprintln!();
        eprint!("Operand Stack:");
        let operand_stack_ref = current_frame.operand_stack(jvm);
        let operand_stack_types = operand_stack_ref.simplified_types();
        unsafe {
            for (i, operand_stack_type) in operand_stack_types.into_iter().enumerate() {
                let jv = operand_stack_ref.raw_get(i as u16);
                eprint!("#{}: {:?}\t", i, jv.object)
            }
        }
        eprintln!();
        return;
    }
    let local_var_types = int_state.current_frame().local_var_types(jvm);
    eprint!("Local Vars:");
    unsafe {
        for (i, local_var_type) in local_var_types.into_iter().enumerate() {
            match local_var_type.to_runtime_type() {
                RuntimeType::TopType => {
                    let jv = int_state.current_frame().local_vars(jvm).raw_get(i as u16);
                    eprint!("#{}: Top: {:?}\t", i, jv as *const c_void)
                }
                _ => {
                    let jv = int_state.current_frame().local_vars(jvm).get(i as u16, local_var_type.to_runtime_type());
                    if let Some(Some(obj)) = jv.try_unwrap_object_alloc() {
                        display_obj(jvm, int_state, i, obj);
                    } else {
                        let jv = int_state.current_frame().local_vars(jvm).get(i as u16, local_var_type.to_runtime_type());
                        eprint!("#{}: {:?}\t", i, jv.as_njv())
                    }
                }
            }
        }
    }
    eprintln!();
    let operand_stack_types = int_state.current_frame().operand_stack(jvm).types();
    // current_frame.ir_stack_entry_debug_print();
    eprint!("Operand Stack:");
    for (i, operand_stack_type) in operand_stack_types.into_iter().enumerate() {
        if let RuntimeType::TopType = operand_stack_type {
            panic!()
            /*let jv = operand_stack.raw_get(i as u16);
            eprint!("#{}: Top: {:?}\t", i, jv.object)*/
        } else {
            let jv = int_state.current_frame().operand_stack(jvm).get(i as u16, operand_stack_type.clone());
            if let Some(Some(obj)) = jv.try_unwrap_object_alloc() {
                display_obj(jvm, int_state, i, obj)
            } else {
                let jv = int_state.current_frame().operand_stack(jvm).get(i as u16, operand_stack_type);
                eprint!("#{}: {:?}\t", i, jv.as_njv())
            }
        }
    }
    eprintln!()
}

static mut IN_TO_STRING: bool = false;

fn display_obj<'gc>(jvm: &'gc JVMState<'gc>, int_state: &mut InterpreterStateGuard<'gc, '_>, i: usize, obj: AllocatedObjectHandle<'gc>) {
    let obj_type = obj.as_allocated_obj().runtime_class(jvm).cpdtype();
    unsafe {
        if obj_type == CClassName::string().into() {
            let ptr = obj.ptr;
            let string = obj.cast_string();
            eprint!("#{}: {:?}(String:{:?})\t", i, ptr, string.to_rust_string_better(jvm).unwrap_or("malformed_string".to_string()))
        } else if obj_type == CClassName::class().into() {
            let class_short_name = match jvm.classes.read().unwrap().class_object_pool.get_by_left(&ByAddressAllocatedObject::LookupOnly(obj.as_allocated_obj().raw_ptr_usize())) {
                Some(class) => {
                    Some(class.cpdtype().jvm_representation(&jvm.string_pool))
                }
                None => None,
            };
            let ptr = obj.ptr;
            let ref_data = obj.as_allocated_obj().get_var_top_level(jvm, FieldName::field_reflectionData());
            eprint!("#{}: {:?}(Class:{:?} {:?})\t", i, ptr, class_short_name, ref_data.as_njv().to_native().object)
        } else {
            let ptr = obj.ptr;
            let save = IN_TO_STRING;
            IN_TO_STRING = true;
            eprint!("#{}: {:?}({})({})\t", i, ptr, obj_type.short_representation(&jvm.string_pool), obj.cast_object().to_string(jvm, int_state).unwrap().unwrap().to_rust_string(jvm));
            IN_TO_STRING = save;
        }
    }
}

pub struct JavaVMStateWrapper<'vm_life> {
    pub ir: IRVMState<'vm_life, ()>,
    pub inner: RwLock<JavaVMStateWrapperInner<'vm_life>>,
    // should be per thread
    labeler: Labeler,
    pub(crate) assertion_state: Mutex<AssertionState<'vm_life>>,
}

impl<'vm_life> JavaVMStateWrapper<'vm_life> {
    pub fn new() -> Self {
        Self {
            ir: IRVMState::new(),
            inner: RwLock::new(JavaVMStateWrapperInner {
                most_up_to_date_ir_method_id_for_method_id: Default::default(),
                methods: Default::default(),
                method_exit_handlers: Default::default(),
            }),
            labeler: Labeler::new(),
            assertion_state: Mutex::new(AssertionState { current_before: vec![] }),
        }
    }

    pub fn add_top_level_vm_exit(&'vm_life self) {
        //&IRVMExitEvent, IRStackMut, &IRVMState<'vm_life, ExtraData>, &mut ExtraData
        let (ir_method_id, restart_points) = self.ir.add_function(vec![IRInstr::VMExit2 { exit_type: IRVMExitType::TopLevelReturn {} }], FRAME_HEADER_END_OFFSET, Arc::new(|event, ir_stack_mut, ir_vm_state, extra| {
            match &event.exit_type {
                RuntimeVMExitInput::TopLevelReturn { return_value } => IRVMExitAction::ExitVMCompletely { return_data: *return_value },
                _ => panic!()
            }
        }));
        assert!(restart_points.is_empty());
        self.ir.init_top_level_exit_id(ir_method_id)
    }

    pub fn run_method<'l>(&'vm_life self, jvm: &'vm_life JVMState<'vm_life>, int_state: &'_ mut InterpreterStateGuard<'vm_life, 'l>, method_id: MethodId) -> u64 {
        let (rc, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
        let view = rc.view();
        let method_view = view.method_view_i(method_i);
        let method_name = method_view.name().0.to_str(&jvm.string_pool);
        let class_name = view.name().unwrap_name().0.to_str(&jvm.string_pool);
        let desc_str = method_view.desc_str().to_str(&jvm.string_pool);
        // eprintln!("ENTER RUN METHOD: {} {} {}", &class_name, &method_name, &desc_str);
        let ir_method_id = *self.inner.read().unwrap().most_up_to_date_ir_method_id_for_method_id.get(&method_id).unwrap();
        let current_frame_pointer = int_state.current_frame().frame_view.ir_ref.frame_ptr();
        let assert_data = int_state.frame_state_assert_save_from(current_frame_pointer);
        let mut frame_to_run_on = int_state.current_frame_mut();
        let frame_ir_method_id = frame_to_run_on.frame_view.ir_mut.downgrade().ir_method_id().unwrap();
        assert_eq!(self.inner.read().unwrap().associated_method_id(ir_method_id), method_id);
        if frame_ir_method_id != ir_method_id {
            frame_to_run_on.frame_view.ir_mut.set_ir_method_id(ir_method_id);
        }
        assert!(jvm.thread_state.int_state_guard_valid.get().borrow().clone());
        let res = self.ir.run_method(ir_method_id, &mut frame_to_run_on.frame_view.ir_mut, &mut ());
        int_state.saved_assert_frame_from(assert_data, current_frame_pointer);
        // eprintln!("EXIT RUN METHOD: {} {} {}", &class_name, &method_name, &desc_str);
        res
    }

    pub fn lookup_ir_method_id(&self, opaque_or_not: OpaqueFrameIdOrMethodID) -> IRMethodID {
        self.try_lookup_ir_method_id(opaque_or_not).unwrap()
    }

    pub fn lookup_resolved_invoke_virtual(&self, method_id: MethodId, resolver: &MethodResolver) -> Result<ResolvedInvokeVirtual, NotCompiledYet> {
        let ir_method_id = self.lookup_method_ir_method_id(method_id);
        let address = self.ir.lookup_ir_method_id_pointer(ir_method_id);

        let new_frame_size = resolver.lookup_method_layout(method_id).full_frame_size();
        Ok(ResolvedInvokeVirtual {
            address,
            ir_method_id,
            method_id,
            new_frame_size,
        })
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
        let ir_instruct_index = read_guard.restart_location(ir_method_id, restart_point_id);
        drop(read_guard);
        self.ir.lookup_location_of_ir_instruct(ir_method_id, ir_instruct_index).0
    }

    pub fn lookup_ip(&self, ip: *const c_void) -> Option<(MethodId, ByteCodeOffset)> {
        let (ir_method_id, ir_instruct_index) = self.ir.lookup_ip(ip);
        if ir_method_id == self.ir.get_top_level_return_ir_method_id() {
            return None;
        }
        let guard = self.inner.read().unwrap();
        let method = guard.methods.get(&ir_method_id).unwrap();
        let method_id = method.associated_method_id;
        let pc = *method.ir_index_to_bytecode_pc.get(&ir_instruct_index).unwrap();
        Some((method_id, pc))
    }

    pub fn lookup_byte_code_offset(&self, ir_method_id: IRMethodID, java_pc: ByteCodeOffset) -> *const c_void {
        let read_guard = self.inner.read().unwrap();
        let ir_instruct_index = *read_guard.methods.get(&ir_method_id).unwrap().bytecode_pc_to_start_ir_index.get(&java_pc).unwrap();
        self.ir.lookup_location_of_ir_instruct(ir_method_id, ir_instruct_index).0
    }
}


impl<'vm_life> JavaVMStateWrapper<'vm_life> {
    pub fn add_method_if_needed(&'vm_life self, jvm: &'vm_life JVMState<'vm_life>, resolver: &MethodResolver<'vm_life>, method_id: MethodId) {
        let compile_guard = jvm.perf_metrics.compilation_start();
        if jvm.recompilation_conditions.read().unwrap().should_recompile(method_id, resolver) {
            let mut recompilation_guard = jvm.recompilation_conditions.write().unwrap();
            let mut recompile_conditions = recompilation_guard.recompile_conditions(method_id);
            eprintln!("Re/Compile: {}", jvm.method_table.read().unwrap().lookup_method_string(method_id, &jvm.string_pool));
            //todo need some mechanism for detecting recompile necessary
            //todo unify resolver and recompile_conditions
            let is_native = jvm.is_native_by_method_id(method_id);
            assert!(!is_native);
            let mut java_function_frame_guard = jvm.java_function_frame_data.write().unwrap();
            let java_frame_data = &java_function_frame_guard.entry(method_id).or_insert_with(|| JavaCompilerMethodAndFrameData::new(jvm, method_id));
            let ir_instructions_and_offsets = compile_to_ir(resolver, &self.labeler, java_frame_data, &mut recompile_conditions);
            let ir_exit_handler: ExitHandlerType<'vm_life, ()> = Arc::new(move |ir_vm_exit_event: &IRVMExitEvent, ir_stack_mut: IRStackMut, ir_vm_state: &IRVMState<'vm_life, ()>, extra| {
                let ir_stack_mut: IRStackMut = ir_stack_mut;
                let frame_ptr = ir_vm_exit_event.inner.saved_guest_registers.saved_registers_without_ip.rbp;
                let ir_num = ExitNumber(ir_vm_exit_event.inner.saved_guest_registers.saved_registers_without_ip.rax as u64);
                let read_guard = self.inner.read().unwrap();
                let ir_method_id = ir_vm_exit_event.ir_method;
                let method = read_guard.methods.get(&ir_method_id).unwrap();
                let method_id = method.associated_method_id;
                let exiting_pc = *method.ir_index_to_bytecode_pc.get(&ir_vm_exit_event.exit_ir_instr).unwrap();
                drop(read_guard);
                let mmaped_top = ir_stack_mut.owned_ir_stack.native.mmaped_top;

                let mut int_state = InterpreterStateGuard::LocalInterpreterState {
                    int_state: ir_stack_mut,
                    thread: jvm.thread_state.get_current_thread(),
                    registered: false,
                    jvm,
                    current_exited_pc: Some(exiting_pc),
                    throw: None,
                };
                let old_intstate = int_state.register_interpreter_state_guard(jvm);
                unsafe {
                    let exiting_frame_position_rbp = ir_vm_exit_event.inner.saved_guest_registers.saved_registers_without_ip.rbp;
                    let exiting_stack_pointer = ir_vm_exit_event.inner.saved_guest_registers.saved_registers_without_ip.rsp;
                    if exiting_stack_pointer != mmaped_top {
                        let offset = exiting_frame_position_rbp.offset_from(exiting_stack_pointer).abs() as usize /*+ size_of::<u64>()*/;
                        let frame_ref = int_state.current_frame().frame_view.ir_ref;
                        let expected_current_frame_size = frame_ref.frame_size(&jvm.java_vm_state.ir);
                        assert_eq!(offset, expected_current_frame_size);
                    }
                }
                let res = JavaVMStateWrapperInner::handle_vm_exit(jvm, &mut int_state, method_id, &ir_vm_exit_event.exit_type, exiting_pc);
                int_state.deregister_int_state(jvm, old_intstate);
                res
            });
            let mut ir_instructions = vec![];
            let mut ir_index_to_bytecode_pc = HashMap::new();
            let mut bytecode_pc_to_start_ir_index = HashMap::new();
            //todo consider making this use iterators and stuff.
            for (i, (offset, ir_instr)) in ir_instructions_and_offsets.into_iter().enumerate() {
                let current_ir_index = IRInstructIndex(i);
                let prev_value = ir_index_to_bytecode_pc.insert(current_ir_index, offset);
                assert!(prev_value.is_none());
                let prev_value = bytecode_pc_to_start_ir_index.insert(offset, current_ir_index);
                match prev_value {
                    None => {}
                    Some(prev_index) => {
                        if prev_index < current_ir_index {
                            bytecode_pc_to_start_ir_index.insert(offset, prev_index);
                        }
                    }
                }
                ir_instructions.push(ir_instr);
            }
            let (ir_method_id, restart_points) = self.ir.add_function(ir_instructions, java_frame_data.full_frame_size(), ir_exit_handler);
            let mut write_guard = self.inner.write().unwrap();
            write_guard.most_up_to_date_ir_method_id_for_method_id.insert(method_id, ir_method_id);
            write_guard.methods.insert(ir_method_id, JavaVMStateMethod {
                restart_points,
                ir_index_to_bytecode_pc,
                bytecode_pc_to_start_ir_index,
                associated_method_id: method_id,
            });
            /*        jvm.vtables.write().unwrap().notify_compile_or_recompile(jvm, method_id, ResolvedInvokeVirtual {
                    address: self.ir.lookup_ir_method_id_pointer(ir_method_id),
                    ir_method_id,
                    method_id,
                    new_frame_size: java_frame_data.full_frame_size(),
                })*/
            drop(write_guard);
        }
    }
}


pub mod instruction_correctness_assertions;