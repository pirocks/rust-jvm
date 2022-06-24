use std::collections::HashMap;
use std::ffi::c_void;
use std::mem::size_of;
use std::sync::{Arc, RwLock};
use another_jit_vm::code_modification::GlobalCodeEditingLock;
use another_jit_vm::{IRMethodID, Register};
use another_jit_vm_ir::{ExitHandlerType, IRInstructIndex, IRVMExitAction, IRVMExitEvent, IRVMState, WasException};
use another_jit_vm_ir::compiler::{IRInstr, RestartPointID};
use another_jit_vm_ir::ir_stack::{IRStackMut};
use another_jit_vm_ir::vm_exit_abi::{IRVMExitType};
use gc_memory_layout_common::layout::{FRAME_HEADER_END_OFFSET, FrameHeader, NativeStackframeMemoryLayout};
use interface_vtable::ResolvedInterfaceVTableEntry;
use rust_jvm_common::{ByteCodeOffset, MethodId};
use crate::{InterpreterStateGuard, JVMState, MethodResolverImpl};
use crate::function_call_targets_updating::FunctionCallTargetsByFunction;
use stage0::compiler::{compile_to_ir, Labeler, native_to_ir, NeedsRecompileIf};
use stage0::compiler_common::{JavaCompilerMethodAndFrameData, MethodResolver};
use crate::ir_to_java_layer::java_stack::OpaqueFrameIdOrMethodID;
use crate::ir_to_java_layer::{ByteCodeIRMapping, JavaVMStateMethod, JavaVMStateWrapperInner};
use crate::jit::{NotCompiledYet, ResolvedInvokeVirtual};

pub struct JavaVMStateWrapper<'vm> {
    pub ir: IRVMState<'vm, ()>,
    pub inner: RwLock<JavaVMStateWrapperInner>,
    // should be per thread
    labeler: Labeler,
    function_call_targets: RwLock<FunctionCallTargetsByFunction>,
    modication_lock: GlobalCodeEditingLock,
}

impl<'vm> JavaVMStateWrapper<'vm> {
    pub fn new() -> Self {
        let res = Self {
            ir: IRVMState::new(),
            inner: RwLock::new(JavaVMStateWrapperInner {
                most_up_to_date_ir_method_id_for_method_id: Default::default(),
                methods: Default::default(),
            }),
            labeler: Labeler::new(),
            function_call_targets: RwLock::new(FunctionCallTargetsByFunction::new()),
            modication_lock: GlobalCodeEditingLock::new(),
        };
        res
    }

    pub fn init(&'vm self, jvm: &'vm JVMState<'vm>) {
        self.ir.inner.write().unwrap().handler.get_or_init(|| {
            let ir_exit_handler: ExitHandlerType<'vm, ()> = Arc::new(move |ir_vm_exit_event: &IRVMExitEvent, ir_stack_mut: IRStackMut, ir_vm_state: &IRVMState<'vm, ()>, extra| {
                JavaVMStateWrapper::exit_handler(&jvm, &ir_vm_exit_event, ir_stack_mut)
            });
            ir_exit_handler
        });
        self.add_top_level_vm_exit();
    }

    pub fn add_top_level_vm_exit(&'vm self) {
        //&IRVMExitEvent, IRStackMut, &IRVMState<'vm, ExtraData>, &mut ExtraData
        let ir_method_id = self.ir.reserve_method_id();
        let (ir_method_id, restart_points, _) = self.ir.add_function(vec![IRInstr::VMExit2 { exit_type: IRVMExitType::TopLevelReturn {} }], FRAME_HEADER_END_OFFSET, ir_method_id, self.modication_lock.acquire());
        assert!(restart_points.is_empty());
        self.ir.init_top_level_exit_id(ir_method_id)
    }

    pub fn run_method<'l>(&'vm self, jvm: &'vm JVMState<'vm>, int_state: &'_ mut InterpreterStateGuard<'vm, 'l>, method_id: MethodId) -> Result<u64, WasException> {
        // let (rc, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
        // let view = rc.view();
        // let method_view = view.method_view_i(method_i);
        // let method_name = method_view.name().0.to_str(&jvm.string_pool);
        // let class_name = view.name().unwrap_name().0.to_str(&jvm.string_pool);
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
        assert!(jvm.thread_state.int_state_guard_valid.with(|inner| inner.borrow().clone()));
        let res = match self.ir.run_method(ir_method_id, &mut frame_to_run_on.frame_view.ir_mut, &mut ()) {
            Ok(res) => {
                // eprintln!("{}",jvm.method_table.read().unwrap().lookup_method_string(method_id, &jvm.string_pool));
                res
            }
            Err(err_obj) => {
                let obj = jvm.gc.register_root_reentrant(jvm, err_obj);
                int_state.set_throw(Some(obj));
                return Err(WasException {});
            }
        };
        int_state.saved_assert_frame_from(assert_data, current_frame_pointer);
        // eprintln!("EXIT RUN METHOD: {} {} {}", &class_name, &method_name, &desc_str);
        Ok(res)
    }

    pub fn lookup_ir_method_id(&self, opaque_or_not: OpaqueFrameIdOrMethodID) -> IRMethodID {
        self.try_lookup_ir_method_id(opaque_or_not).unwrap()
    }

    pub fn lookup_resolved_invoke_virtual(&self, method_id: MethodId, resolver: &MethodResolverImpl) -> Result<ResolvedInvokeVirtual, NotCompiledYet> {
        let ir_method_id = self.lookup_method_ir_method_id(method_id);
        let address = self.ir.lookup_ir_method_id_pointer(ir_method_id);

        let new_frame_size = if resolver.is_native(method_id) {
            resolver.lookup_native_method_layout(method_id).full_frame_size()
        } else {
            resolver.lookup_partial_method_layout(method_id).full_frame_size()
        };
        assert!(new_frame_size > size_of::<FrameHeader>());
        Ok(ResolvedInvokeVirtual {
            address: address.as_ptr(),
            ir_method_id,
            method_id,
            new_frame_size,
        })
    }

    pub fn lookup_method_ir_method_id(&self, method_id: MethodId) -> IRMethodID {
        self.try_lookup_method_ir_method_id(method_id).unwrap()
    }

    pub fn try_lookup_method_ir_method_id(&self, method_id: MethodId) -> Option<IRMethodID> {
        let inner = self.inner.read().unwrap();
        inner.most_up_to_date_ir_method_id_for_method_id.get(&method_id).cloned()
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
        let pc = *method.byte_code_ir_mapping.as_ref()?.ir_index_to_bytecode_pc.get(&ir_instruct_index).unwrap();
        Some((method_id, pc))
    }

    pub fn lookup_byte_code_offset(&self, ir_method_id: IRMethodID, java_pc: ByteCodeOffset) -> *const c_void {
        let read_guard = self.inner.read().unwrap();
        let ir_instruct_index = *read_guard.methods.get(&ir_method_id).unwrap().byte_code_ir_mapping.as_ref().unwrap().bytecode_pc_to_start_ir_index.get(&java_pc).unwrap();
        self.ir.lookup_location_of_ir_instruct(ir_method_id, ir_instruct_index).0
    }
}

