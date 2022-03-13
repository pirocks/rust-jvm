use std::arch::x86_64::_mm256_testc_pd;
use std::cell::RefCell;
use std::collections::HashSet;
use std::ffi::c_void;
use std::mem::{size_of, transmute};
use std::ops::{Deref, DerefMut};
use std::ptr::{null_mut, slice_from_raw_parts};
use std::sync::{Arc, MutexGuard, RwLockWriteGuard};

use iced_x86::CC_b::c;
use iced_x86::CC_ne::ne;
use itertools::Itertools;

use another_jit_vm_ir::ir_stack::{IRFrameIterRef, IRPushFrameGuard, IRStackMut, OwnedIRStack};
use another_jit_vm_ir::IRMethodID;
use classfile_view::view::{ClassView, HasAccessFlags};
use gc_memory_layout_common::FramePointerOffset;
use jvmti_jni_bindings::{jobject, jvalue};
use rust_jvm_common::ByteCodeOffset;
use rust_jvm_common::classfile::CPIndex;
use rust_jvm_common::classfile::InstructionInfo::{ireturn, jsr};
use rust_jvm_common::loading::LoaderName;
use rust_jvm_common::runtime_type::RuntimeType;
use threads::signal::__pthread_cond_s__bindgen_ty_2;

use crate::interpreter_state::AddFrameNotifyError::{NothingAtDepth, Opaque};
use crate::ir_to_java_layer::dump_frame_contents_impl;
use crate::ir_to_java_layer::java_stack::{JavaStackPosition, OpaqueFrameIdOrMethodID, OwnedJavaStack, RuntimeJavaStackFrameMut, RuntimeJavaStackFrameRef};
use crate::java_values::{GcManagedObject, JavaValue, NativeJavaValue};
use crate::jit::MethodResolver;
use crate::jit_common::java_stack::{JavaStack, JavaStatus};
use crate::jvm_state::JVMState;
use crate::new_java_values::{AllocatedObject, NewJVObject};
use crate::rust_jni::native_util::{from_object, to_object};
use crate::stack_entry::{FrameView, NonNativeFrameData, OpaqueFrameOptional, StackEntry, StackEntryMut, StackEntryPush, StackEntryRef, StackIter};
use crate::threading::JavaThread;

pub struct InterpreterState<'gc_life> {
    pub call_stack: OwnedJavaStack<'gc_life>,
    jvm: &'gc_life JVMState<'gc_life>,
    pub current_stack_position: JavaStackPosition,
}

impl<'gc_life> InterpreterState<'gc_life> {
    pub(crate) fn new(jvm: &'gc_life JVMState<'gc_life>) -> Self {
        InterpreterState {
            call_stack: OwnedJavaStack::new(&jvm.java_vm_state, jvm),
            jvm,
            current_stack_position: JavaStackPosition::Top,
        }
    }
}

pub enum InterpreterStateGuard<'vm_life, 'l> {
    //todo these internals need to change to reflect that we need to halt thread to get current rbp.
    RemoteInterpreterState {
        int_state: Option<MutexGuard<'l, InterpreterState<'vm_life>>>,
        thread: Arc<JavaThread<'vm_life>>,
        registered: bool,
        jvm: &'vm_life JVMState<'vm_life>,
    },
    LocalInterpreterState {
        int_state: IRStackMut<'l>,
        thread: Arc<JavaThread<'vm_life>>,
        registered: bool,
        jvm: &'vm_life JVMState<'vm_life>,
        current_exited_pc: Option<ByteCodeOffset>,
    },
}

thread_local! {
pub static CURRENT_INT_STATE_GUARD_VALID :RefCell<bool> = RefCell::new(false);
}

thread_local! {
pub static CURRENT_INT_STATE_GUARD :RefCell<Option<*mut InterpreterStateGuard<'static,'static>>> = RefCell::new(None);
}

#[must_use]
pub struct OldInterpreterState{
    old: Option<*mut InterpreterStateGuard<'static,'static>>
}

