use std::collections::HashMap;
use std::ffi::c_void;
use std::hash::Hash;
use std::marker::PhantomData;
use std::mem::{size_of, transmute};
use std::ops::Deref;
use std::ptr::{NonNull, null_mut};
use std::sync::{Arc, RwLock};
use std::thread::current;

use bimap::BiHashMap;
use iced_x86::CC_b::c;
use iced_x86::CC_g::g;
use iced_x86::CC_np::po;
use iced_x86::ConditionCode::{o, s};
use iced_x86::CpuidFeature::UDBG;
use iced_x86::OpCodeOperandKind::al;
use itertools::Itertools;
use libc::{memcpy, memset, read};

use another_jit_vm::{Method, VMExitAction};
use another_jit_vm::saved_registers_utils::{SavedRegistersWithIPDiff, SavedRegistersWithoutIPDiff};
use another_jit_vm_ir::{ExitHandlerType, IRInstructIndex, IRMethodID, IRVMExitAction, IRVMExitEvent, IRVMState};
use another_jit_vm_ir::compiler::{IRInstr, RestartPointID};
use another_jit_vm_ir::ir_stack::{FRAME_HEADER_END_OFFSET, IRStackMut};
use another_jit_vm_ir::vm_exit_abi::{InvokeVirtualResolve, IRVMExitType, RuntimeVMExitInput, VMExitTypeWithArgs};
use gc_memory_layout_common::AllocatedObjectType;
use jvmti_jni_bindings::{jint, jlong};
use rust_jvm_common::{ByteCodeOffset, MethodId};
use rust_jvm_common::compressed_classfile::code::{CompressedCode, CompressedInstruction, CompressedInstructionInfo};
use rust_jvm_common::compressed_classfile::CPDType;
use rust_jvm_common::compressed_classfile::names::CClassName;
use rust_jvm_common::loading::LoaderName;
use rust_jvm_common::runtime_type::{RuntimeRefType, RuntimeType};
use rust_jvm_common::vtype::VType;

use crate::{check_initing_or_inited_class, check_loaded_class_force_loader, InterpreterStateGuard, JavaValue, JString, JVMState};
use crate::class_loading::assert_inited_or_initing_class;
use crate::inheritance_vtable::{NotCompiledYet, ResolvedInvokeVirtual};
use crate::instructions::invoke::native::mhn_temp::init::init;
use crate::instructions::invoke::native::run_native_method;
use crate::interpreter::FrameToRunOn;
use crate::interpreter_state::FramePushGuard;
use crate::ir_to_java_layer::compiler::{ByteCodeIndex, compile_to_ir, JavaCompilerMethodAndFrameData};
use crate::ir_to_java_layer::java_stack::{JavaStackPosition, OpaqueFrameIdOrMethodID, OwnedJavaStack};
use crate::java::lang::class::JClass;
use crate::java::lang::int::Int;
use crate::java_values::{GcManagedObject, NativeJavaValue, StackNativeJavaValue};
use crate::jit::{MethodResolver, ToIR};
use crate::jit::state::{Labeler, NaiveStackframeLayout, runtime_class_to_allocated_object_type};
use crate::jit_common::java_stack::JavaStack;
use crate::runtime_class::RuntimeClass;
use crate::stack_entry::{StackEntryMut, StackEntryRef};
use crate::threading::safepoints::Monitor2;
use crate::utils::run_static_or_virtual;

pub mod compiler;
pub mod java_stack;
pub mod vm_exit_abi;

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub struct ExitNumber(u64);

pub struct JavaVMStateMethod {
    restart_points: HashMap<RestartPointID, IRInstructIndex>,
    ir_index_to_bytecode_pc: HashMap<IRInstructIndex, ByteCodeOffset>,
    associated_method_id: MethodId,
}

pub struct JavaVMStateWrapperInner<'gc_life> {
    most_up_to_date_ir_method_id_for_method_id: HashMap<MethodId, IRMethodID>,
    methods: HashMap<IRMethodID, JavaVMStateMethod>,
    max_exit_number: ExitNumber,
    method_exit_handlers: HashMap<ExitNumber, Box<dyn for<'l> Fn(&'gc_life JVMState<'gc_life>, &mut InterpreterStateGuard<'l, 'gc_life>, MethodId, &VMExitTypeWithArgs) -> JavaExitAction>>,
}