//todo rework so that we always recompile but sometimes recompile to exit and interpret

impl<'vm> JavaVMStateWrapper<'vm> {
    pub fn add_method_if_needed(&'vm self, jvm: &'vm JVMState<'vm>, resolver: &MethodResolverImpl<'vm>, method_id: MethodId, interpreter_debug: bool) {
        // let compile_guard = jvm.perf_metrics.compilation_start();
        // let method_string = jvm.method_table.read().unwrap().lookup_method_string(method_id, &jvm.string_pool);
        // dbg!(method_string);
        if jvm.recompilation_conditions.read().unwrap().should_recompile(method_id, resolver, interpreter_debug) {
            let prev_address = self.try_lookup_method_ir_method_id(method_id).map(|it| self.ir.lookup_ir_method_id_pointer(it));
            let mut recompilation_guard = jvm.recompilation_conditions.write().unwrap();
            let mut recompile_conditions = recompilation_guard.recompile_conditions(method_id);
            eprintln!("Re/Compile: {}", jvm.method_table.read().unwrap().lookup_method_string(method_id, &jvm.string_pool));
            //todo need some mechanism for detecting recompile necessary
            //todo unify resolver and recompile_conditions
            let is_native = jvm.is_native_by_method_id(method_id);
            let reserved_method_id = self.ir.reserve_method_id();
            let (ir_instructions, full_frame_size, byte_code_ir_mapping) = if is_native {
                let ir_instr = native_to_ir(resolver, method_id, reserved_method_id);
                (ir_instr, NativeStackframeMemoryLayout { num_locals: jvm.num_local_vars_native(method_id) }.full_frame_size(), None)
            } else {
                let mut java_function_frame_guard = jvm.java_function_frame_data.write().unwrap();
                let java_frame_data = &java_function_frame_guard.entry(method_id)
                    .or_insert_with(|| {
                        let function_frame_data = jvm.function_frame_type_data.read().unwrap();
                        let method_table = jvm.method_table.read().unwrap();
                        JavaCompilerMethodAndFrameData::new(jvm.instruction_trace_options.should_trace(method_id, jvm), &method_table, &function_frame_data.no_tops.get(&method_id).unwrap(), method_id)
                    });
                let ir_instructions_and_offsets = if resolver.compile_interpreted(method_id) {
                    recompile_conditions.add_condition(NeedsRecompileIf::Interpreted { method_id });
                    vec![
                        (ByteCodeOffset(0), IRInstr::IRStart {
                            temp_register: Register(0),
                            ir_method_id: reserved_method_id,
                            method_id,
                            frame_size: dbg!(java_frame_data.full_frame_size()),
                            num_locals: java_frame_data.local_vars,
                        }),
                        (ByteCodeOffset(0), IRInstr::VMExit2 {
                            exit_type: IRVMExitType::RunInterpreted { method_id }
                        }),
                        (ByteCodeOffset(0), IRInstr::Return {
                            return_val: Some(Register(0)),
                            temp_register_1: Register(1),
                            temp_register_2: Register(2),
                            temp_register_3: Register(3),
                            temp_register_4: Register(4),
                            frame_size: java_frame_data.full_frame_size(),
                        }),
                    ]
                } else {
                    compile_to_ir(resolver, &self.labeler, java_frame_data, &mut recompile_conditions, reserved_method_id)
                };
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
                (ir_instructions, java_frame_data.full_frame_size(), Some(ByteCodeIRMapping {
                    ir_index_to_bytecode_pc,
                    bytecode_pc_to_start_ir_index,
                }))
            };
            let (ir_method_id, restart_points, function_call_targets) = self.ir.add_function(ir_instructions, full_frame_size, reserved_method_id, self.modication_lock.acquire());
            self.function_call_targets.write().unwrap().sink_targets(function_call_targets);
            let mut write_guard = self.inner.write().unwrap();
            write_guard.most_up_to_date_ir_method_id_for_method_id.insert(method_id, ir_method_id);
            write_guard.methods.insert(ir_method_id, JavaVMStateMethod {
                restart_points,
                byte_code_ir_mapping,
                associated_method_id: method_id,
            });
            let new_address = self.ir.lookup_ir_method_id_pointer(ir_method_id);
            self.function_call_targets.read().unwrap().update_target(method_id, new_address, self.modication_lock.acquire());
            if let Some(prev_address) = prev_address {
                jvm.vtables.lock().unwrap().update_address(prev_address, new_address);
                jvm.invoke_interface_lookup_cache.write().unwrap().update(method_id, ResolvedInterfaceVTableEntry {
                    address: new_address,
                    ir_method_id,
                    method_id,
                    new_frame_size: full_frame_size,
                });
                jvm.itables.lock().unwrap().update(prev_address.into(),new_address.into());
            }
            drop(write_guard);
        }
        assert!(!jvm.recompilation_conditions.read().unwrap().should_recompile(method_id, resolver, false));
    }