impl<'gc_life, 'interpreter_guard> InterpreterStateGuard<'gc_life, 'interpreter_guard> {
    /*pub fn copy_with_new_stack_position(&self, new_stack_position: JavaStackPosition) -> Self {
        let InterpreterStateGuard {
            int_state: InterpreterState {
                call_stack,
                jvm,
                current_stack_position
            },
            thread,
            registered
        } = self;
        Self {
            int_state: InterpreterState {
                call_stack: call_stack.clone(),
                jvm,
                current_stack_position: new_stack_position,
            },
            thread: thread.clone(),
            registered: false,
        }
    }*/

    pub fn registered(&self) -> bool {
        match self {
            InterpreterStateGuard::RemoteInterpreterState { registered, .. } => *registered,
            InterpreterStateGuard::LocalInterpreterState { registered, .. } => *registered
        }
    }

    pub fn thread(&self) -> Arc<JavaThread<'gc_life>> {
        match self {
            InterpreterStateGuard::RemoteInterpreterState { thread, .. } => thread.clone(),
            InterpreterStateGuard::LocalInterpreterState { thread, .. } => thread.clone()
        }
    }

    pub fn self_check(&self, jvm: &'gc_life JVMState<'gc_life>) {
        todo!()
        /*for stack_entry in self.cloned_stack_snapshot(jvm) {
            for jv in stack_entry.operand_stack.iter().chain(stack_entry.local_vars.iter()) {
                jv.self_check();
            }
        }*/
    }

    pub fn register_interpreter_state_guard(&mut self, jvm: &'gc_life JVMState<'gc_life>) -> OldInterpreterState {
        let ptr = unsafe { transmute::<_, *mut InterpreterStateGuard<'static, 'static>>(self as *mut InterpreterStateGuard<'gc_life, '_>) };
        let old = jvm.thread_state.int_state_guard.get().deref().borrow().clone();
        jvm.thread_state.int_state_guard.get().replace(ptr.into());
        jvm.thread_state.int_state_guard_valid.get().replace(true);
        match self {
            InterpreterStateGuard::RemoteInterpreterState { registered, .. } => {
                *registered = true;
            }
            InterpreterStateGuard::LocalInterpreterState { registered, .. } => {
                *registered = true;
            }
        }
        assert!(self.thread().is_alive());
        OldInterpreterState{
            old
        }
    }

    pub fn deregister_int_state(&mut self, jvm: &'gc_life JVMState<'gc_life>, old: OldInterpreterState) {
        jvm.thread_state.int_state_guard.get().replace(old.old);
    }

    pub fn new(jvm: &'gc_life JVMState<'gc_life>, thread: Arc<JavaThread<'gc_life>>, int_state: MutexGuard<'interpreter_guard, InterpreterState<'gc_life>>) -> InterpreterStateGuard<'gc_life, 'interpreter_guard> {
        jvm.thread_state.int_state_guard_valid.get().replace(false);
        Self::RemoteInterpreterState { int_state: Some(int_state), thread: thread.clone(), registered: true, jvm }
    }

    pub fn java_stack(&mut self) -> &mut OwnedJavaStack<'gc_life> {
        todo!()
        // &mut self.int_state.as_mut().unwrap().call_stack
    }

    pub fn current_loader(&self, jvm: &'gc_life JVMState<'gc_life>) -> LoaderName {
        self.current_frame().loader(jvm)
    }

    pub fn current_class_view(&self, jvm: &'gc_life JVMState<'gc_life>) -> Arc<dyn ClassView> {
        self.current_frame().try_class_pointer(jvm).unwrap().view()
    }

    pub fn current_frame(&'_ self) -> StackEntryRef<'gc_life, '_> {
        match self {
            InterpreterStateGuard::RemoteInterpreterState { .. } => {
                todo!()
            }
            InterpreterStateGuard::LocalInterpreterState { int_state, jvm, current_exited_pc, .. } => {
                return StackEntryRef {
                    frame_view: RuntimeJavaStackFrameRef {
                        ir_ref: int_state.current_frame_ref(),
                        jvm,
                    },
                    pc: *current_exited_pc,
                };
            }
        }
    }

    pub fn current_frame_mut(&'_ mut self) -> StackEntryMut<'gc_life, '_> {
        match self {
            InterpreterStateGuard::RemoteInterpreterState { .. } => todo!(),
            InterpreterStateGuard::LocalInterpreterState { int_state, jvm, .. } => {
                return StackEntryMut {
                    frame_view: RuntimeJavaStackFrameMut {
                        ir_mut: int_state.current_frame_mut(),
                        jvm,
                    }
                };
            }
        }
    }

    pub fn raw_read_at_frame_pointer_offset(&self, offset: FramePointerOffset, expected_type: RuntimeType) -> JavaValue<'gc_life> {
        /*let interpreter_state = self.int_state.as_ref().unwrap().deref();
        match interpreter_state {
            InterpreterState::Jit { call_stack, jvm } => {
                let frame_ptr = call_stack.current_frame_ptr();
                unsafe {
                    let offseted = frame_ptr.offset(offset.0 as isize);
                    FrameView::read_target(jvm, offseted, expected_type)
                }
            }
        }*/
        todo!()
    }

    pub fn raw_write_at_frame_pointer_offset(&self, offset: FramePointerOffset, jv: jvalue) {
        /*let interpreter_state = self.int_state.as_ref().unwrap().deref();
        match interpreter_state {
            InterpreterState::Jit { call_stack, jvm } => {
                let frame_ptr = call_stack.current_frame_ptr();
                unsafe {
                    let offseted = frame_ptr.offset(offset.0 as isize);
                    FrameView::raw_write_target(offseted, jv);
                }
            }
        }*/
        todo!()
    }

    pub fn push_current_operand_stack(&mut self, jval: JavaValue<'gc_life>) {
        self.current_frame_mut().push(jval)
    }

    pub fn pop_current_operand_stack(&mut self, expected_type: Option<RuntimeType>) -> JavaValue<'gc_life> {
        self.current_frame_mut().operand_stack_mut().pop(expected_type).unwrap()
    }

    pub fn previous_frame_mut(&mut self) -> StackEntryMut<'gc_life, '_> {
        /*match self.int_state.as_mut().unwrap().deref_mut() {
            /*InterpreterState::LegacyInterpreter { call_stack, .. } => {
                let len = call_stack.len();
                StackEntryMut::LegacyInterpreter { entry: &mut call_stack[len - 2] }
            }*/
            InterpreterState::Jit { call_stack, jvm } => StackEntryMut::Jit { frame_view: FrameView::new(call_stack.previous_frame_ptr(), call_stack, null_mut()), jvm },
        }*/
        todo!()
    }

    pub fn previous_frame(&self) -> Option<StackEntryRef<'gc_life, '_>> {
        match self {
            InterpreterStateGuard::RemoteInterpreterState { .. } => {
                todo!()
            }
            InterpreterStateGuard::LocalInterpreterState { int_state, jvm, .. } => {
                let current = int_state.previous_frame_ref();
                let prev_method_id = int_state.previous_frame_ref().ir_method_id();
                if prev_method_id.is_none() || prev_method_id.unwrap() == jvm.java_vm_state.ir.get_top_level_return_ir_method_id() {
                    return None;
                }
                let (prev_method_id, prev_pc) = jvm.java_vm_state.lookup_ip(current.prev_rip())?;
                let prev_ir_frame = int_state.previous_frame_ref();
                // assert_eq!(prev_ir_frame.method_id(), Some(prev_method_id));
                Some(StackEntryRef {
                    frame_view: RuntimeJavaStackFrameRef {
                        ir_ref: prev_ir_frame,
                        jvm,
                    },
                    pc: Some(prev_pc),
                })
            }
        }
    }

    pub fn set_throw<'irrelevant_for_now>(&mut self, val: Option<NewJVObject<'gc_life,'irrelevant_for_now>>) {
        /*match self.int_state.as_mut() {
            None => {
                let mut guard = self.thread.interpreter_state.write().unwrap();
                match guard.deref_mut() {
                    /*InterpreterState::LegacyInterpreter { throw, .. } => {
                        *throw = val;
                    }*/
                    InterpreterState::Jit { .. } => todo!(),
                }
            }
            Some(val_mut) => {
                match val_mut.deref_mut() {
                    /*InterpreterState::LegacyInterpreter { throw, .. } => {
                        *throw = val;
                    }*/
                    InterpreterState::Jit { jvm, call_stack } => call_stack.set_throw(unsafe { to_object(val) }),
                }
            }
        }*/
        todo!()
    }

    pub fn function_return(&mut self) -> bool {
        /*let int_state = self.int_state.as_mut().unwrap();
        match int_state.deref_mut() {
            /*InterpreterState::LegacyInterpreter { function_return, .. } => {
                *function_return
            }*/
            InterpreterState::Jit { call_stack, .. } => unsafe { call_stack.saved_registers().status_register.as_mut() }.unwrap().function_return,
        }*/
        todo!()
    }

    pub fn set_function_return(&mut self, to: bool) {
        /*let int_state = self.int_state.as_mut().unwrap();
        match int_state.deref_mut() {
            /*InterpreterState::LegacyInterpreter { function_return, .. } => {
                *function_return = to;
            }*/
            InterpreterState::Jit { call_stack, .. } => unsafe {
                call_stack.saved_registers().status_register.as_mut().unwrap().function_return = to;
            },
        }*/
        todo!()
    }

    pub fn throw(&self) -> Option<GcManagedObject<'gc_life>> {
        None
        /*match self.int_state.as_ref() {
            None => {
                match self.thread.interpreter_state.read().unwrap().deref() {
                    /*InterpreterState::LegacyInterpreter { throw, .. } => {
                        throw.clone()
                    }*/
                    InterpreterState::Jit { .. } => {
                        todo!()
                    }
                }
            }
            Some(int_state) => match int_state.deref() {
                /*InterpreterState::LegacyInterpreter { throw, .. } => {
                    throw.clone()
                }*/
                InterpreterState::Jit { call_stack, jvm } => unsafe { from_object(jvm, call_stack.throw()) },
            },
        }*/
    }

    pub fn push_frame<'k>(&mut self, frame: StackEntryPush<'gc_life,'k>) -> FramePushGuard {
        let frame_push_guard = match self {
            InterpreterStateGuard::RemoteInterpreterState { .. } => {
                todo!()
            }
            InterpreterStateGuard::LocalInterpreterState { int_state, jvm, .. } => {
                let ir_vm_state = &jvm.java_vm_state.ir;
                let top_level_ir_method_id = ir_vm_state.get_top_level_return_ir_method_id();
                let top_level_exit_ptr = ir_vm_state.lookup_ir_method_id_pointer(top_level_ir_method_id);
                let ir_frame_push_guard = match frame {
                    StackEntryPush::Java { operand_stack, local_vars, method_id } => {
                        let ir_method_id = jvm.java_vm_state.lookup_method_ir_method_id(method_id);
                        let mut data = vec![];
                        for local_var in local_vars {
                            if let Some(Some(obj)) = local_var.try_unwrap_object_alloc(){
                                jvm.gc.memory_region.lock().unwrap().find_object_allocated_type(obj.handle.ptr);
                            }
                            data.push(unsafe { local_var.to_native().as_u64 });
                        }
                        for jv in operand_stack {
                            data.push(unsafe { jv.to_native().as_u64 });
                        }
                        let wrapped_method_id = OpaqueFrameIdOrMethodID::Method { method_id: method_id as u64 };
                        int_state.push_frame(top_level_exit_ptr, Some(ir_method_id), wrapped_method_id.to_native(), data.as_slice(), ir_vm_state)
                    }
                    StackEntryPush::Native { method_id, native_local_refs, local_vars, operand_stack } => {
                        let (rc, _) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
                        let loader = jvm.classes.read().unwrap().get_initiating_loader(&rc);
                        let native_frame_info = NativeFrameInfo {
                            method_id,
                            loader,
                            native_local_refs,
                            local_vars: local_vars.iter().map(|njv|njv.to_native()).collect(),
                            operand_stack: operand_stack.iter().map(|njv|njv.to_native()).collect(),
                        };
                        let raw_frame_info_pointer = Box::into_raw(box native_frame_info);
                        let wrapped_method_id = OpaqueFrameIdOrMethodID::Method { method_id: method_id as u64 };
                        let data = [raw_frame_info_pointer as *const c_void as usize as u64];
                        int_state.push_frame(top_level_exit_ptr, None, wrapped_method_id.to_native(), data.as_slice(), ir_vm_state)
                    }
                    StackEntryPush::Opaque { opaque_id, native_local_refs } => {
                        let wrapped_opaque_id = OpaqueFrameIdOrMethodID::Opaque { opaque_id };
                        let opaque_frame_info = OpaqueFrameInfo { native_local_refs, operand_stack: vec![] };
                        let raw_frame_info_pointer = Box::into_raw(box opaque_frame_info);
                        let data = [raw_frame_info_pointer as *const c_void as usize as u64];
                        int_state.push_frame(top_level_exit_ptr, None, wrapped_opaque_id.to_native(), data.as_slice(), ir_vm_state)
                    }
                };
                FramePushGuard {
                    _correctly_exited: false,
                    prev_stack_location: JavaStackPosition::Top,
                    ir_frame_push_guard,
                }
            }
        };
        frame_push_guard
    }

    pub fn pop_frame(&mut self, jvm: &'gc_life JVMState<'gc_life>, mut frame_push_guard: FramePushGuard, was_exception: bool) {
        frame_push_guard._correctly_exited = true;
        if self.current_frame().is_opaque() {
            unsafe { drop(Box::from_raw(self.current_frame().opaque_frame_ptr())) }
        }
        if self.current_frame().is_native_method() {
            unsafe { drop(Box::from_raw(self.current_frame().native_frame_ptr())) }
        }
        match self {
            InterpreterStateGuard::RemoteInterpreterState { .. } => todo!(),
            InterpreterStateGuard::LocalInterpreterState { int_state, .. } => {
                int_state.pop_frame(frame_push_guard.ir_frame_push_guard);
            }
        }
    }

    pub fn call_stack_depth(&self) -> usize {
        /*match self.int_state.as_ref().unwrap().deref() {
            /*InterpreterState::LegacyInterpreter { call_stack, .. } => call_stack.len(),*/
            InterpreterState::Jit { call_stack, .. } => unsafe { call_stack.call_stack_depth() },
        }*/
        todo!()
    }

    pub fn set_current_pc(&mut self, new_pc: ByteCodeOffset) {
        todo!()
        // self.current_frame_mut().set_pc(new_pc);
    }

    pub fn current_pc(&self) -> ByteCodeOffset {
        match self {
            InterpreterStateGuard::RemoteInterpreterState { .. } => todo!(),
            InterpreterStateGuard::LocalInterpreterState { current_exited_pc, .. } => {
                current_exited_pc.unwrap()
            }
        }
    }

    pub fn set_current_pc_offset(&mut self, new_offset: i32) {
        self.current_frame_mut().set_pc_offset(new_offset)
    }

    pub fn current_pc_offset(&self) -> i32 {
        self.current_frame().pc_offset()
    }

    pub fn current_method_i(&self, jvm: &'gc_life JVMState<'gc_life>) -> CPIndex {
        self.current_frame().method_i(jvm)
    }

    pub fn frame_iter(&self) -> JavaFrameIterRef<'_, '_, 'gc_life, ()> {
        match self {
            InterpreterStateGuard::RemoteInterpreterState { .. } => todo!(),
            InterpreterStateGuard::LocalInterpreterState { int_state, jvm, .. } => {
                JavaFrameIterRef {
                    ir: int_state.frame_iter(&jvm.java_vm_state.ir),
                    jvm,
                    current_pc: Some(self.current_pc()),
                }
            }
        }
    }


    pub fn debug_print_stack_trace(&self, jvm: &'gc_life JVMState<'gc_life>) {
        let full = false;
        let pc = self.current_frame().pc;
        let iter = self.frame_iter();
        for (i, stack_entry) in iter.enumerate() {
            if stack_entry.try_class_pointer(jvm).is_some()
            {
                let type_ = stack_entry.class_pointer(jvm).view().type_();
                let view = stack_entry.class_pointer(jvm).view();
                let method_view = view.method_view_i(stack_entry.method_i(jvm));
                let meth_name = method_view.name().0.to_str(&jvm.string_pool);
                let method_desc_str = method_view.desc_str().to_str(&jvm.string_pool);
                if method_view.is_native() {
                    println!("{:?}.{} {} {}", type_, meth_name, method_desc_str, i)
                } else {
                    println!("{:?}.{} {} {} {} {} {:?}", type_.unwrap_class_type().0.to_str(&jvm.string_pool), meth_name, method_desc_str, i, stack_entry.loader(jvm), stack_entry.pc.map(|offset|offset.0 as i32).unwrap_or(-1),stack_entry.frame_view.ir_ref.frame_ptr());
                    if full {
                        if stack_entry.pc.is_some() {
                            dump_frame_contents_impl(jvm, stack_entry);
                        } else {
                            // stack_entry.ir_stack_entry_debug_print();
                        }
                    }
                }
            } else {
                println!("missing");
            }
        }
    }

    pub fn cloned_stack_snapshot(&self, jvm: &'gc_life JVMState<'gc_life>) -> Vec<StackEntry<'gc_life>> {
        todo!()
    }

    pub fn depth(&self) -> usize {
        /*match self.int_state.as_ref().unwrap().deref() {
            /*InterpreterState::LegacyInterpreter { call_stack, .. } => call_stack.len(),*/
            InterpreterState::Jit { .. } => todo!(),
        }*/
        todo!()
    }

    pub fn add_should_frame_pop_notify(&mut self, depth: usize) -> Result<(), AddFrameNotifyError> {
        /*let call_stack_depth = self.call_stack_depth();
        let int_state = self.int_state.as_mut().unwrap().deref_mut();
        if depth >= call_stack_depth {
            return Err(NothingAtDepth);
        }
        let entry: &StackEntry = &match int_state {
            /*InterpreterState::LegacyInterpreter { call_stack, .. } => {
                &call_stack[depth]
            }*/
            InterpreterState::Jit { .. } => todo!(),
        };
        if entry.is_native() || entry.try_class_pointer().is_none() {
            return Err(Opaque);
        }
        match int_state {
            /*InterpreterState::LegacyInterpreter { should_frame_pop_notify, .. } => {
                should_frame_pop_notify.insert(depth);
            }*/
            InterpreterState::Jit { .. } => todo!(),
        };
        Ok(())*/
        todo!()
    }

    pub fn frame_state_assert_save(&self) -> SavedAssertState {
        match self {
            InterpreterStateGuard::RemoteInterpreterState { .. } => todo!(),
            InterpreterStateGuard::LocalInterpreterState { int_state, .. } => {
                unsafe { self.frame_state_assert_save_from(int_state.current_rsp.add(size_of::<u64>())) }
            }
        }
    }

    pub fn frame_state_assert_save_from(&self, from_inclusive: *const c_void) -> SavedAssertState {
        match self {
            InterpreterStateGuard::RemoteInterpreterState { .. } => todo!(),
            InterpreterStateGuard::LocalInterpreterState { int_state, thread, registered, jvm, current_exited_pc } => {
                let mmaped_top = int_state.owned_ir_stack.native.mmaped_top;
                let curent_rsp = int_state.current_rsp;
                let curent_rbp = int_state.current_rbp;
                let slice = unsafe { slice_from_raw_parts(from_inclusive as *const u64, mmaped_top.offset_from(from_inclusive).abs() as usize / size_of::<u64>() + 1)  };
                let data = unsafe { slice.as_ref() }.unwrap().iter().cloned().map(|elem| elem as usize as *const c_void).collect();
                SavedAssertState {
                    frame_pointer: curent_rbp,
                    stack_pointer: curent_rsp,
                    data,
                }
            }
        }
    }

    pub fn saved_assert_frame_from(&self, previous: SavedAssertState, from_inclusive: *const c_void) {
        let current = self.frame_state_assert_save_from(from_inclusive);
        // dbg!(&current.data);
        // dbg!(&previous.data);
        assert_eq!(current, previous);
    }

    pub fn saved_assert_frame_same(&self, previous: SavedAssertState) {
        match self {
            InterpreterStateGuard::RemoteInterpreterState { .. } => todo!(),
            InterpreterStateGuard::LocalInterpreterState { int_state, .. } => {
                let from = unsafe { int_state.current_rsp.add(size_of::<u64>()) };
                self.saved_assert_frame_from(previous, from)
            }
        }
    }
}