impl<'gc_life> JavaVMStateWrapperInner<'gc_life> {
    pub fn java_method_for_ir_method_id(&self, ir_method_id: IRMethodID) -> &JavaVMStateMethod {
        self.methods.get(&ir_method_id).unwrap()
    }

    pub fn associated_method_id(&self, ir_method_id: IRMethodID) -> MethodId {
        self.java_method_for_ir_method_id(ir_method_id).associated_method_id
    }

    pub fn restart_location(&self, ir_method_id: IRMethodID, restart_point: RestartPointID) -> IRInstructIndex {
        *self.methods.get(&ir_method_id).unwrap().restart_points.get(&restart_point).unwrap()
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

impl<'gc_life> JavaVMStateWrapperInner<'gc_life> {
    fn handle_vm_exit(jvm: &'gc_life JVMState<'gc_life>, int_state: &mut InterpreterStateGuard<'gc_life, 'l>, method_id: MethodId, vm_exit_type: &RuntimeVMExitInput, exiting_pc: ByteCodeOffset) -> IRVMExitAction {
        // let current_frame = int_state.current_frame();
        // let (rc, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
        // let view = rc.view();
        // let method_view = view.method_view_i(method_i);
        // let code = method_view.code_attribute().unwrap();
        // drop(current_frame);

        match vm_exit_type {
            RuntimeVMExitInput::AllocateObjectArray { type_, len, return_to_ptr, res_address } => {
                eprintln!("AllocateObjectArray");
                let type_ = jvm.cpdtype_table.read().unwrap().get_cpdtype(*type_).unwrap_ref_type().clone();
                assert!(*len >= 0);
                let rc = assert_inited_or_initing_class(jvm, CPDType::Ref(type_.clone()));
                let object_array = runtime_class_to_allocated_object_type(rc.as_ref(), int_state.current_loader(jvm), Some(*len as usize), int_state.thread().java_tid);
                let mut memory_region_guard = jvm.gc.memory_region.lock().unwrap();
                let array_size = object_array.size();
                let region_data = memory_region_guard.find_or_new_region_for(object_array);
                dbg!(region_data.region_type);
                let allocated_object = region_data.get_allocation();
                unsafe { res_address.write(allocated_object) }
                unsafe {
                    memset(allocated_object.as_ptr(), 0, array_size);
                }//todo init this properly according to type
                unsafe { *allocated_object.cast::<jint>().as_mut() = *len }//init the length
                IRVMExitAction::RestartAtPtr { ptr: *return_to_ptr }
            }
            RuntimeVMExitInput::LoadClassAndRecompile { .. } => todo!(),
            RuntimeVMExitInput::RunStaticNative { method_id, arg_start, num_args, res_ptr, return_to_ptr } => {
                eprintln!("RunStaticNative");
                int_state.debug_print_stack_trace(jvm, false);
                let (rc, method_i) = jvm.method_table.read().unwrap().try_lookup(*method_id).unwrap();
                let mut args_jv_handle = vec![];
                let class_view = rc.view();
                let method_view = class_view.method_view_i(method_i);
                let arg_types = &method_view.desc().arg_types;
                unsafe {
                    for (i, cpdtype) in (0..*num_args).zip(arg_types.iter()) {
                        let arg_ptr = arg_start.offset(-(i as isize) * size_of::<jlong>() as isize) as *const u64;//stack grows down
                        let native_jv = NativeJavaValue { as_u64: arg_ptr.read() };
                        dbg!(native_jv.as_u64);
                        args_jv_handle.push(native_jv.to_new_java_value(cpdtype, jvm))
                    }
                }
                assert!(jvm.thread_state.int_state_guard_valid.with(|refcell| { *refcell.borrow() }));
                let args_new_jv = args_jv_handle.iter().map(|handle|handle.as_njv()).collect();
                let res = run_native_method(jvm, int_state, rc, method_i, args_new_jv).unwrap();
                if let Some(res) = res {
                    unsafe { (*res_ptr as *mut NativeJavaValue<'static>).write(transmute::<NativeJavaValue<'_>, NativeJavaValue<'static>>(res.to_native())) }
                };
                IRVMExitAction::RestartAtPtr { ptr: *return_to_ptr }
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
                IRVMExitAction::RestartAtPtr { ptr: restart_point }
            }
            RuntimeVMExitInput::PutStatic { field_id, value_ptr, return_to_ptr } => {
                eprintln!("PutStatic");
                let (rc, field_i) = jvm.field_table.read().unwrap().lookup(*field_id);
                let view = rc.view();
                let field_view = view.field(field_i as usize);
                let mut static_vars_guard = rc.static_vars(jvm);
                let field_name = field_view.field_name();
                let static_var = static_vars_guard.get(field_name);
                let jv = unsafe { (*value_ptr as *mut NativeJavaValue<'gc_life>).as_ref() }.unwrap().to_java_value(&field_view.field_type(), jvm);
                static_vars_guard.set(field_name,todo!()/*jv*/);
                IRVMExitAction::RestartAtPtr { ptr: *return_to_ptr }
            }
            RuntimeVMExitInput::InitClassAndRecompile { class_type, current_method_id, restart_point, rbp } => {
                eprintln!("InitClassAndRecompile");
                let cpdtype = jvm.cpdtype_table.read().unwrap().get_cpdtype(*class_type).clone();
                let saved = int_state.frame_state_assert_save();
                let inited = check_initing_or_inited_class(jvm, int_state, cpdtype).unwrap();
                int_state.saved_assert_frame_same(saved);
                let method_resolver = MethodResolver { jvm, loader: int_state.current_loader(jvm) };
                jvm.java_vm_state.add_method(jvm, &method_resolver, *current_method_id);
                let restart_point = jvm.java_vm_state.lookup_restart_point(*current_method_id, *restart_point);
                IRVMExitAction::RestartAtPtr { ptr: restart_point }
            }
            RuntimeVMExitInput::AllocatePrimitiveArray { .. } => todo!(),
            RuntimeVMExitInput::LogFramePointerOffsetValue { value, return_to_ptr } => {
                eprintln!("value:{}", value);
                IRVMExitAction::RestartAtPtr { ptr: *return_to_ptr }
            }
            RuntimeVMExitInput::LogWholeFrame { return_to_ptr } => {
                eprintln!("LogWholeFrame");
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
                IRVMExitAction::RestartAtPtr { ptr: *return_to_ptr }
            }
            RuntimeVMExitInput::TraceInstructionBefore { method_id, return_to_ptr, bytecode_offset } => {
                let (rc, method_i) = jvm.method_table.read().unwrap().try_lookup(*method_id).unwrap();
                let view = rc.view();
                let method_view = view.method_view_i(method_i);
                let code = method_view.code_attribute().unwrap();
                let instr = code.instructions.get(bytecode_offset).unwrap();
                eprintln!("Before:{:?} {}", instr.info, bytecode_offset.0);
                if jvm.static_breakpoints.should_break(view.name().unwrap_name(),method_view.name(),method_view.desc().clone(),*bytecode_offset){
                    eprintln!("here");
                }
                IRVMExitAction::RestartAtPtr { ptr: *return_to_ptr }
            }
            RuntimeVMExitInput::TraceInstructionAfter { method_id, return_to_ptr, bytecode_offset } => {
                assert_eq!(Some(*method_id), int_state.current_frame().frame_view.ir_ref.method_id());
                let (rc, method_i) = jvm.method_table.read().unwrap().try_lookup(*method_id).unwrap();
                let view = rc.view();
                let method_view = view.method_view_i(method_i);
                let code = method_view.code_attribute().unwrap();
                let instr = code.instructions.get(bytecode_offset).unwrap();
                eprintln!("After:{}/{:?}", jvm.method_table.read().unwrap().lookup_method_string(*method_id, &jvm.string_pool), instr.info);
                dump_frame_contents(jvm, int_state);
                IRVMExitAction::RestartAtPtr { ptr: *return_to_ptr }
            }
            RuntimeVMExitInput::NPE { .. } => {
                int_state.debug_print_stack_trace(jvm, true);
                todo!()
            }
            RuntimeVMExitInput::BeforeReturn { return_to_ptr, frame_size_allegedly } => {
                // int_state.debug_print_stack_trace(jvm, false);
                let saved = int_state.frame_state_assert_save();
                dbg!(saved);
                int_state.current_frame().ir_stack_entry_debug_print();
                if let Some(previous_frame) = int_state.previous_frame() {
                    // previous_frame.ir_stack_entry_debug_print();
                    dbg!(frame_size_allegedly);
                }
                IRVMExitAction::RestartAtPtr { ptr: *return_to_ptr }
            }
            RuntimeVMExitInput::AllocateObject { type_, return_to_ptr, res_address } => {
                eprintln!("AllocateObject");
                let type_ = jvm.cpdtype_table.read().unwrap().get_cpdtype(*type_).unwrap_ref_type().clone();
                let rc = assert_inited_or_initing_class(jvm, CPDType::Ref(type_.clone()));
                let object_type = runtime_class_to_allocated_object_type(rc.as_ref(), int_state.current_loader(jvm), None, int_state.thread().java_tid);
                let mut memory_region_guard = jvm.gc.memory_region.lock().unwrap();
                let object_size = object_type.size();
                let allocated_object = memory_region_guard.find_or_new_region_for(object_type).get_allocation();
                unsafe {
                    libc::memset(allocated_object.as_ptr(), 0, object_size);
                }//todo do correct initing of fields
                unsafe { res_address.write(allocated_object) }
                IRVMExitAction::RestartAtPtr { ptr: *return_to_ptr }
            }
            RuntimeVMExitInput::NewString { return_to_ptr, res, compressed_wtf8 } => {
                eprintln!("NewString");
                let wtf8buf = compressed_wtf8.to_wtf8(&jvm.wtf8_pool);
                dbg!(&wtf8buf);
                int_state.debug_print_stack_trace(jvm, false);
                let jstring = JString::from_rust(jvm, int_state, wtf8buf).expect("todo exceptions");
                dbg!(jstring.value(jvm));
                let jv = jstring.java_value();
                unsafe {
                    let raw_64 = jv.to_native().as_u64;
                    (*res as *mut u64).write(raw_64);
                }
                IRVMExitAction::RestartAtPtr { ptr: *return_to_ptr }
            }
            RuntimeVMExitInput::NewClass { type_, res, return_to_ptr } => {
                eprintln!("NewClass");
                let cpdtype = jvm.cpdtype_table.write().unwrap().get_cpdtype(*type_).clone();
                let jclass = JClass::from_type(jvm, int_state, cpdtype).unwrap();
                let jv = jclass.java_value();
                unsafe {
                    let raw_64 = jv.to_native().as_u64;
                    (*res as *mut u64).write(raw_64);
                };
                IRVMExitAction::RestartAtPtr { ptr: *return_to_ptr }
            }
            RuntimeVMExitInput::InvokeVirtualResolve { object_ref, return_to_ptr, inheritance_id, target_method_id: debug_method_id } => {
                eprintln!("InvokeVirtualResolve");
                let mut memory_region_guard = jvm.gc.memory_region.lock().unwrap();
                let allocated_type = memory_region_guard.find_object_allocated_type(NonNull::new(*object_ref as usize as *mut c_void).unwrap()).clone();
                let allocated_type_id = memory_region_guard.lookup_or_add_type(&allocated_type);
                drop(memory_region_guard);
                let lookup_res = jvm.vtables.read().unwrap().lookup_resolved(allocated_type_id, *inheritance_id);
                let ResolvedInvokeVirtual {
                    address,
                    ir_method_id,
                    method_id,
                    new_frame_size
                } = match lookup_res {
                    Ok(resolved) => {
                        resolved
                    }
                    Err(NotCompiledYet {}) => {
                        jvm.java_vm_state.add_method(jvm, &MethodResolver { jvm, loader: int_state.current_loader(jvm) }, *debug_method_id);
                        jvm.vtables.read().unwrap().lookup_resolved(allocated_type_id, *inheritance_id).unwrap()
                    }
                };
                let mut start_diff = SavedRegistersWithoutIPDiff::no_change();
                start_diff.add_change(InvokeVirtualResolve::ADDRESS_RES, address as *mut c_void);
                start_diff.add_change(InvokeVirtualResolve::IR_METHOD_ID_RES, ir_method_id.0 as *mut c_void);
                start_diff.add_change(InvokeVirtualResolve::METHOD_ID_RES, method_id as *mut c_void);
                start_diff.add_change(InvokeVirtualResolve::NEW_FRAME_SIZE_RES, new_frame_size as *mut c_void);

                IRVMExitAction::RestartWithRegisterState {
                    diff: SavedRegistersWithIPDiff {
                        rip: Some(*return_to_ptr),
                        saved_registers_without_ip: Some(start_diff),
                    }
                }
            }
            RuntimeVMExitInput::MonitorEnter { obj_ptr, return_to_ptr } => {
                let mut monitors_guard = jvm.object_monitors.write().unwrap();
                let next_id = monitors_guard.len();
                let monitor = monitors_guard.entry(*obj_ptr).or_insert_with(|| Monitor2::new(next_id));
                monitor.lock(jvm, int_state).unwrap();
                IRVMExitAction::RestartAtPtr { ptr: *return_to_ptr }
            }
            RuntimeVMExitInput::MonitorExit { obj_ptr, return_to_ptr } => {
                let mut monitors_guard = jvm.object_monitors.write().unwrap();
                let next_id = monitors_guard.len();
                let monitor = monitors_guard.entry(*obj_ptr).or_insert_with(|| Monitor2::new(next_id));
                monitor.unlock(jvm, int_state).unwrap();
                IRVMExitAction::RestartAtPtr { ptr: *return_to_ptr }
            }
        }
    }
}

pub fn dump_frame_contents<'gc_life, 'l>(jvm: &'gc_life JVMState<'gc_life>, int_state: &mut InterpreterStateGuard<'gc_life, 'l>) {
    let current_frame = int_state.current_frame();
    dump_frame_contents_impl(jvm, current_frame)
}

pub fn dump_frame_contents_impl(jvm: &'gc_life JVMState<'gc_life>, current_frame: StackEntryRef<'gc_life, '_>) {
    let local_var_types = current_frame.local_var_types(jvm);
    let local_vars = current_frame.local_vars(jvm);
    eprint!("Local Vars:");
    unsafe {
        for (i, local_var_type) in local_var_types.into_iter().enumerate() {
            match local_var_type.to_runtime_type() {
                RuntimeType::TopType => {
                    let jv = local_vars.raw_get(i as u16);
                    eprint!("#{}: Top: {:?}\t", i, jv as *const c_void)
                }
                _ => {
                    let jv = local_vars.get(i as u16, local_var_type.to_runtime_type());
                    eprint!("#{}: {:?}\t", i, jv.as_njv())
                }
            }
        }
    }
    eprintln!();
    let operand_stack_types = current_frame.operand_stack(jvm).types();
    let operand_stack = current_frame.operand_stack(jvm);
    // current_frame.ir_stack_entry_debug_print();
    eprint!("Operand Stack:");
    for (i, operand_stack_type) in operand_stack_types.into_iter().enumerate() {
        let jv = operand_stack.get(i as u16, operand_stack_type);
        eprint!("#{}: {:?}\t", i, jv.as_njv())
    }
    eprintln!()
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
                methods: Default::default(),
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
        let (ir_method_id, restart_points) = self.ir.add_function(vec![IRInstr::VMExit2 { exit_type: IRVMExitType::TopLevelReturn {} }], FRAME_HEADER_END_OFFSET, Arc::new(|event, ir_stack_mut, ir_vm_state, extra| {
            match &event.exit_type {
                RuntimeVMExitInput::TopLevelReturn { return_value } => IRVMExitAction::ExitVMCompletely { return_data: *return_value },
                _ => panic!()
            }
        }));
        assert!(restart_points.is_empty());
        self.ir.init_top_level_exit_id(ir_method_id)
    }

    pub fn run_method(&'vm_life self, jvm: &'vm_life JVMState<'vm_life>, int_state: &'_ mut InterpreterStateGuard<'vm_life, 'l>, method_id: MethodId) -> u64 {
        let (rc, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
        let view = rc.view();
        let method_view = view.method_view_i(method_i);
        let method_name = method_view.name().0.to_str(&jvm.string_pool);
        let class_name = view.name().unwrap_name().0.to_str(&jvm.string_pool);
        let desc_str = method_view.desc_str().to_str(&jvm.string_pool);
        eprintln!("ENTER RUN METHOD: {} {} {}", &class_name, &method_name, &desc_str);
        let ir_method_id = *self.inner.read().unwrap().most_up_to_date_ir_method_id_for_method_id.get(&method_id).unwrap();
        let current_frame_pointer = int_state.current_frame().frame_view.ir_ref.frame_ptr();
        let assert_data = int_state.frame_state_assert_save_from(current_frame_pointer);
        let mut frame_to_run_on = int_state.current_frame_mut();
        let frame_ir_method_id = frame_to_run_on.frame_view.ir_mut.downgrade().ir_method_id().unwrap();
        assert_eq!(self.inner.read().unwrap().associated_method_id(ir_method_id), method_id);
        if frame_ir_method_id != ir_method_id {
            frame_to_run_on.frame_view.ir_mut.set_ir_method_id(ir_method_id);
        }
        assert!(jvm.thread_state.int_state_guard_valid.with(|refcell| { *refcell.borrow() }));
        let res = self.ir.run_method(ir_method_id, &mut frame_to_run_on.frame_view.ir_mut, &mut ());
        int_state.saved_assert_frame_from(assert_data, current_frame_pointer);
        eprintln!("EXIT RUN METHOD: {} {} {}", &class_name, &method_name, &desc_str);
        res
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
}


impl<'vm_life> JavaVMStateWrapper<'vm_life> {
    pub fn add_method(&'vm_life self, jvm: &'vm_life JVMState<'vm_life>, resolver: &MethodResolver<'vm_life>, method_id: MethodId) {
        eprintln!("Re/Compile: {}", jvm.method_table.read().unwrap().lookup_method_string(method_id, &jvm.string_pool));
        //todo need some mechanism for detecting recompile necessary
        let mut java_function_frame_guard = jvm.java_function_frame_data.write().unwrap();
        let java_frame_data = &java_function_frame_guard.entry(method_id).or_insert_with(|| JavaCompilerMethodAndFrameData::new(jvm, method_id));
        let ir_instructions_and_offsets = compile_to_ir(resolver, &self.labeler, java_frame_data);
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
            };
            int_state.register_interpreter_state_guard(jvm);
            unsafe {
                let exiting_frame_position_rbp = ir_vm_exit_event.inner.saved_guest_registers.saved_registers_without_ip.rbp;
                let exiting_stack_pointer = ir_vm_exit_event.inner.saved_guest_registers.saved_registers_without_ip.rsp;
                if exiting_stack_pointer != mmaped_top {
                    let offset = exiting_frame_position_rbp.offset_from(exiting_stack_pointer).abs() as usize /*+ size_of::<u64>()*/;
                    let frame_ref = int_state.current_frame().frame_view.ir_ref;
                    let expected_current_frame_size = frame_ref.frame_size(&jvm.java_vm_state.ir);
                    // dbg!(jvm.method_table.read().unwrap().lookup_method_string(int_state.current_frame().frame_view.ir_ref.method_id().unwrap(), &jvm.string_pool));
                    // dbg!(&ir_vm_exit_event.exit_type);
                    assert_eq!(offset, expected_current_frame_size);
                }
            }
            JavaVMStateWrapperInner::handle_vm_exit(jvm, &mut int_state, method_id, &ir_vm_exit_event.exit_type, exiting_pc)
        });
        let mut ir_instructions = vec![];
        let mut ir_index_to_bytecode_pc = HashMap::new();
        //todo consider making this use iterators and stuff.
        for (i, (offset, ir_instr)) in ir_instructions_and_offsets.into_iter().enumerate() {
            let prev_value = ir_index_to_bytecode_pc.insert(IRInstructIndex(i), offset);
            assert!(prev_value.is_none());
            ir_instructions.push(ir_instr);
        }
        let (ir_method_id, restart_points) = self.ir.add_function(ir_instructions, java_frame_data.full_frame_size(), ir_exit_handler);
        let mut write_guard = self.inner.write().unwrap();
        write_guard.most_up_to_date_ir_method_id_for_method_id.insert(method_id, ir_method_id);
        write_guard.methods.insert(ir_method_id, JavaVMStateMethod {
            restart_points,
            ir_index_to_bytecode_pc,
            associated_method_id: method_id,
        });
        jvm.vtables.write().unwrap().notify_compile_or_recompile(jvm, method_id, ResolvedInvokeVirtual {
            address: self.ir.lookup_ir_method_id_pointer(ir_method_id),
            ir_method_id,
            method_id,
            new_frame_size: java_frame_data.full_frame_size(),
        });
        drop(write_guard);
    }
}