    #[inline(never)]
    fn exit_handler(jvm: &'vm JVMState<'vm>, ir_vm_exit_event: &IRVMExitEvent, ir_stack_mut: IRStackMut) -> IRVMExitAction {
        let ir_stack_mut: IRStackMut = ir_stack_mut;
        // let read_guard = self.inner.read().unwrap();
        // let ir_method_id = ir_vm_exit_event.ir_method;
        // let method = read_guard.methods.get(&ir_method_id).unwrap();
        // let method_id = method.associated_method_id;
        // let exiting_pc = *method.ir_index_to_bytecode_pc.get(&ir_vm_exit_event.exit_ir_instr).unwrap();
        // drop(read_guard);
        let mmaped_top = ir_stack_mut.owned_ir_stack.native.mmaped_top;

        let mut int_state = InterpreterStateGuard::LocalInterpreterState {
            int_state: ir_stack_mut,
            thread: jvm.thread_state.get_current_thread(),
            registered: false,
            jvm,
            current_exited_pc: ir_vm_exit_event.exit_type.exiting_pc(),
            throw: None,
        };
        let old_intstate = int_state.register_interpreter_state_guard(jvm);
        unsafe {
            let exiting_frame_position_rbp = ir_vm_exit_event.inner.saved_guest_registers.saved_registers_without_ip.rbp;
            let exiting_stack_pointer = ir_vm_exit_event.inner.saved_guest_registers.saved_registers_without_ip.rsp;
            if exiting_stack_pointer != mmaped_top {
                let offset = exiting_frame_position_rbp.offset_from(exiting_stack_pointer).abs() as usize;
                let frame_ref = int_state.current_frame().frame_view.ir_ref;
                if let Some(expected_current_frame_size) = frame_ref.try_frame_size(&jvm.java_vm_state.ir) {
                    if offset != expected_current_frame_size {
                        int_state.debug_print_stack_trace(jvm);
                        dbg!(offset);
                        dbg!(expected_current_frame_size);
                        let java_function_frame_guard = jvm.java_function_frame_data.read().unwrap();
                        let java_frame_data = &java_function_frame_guard.get(&frame_ref.method_id().unwrap()).unwrap();
                        dbg!(java_frame_data.full_frame_size());
                        dbg!(jvm.method_table.read().unwrap().lookup_method_string(frame_ref.method_id().unwrap(), &jvm.string_pool));
                        panic!()
                    }
                }
            }
        }
        let res = JavaVMStateWrapperInner::handle_vm_exit(jvm, Some(&mut int_state), &ir_vm_exit_event.exit_type);
        int_state.deregister_int_state(jvm, old_intstate);
        res
    }
}