#[must_use]
#[derive(Eq, PartialEq, Debug)]
pub struct SavedAssertState {
    frame_pointer: *const c_void,
    stack_pointer: *const c_void,
    data: Vec<*const c_void>,
}

pub struct JavaFrameIterRef<'l, 'h, 'vm_life, ExtraData: 'vm_life> {
    ir: IRFrameIterRef<'l, 'h, 'vm_life, ExtraData>,
    jvm: &'vm_life JVMState<'vm_life>,
    current_pc: Option<ByteCodeOffset>,
}

impl<'l, 'h, 'vm_life, ExtraData> Iterator for JavaFrameIterRef<'l, 'h, 'vm_life, ExtraData> {
    type Item = StackEntryRef<'vm_life, 'l>;

    fn next(&mut self) -> Option<Self::Item> {
        self.ir.next().map(|ir_frame_ref| {
            let prev_rip = ir_frame_ref.prev_rip();
            let res = StackEntryRef {
                frame_view: RuntimeJavaStackFrameRef {
                    ir_ref: ir_frame_ref,
                    jvm: self.jvm,
                },
                pc: self.current_pc,
            };
            match self.jvm.java_vm_state.lookup_ip(prev_rip) {
                Some((_, new_pc)) => {
                    self.current_pc = Some(new_pc);
                }
                None => {
                    self.current_pc = None
                }
            };
            res
        })
    }
}

#[derive(Debug, Clone)]
pub struct OpaqueFrameInfo<'gc_life> {
    pub native_local_refs: Vec<HashSet<jobject>>,
    pub operand_stack: Vec<JavaValue<'gc_life>>,
}

#[derive(Debug, Clone)]
pub struct NativeFrameInfo<'gc_life> {
    pub method_id: usize,
    pub loader: LoaderName,
    pub native_local_refs: Vec<HashSet<jobject>>,
    pub local_vars: Vec<NativeJavaValue<'gc_life>>,
    pub operand_stack: Vec<NativeJavaValue<'gc_life>>,
}

// fn compatible_with_type(jv: &JavaValue, type_: &VType) -> bool {
//     match type_ {
//         VType::DoubleType => {
//             jv.unwrap_double();
//             true
//         }
//         VType::FloatType => {
//             jv.unwrap_float();
//             true
//         }
//         VType::IntType => {
//             jv.unwrap_int();
//             true
//         }
//         VType::LongType => {
//             jv.unwrap_long();
//             true
//         }
//         VType::Class(ClassWithLoader { class_name, .. }) => {
//             match jv.unwrap_object() {
//                 None => true,
//                 Some(obj) => {
//                     true//todo need more granular
//                     // obj.unwrap_normal_object().class_pointer.ptypeview().unwrap_class_type() == class_name.clone()
//                 }
//             }
//         }
//         VType::ArrayReferenceType(array_ref) => {
//             if jv.unwrap_object().is_none() {
//                 return true;
//             }
//             let elem_type = jv.unwrap_array().elem_type.clone();
//             match &elem_type {
//                 PTypeView::ByteType => array_ref == &PTypeView::ByteType,
//                 PTypeView::CharType => array_ref == &PTypeView::CharType,
//                 PTypeView::DoubleType => todo!(),
//                 PTypeView::FloatType => todo!(),
//                 PTypeView::IntType => array_ref == &PTypeView::IntType,
//                 PTypeView::LongType => array_ref == &PTypeView::LongType,
//                 PTypeView::Ref(ref_) => {
//                     match ref_ {
//                         ReferenceTypeView::Class(class_) => {
//                             true//todo need more granular.
//                             // &PTypeView::Ref(ReferenceTypeView::Class(class_.clone())) == array_ref
//                         }
//                         ReferenceTypeView::Array(array_) => {
//                             true//todo need more granular
//                         }
//                     }
//                 }
//                 PTypeView::ShortType => todo!(),
//                 PTypeView::BooleanType => array_ref == &PTypeView::BooleanType,
//                 PTypeView::VoidType => todo!(),
//                 PTypeView::TopType => todo!(),
//                 PTypeView::NullType => todo!(),
//                 PTypeView::Uninitialized(_) => todo!(),
//                 PTypeView::UninitializedThis => todo!(),
//                 PTypeView::UninitializedThisOrClass(_) => todo!()
//             }
//         }
//         VType::VoidType => todo!(),
//         VType::TopType => {
//             match jv {
//                 JavaValue::Top => true,
//                 _ => true
//             }
//         }
//         VType::NullType => {
//             jv.unwrap_object();
//             true
//         }
//         VType::Uninitialized(_) => {
//             jv.unwrap_object_nonnull();
//             true
//         }
//         VType::UninitializedThis => {
//             jv.unwrap_object_nonnull();
//             true
//         }
//         VType::UninitializedThisOrClass(_) => {
//             jv.unwrap_object_nonnull();
//             true
//         }
//         VType::TwoWord => todo!(),
//         VType::OneWord => todo!(),
//         VType::Reference => todo!(),
//         VType::UninitializedEmpty => todo!(),
//     }
// }
//
// fn remove_tops(stack_map: &OperandStack) -> OperandStack {
//     //todo this is jank, should be idiomatic way to do this
//     let mut expecting_top = false;
//
//     let mut data = stack_map.data.iter().rev().flat_map(|cur| {
//         if expecting_top {
//             assert_eq!(cur, &VType::TopType);
//             expecting_top = false;
//             return None;
//         }
//         if &VType::LongType == cur || &VType::DoubleType == cur {
//             expecting_top = true;
//         }
//         Some(cur.clone())
//     }).collect::<VecDeque<_>>();
//     data = data.into_iter().rev().collect();
//     assert!(!expecting_top);
//     OperandStack {
//         data
//     }
// }

pub enum AddFrameNotifyError {
    Opaque,
    NothingAtDepth,
}

#[must_use = "Must handle frame push guard. "]
pub struct FramePushGuard {
    _correctly_exited: bool,
    prev_stack_location: JavaStackPosition,
    pub ir_frame_push_guard: IRPushFrameGuard,
}

/*impl Default for FramePushGuard {
    fn default() -> Self {
        FramePushGuard { _correctly_exited: false, prev_stack_location: todo!(), ir_frame_push_guard: () }
    }
}*/

#[derive(Debug)]
pub struct SuspendedStatus {
    pub suspended: std::sync::Mutex<bool>,
    pub suspend_condvar: std::sync::Condvar,
}

impl Default for SuspendedStatus {
    fn default() -> Self {
        Self { suspended: std::sync::Mutex::new(false), suspend_condvar: Default::default() }
    }
}