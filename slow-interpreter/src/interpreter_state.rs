use std::cell::RefCell;
use std::collections::HashSet;
use std::ffi::c_void;
use std::mem::{size_of, transmute};
use std::ptr::{NonNull, slice_from_raw_parts};
use std::sync::{Arc, MutexGuard};

use itertools::Itertools;
use nonnull_const::NonNullConst;
use another_jit_vm::stack::CannotAllocateStack;

use another_jit_vm_ir::ir_stack::{IRFrameIterRef, IRPushFrameGuard, IRStackMut};
use classfile_view::view::{ClassView, HasAccessFlags};
use jvmti_jni_bindings::jobject;
use rust_jvm_common::{ByteCodeOffset, NativeJavaValue};
use rust_jvm_common::classfile::CPIndex;
use rust_jvm_common::loading::LoaderName;
use rust_jvm_common::runtime_type::RuntimeType;

use crate::{AllocatedHandle, JavaValueCommon, MethodResolverImpl};
use crate::ir_to_java_layer::java_stack::{JavaStackPosition, OpaqueFrameIdOrMethodID, OwnedJavaStack, RuntimeJavaStackFrameMut, RuntimeJavaStackFrameRef};
use crate::java_values::JavaValue;
use crate::jvm_state::JVMState;
use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
use crate::stack_entry::{JavaFramePush, NativeFramePush, OpaqueFramePush, StackEntry, StackEntryMut, StackEntryPush, StackEntryRef};
use crate::threading::JavaThread;

pub struct InterpreterState<'gc> {
    pub call_stack: OwnedJavaStack<'gc>,
    jvm: &'gc JVMState<'gc>,
    pub current_stack_position: JavaStackPosition,
}

impl<'gc> InterpreterState<'gc> {
    pub(crate) fn new(jvm: &'gc JVMState<'gc>) -> Result<Self, CannotAllocateStack> {
        Ok(InterpreterState {
            call_stack: OwnedJavaStack::new(&jvm.java_vm_state)?,
            jvm,
            current_stack_position: JavaStackPosition::Top,
        })
    }
}

pub enum InterpreterStateGuard<'vm, 'l> {
    //todo these internals need to change to reflect that we need to halt thread to get current rbp.
    RemoteInterpreterState {
        int_state: Option<MutexGuard<'l, InterpreterState<'vm>>>,
        thread: Arc<JavaThread<'vm>>,
        registered: bool,
        jvm: &'vm JVMState<'vm>,
    },
    LocalInterpreterState {
        int_state: IRStackMut<'l>,
        thread: Arc<JavaThread<'vm>>,
        registered: bool,
        jvm: &'vm JVMState<'vm>,
        current_exited_pc: Option<ByteCodeOffset>,
        throw: Option<AllocatedHandle<'vm>>,
    },
}

thread_local! {
pub static CURRENT_INT_STATE_GUARD_VALID :RefCell<bool> = RefCell::new(false);
}

thread_local! {
pub static CURRENT_INT_STATE_GUARD :RefCell<Option<*mut InterpreterStateGuard<'static,'static>>> = RefCell::new(None);
}

#[must_use]
pub struct OldInterpreterState {
    pub old: Option<*mut InterpreterStateGuard<'static, 'static>>,
}

impl<'gc, 'interpreter_guard> InterpreterStateGuard<'gc, 'interpreter_guard> {
    pub fn registered(&self) -> bool {
        match self {
            InterpreterStateGuard::RemoteInterpreterState { registered, .. } => *registered,
            InterpreterStateGuard::LocalInterpreterState { registered, .. } => *registered
        }
    }

    pub fn thread(&self) -> Arc<JavaThread<'gc>> {
        match self {
            InterpreterStateGuard::RemoteInterpreterState { thread, .. } => thread.clone(),
            InterpreterStateGuard::LocalInterpreterState { thread, .. } => thread.clone()
        }
    }

    pub fn register_interpreter_state_guard(&mut self, jvm: &'gc JVMState<'gc>) -> OldInterpreterState {
        let ptr = unsafe { transmute::<_, *mut InterpreterStateGuard<'static, 'static>>(self as *mut InterpreterStateGuard<'gc, '_>) };
        let old = jvm.thread_state.int_state_guard.with(|elem| elem.borrow().clone());
        jvm.thread_state.int_state_guard.with(|elem| elem.replace(ptr.into()));
        jvm.thread_state.int_state_guard_valid.with(|elem| elem.replace(true));
        match self {
            InterpreterStateGuard::RemoteInterpreterState { registered, .. } => {
                *registered = true;
            }
            InterpreterStateGuard::LocalInterpreterState { registered, .. } => {
                *registered = true;
            }
        }
        assert!(self.thread().is_alive());
        OldInterpreterState {
            old
        }
    }

    pub fn deregister_int_state(&mut self, jvm: &'gc JVMState<'gc>, old: OldInterpreterState) {
        jvm.thread_state.int_state_guard.with(|elem| elem.replace(old.old));
    }

    pub fn new(jvm: &'gc JVMState<'gc>, thread: Arc<JavaThread<'gc>>, int_state: MutexGuard<'interpreter_guard, InterpreterState<'gc>>) -> InterpreterStateGuard<'gc, 'interpreter_guard> {
        jvm.thread_state.int_state_guard_valid.with(|valid| valid.replace(false));
        Self::LocalInterpreterState {
            int_state: todo!(),
            thread: thread.clone(),
            registered: true,
            jvm,
            current_exited_pc: None,
            throw: None,
        }
    }

    pub fn current_loader(&self, jvm: &'gc JVMState<'gc>) -> LoaderName {
        self.current_frame().loader(jvm)
    }

    pub fn current_class_view(&self, jvm: &'gc JVMState<'gc>) -> Arc<dyn ClassView> {
        self.current_frame().try_class_pointer(jvm).unwrap().view()
    }

    pub fn current_frame(&'_ self) -> StackEntryRef<'gc, '_> {
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

    pub fn current_frame_mut(&'_ mut self) -> StackEntryMut<'gc, '_> {
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

    pub fn push_current_operand_stack(&mut self, jval: JavaValue<'gc>) {
        self.current_frame_mut().push(jval)
    }

    pub fn pop_current_operand_stack(&mut self, expected_type: Option<RuntimeType>) -> JavaValue<'gc> {
        self.current_frame_mut().operand_stack_mut().pop(expected_type).unwrap()
    }

    pub fn previous_frame(&self) -> Option<StackEntryRef<'gc, '_>> {
        match self {
            InterpreterStateGuard::RemoteInterpreterState { .. } => {
                todo!()
            }
            InterpreterStateGuard::LocalInterpreterState { int_state, jvm, .. } => {
                // let current = int_state.previous_frame_ref();
                // let prev_ir_method_id = int_state.previous_frame_ref().ir_method_id();
                // int_state.previous_frame_ref().method_id()?;
                // if prev_ir_method_id.is_none() || prev_ir_method_id.unwrap() == jvm.java_vm_state.ir.get_top_level_return_ir_method_id() {
                //     return None;
                // }
                // let (prev_method_id, prev_pc) = jvm.java_vm_state.lookup_ip(current.prev_rip())?;
                let prev_ir_frame = int_state.previous_frame_ref();
                // assert_eq!(prev_ir_frame.method_id(), Some(prev_method_id));
                Some(StackEntryRef {
                    frame_view: RuntimeJavaStackFrameRef {
                        ir_ref: prev_ir_frame,
                        jvm,
                    },
                    pc: None,
                })
            }
        }
    }

    pub fn set_throw<'irrelevant_for_now>(&mut self, val: Option<AllocatedHandle<'gc>>) {
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
        match self {
            InterpreterStateGuard::RemoteInterpreterState { .. } => todo!(),
            InterpreterStateGuard::LocalInterpreterState { throw, .. } => {
                *throw = val;
            }
        }
    }

    pub fn throw(&self) -> Option<&'_ AllocatedNormalObjectHandle<'gc>> {
        match self {
            InterpreterStateGuard::RemoteInterpreterState { .. } => todo!(),
            InterpreterStateGuard::LocalInterpreterState { throw, .. } => {
                throw.as_ref().map(|handle| handle.unwrap_normal_object_ref())
            }
        }
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

    pub fn push_frame<'k>(&mut self, frame: StackEntryPush<'gc, 'k>) -> FramePushGuard {
        let frame_push_guard = match self {
            InterpreterStateGuard::RemoteInterpreterState { .. } => {
                todo!()
            }
            InterpreterStateGuard::LocalInterpreterState { int_state, jvm, .. } => {
                let ir_vm_state = &jvm.java_vm_state.ir;
                let top_level_ir_method_id = ir_vm_state.get_top_level_return_ir_method_id();
                let top_level_exit_ptr = ir_vm_state.lookup_ir_method_id_pointer(top_level_ir_method_id);
                let ir_frame_push_guard = match frame {
                    StackEntryPush::Java(JavaFramePush { operand_stack, local_vars, method_id }) => {
                        assert_eq!(jvm.num_local_var_slots(method_id) as usize, local_vars.len());
                        let ir_method_id = jvm.java_vm_state.try_lookup_method_ir_method_id(method_id);
                        let mut data = vec![];
                        for local_var in local_vars {
                            if let Some(Some(obj)) = local_var.try_unwrap_object_alloc() {
                                jvm.gc.memory_region.lock().unwrap().find_object_allocated_type(obj.ptr());
                            }
                            data.push(unsafe { local_var.to_native().as_u64 });
                        }
                        for jv in operand_stack {
                            data.push(unsafe { jv.to_native().as_u64 });
                        }
                        let wrapped_method_id = OpaqueFrameIdOrMethodID::Method { method_id: method_id as u64 };
                        int_state.push_frame(top_level_exit_ptr.as_ptr(), ir_method_id, wrapped_method_id.to_native(), data.as_slice(), ir_vm_state)
                    }
                    StackEntryPush::Native(NativeFramePush { method_id, native_local_refs, local_vars, operand_stack }) => {
                        jvm.java_vm_state.add_method_if_needed(jvm, &MethodResolverImpl { jvm, loader: LoaderName::BootstrapLoader/*todo fix*/ }, method_id, false);
                        let ir_method_id = jvm.java_vm_state.lookup_method_ir_method_id(method_id);
                        let (rc, _) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
                        let loader = jvm.classes.read().unwrap().get_initiating_loader(&rc);
                        assert_eq!(jvm.num_local_vars_native(method_id) as usize, local_vars.len());
                        let native_frame_info = NativeFrameInfo {
                            method_id,
                            loader,
                            native_local_refs,
                            // local_vars: local_vars.iter().map(|njv|njv.to_native()).collect(),
                            operand_stack: operand_stack.iter().map(|njv| njv.to_native()).collect(),
                        };
                        let raw_frame_info_pointer = Box::into_raw(box native_frame_info);
                        let wrapped_method_id = OpaqueFrameIdOrMethodID::Method { method_id: method_id as u64 };
                        //todo use NativeStackframeMemoryLayout for this
                        let mut data = local_vars.iter().map(|local_var| unsafe { local_var.to_native().as_u64 }).collect_vec();
                        data.push(raw_frame_info_pointer as *const c_void as usize as u64);
                        int_state.push_frame(top_level_exit_ptr.as_ptr(), Some(ir_method_id), wrapped_method_id.to_native(), data.as_slice(), ir_vm_state)
                    }
                    StackEntryPush::Opaque(OpaqueFramePush { opaque_id, native_local_refs }) => {
                        let wrapped_opaque_id = OpaqueFrameIdOrMethodID::Opaque { opaque_id };
                        let opaque_frame_info = OpaqueFrameInfo { native_local_refs, operand_stack: vec![] };
                        let raw_frame_info_pointer = Box::into_raw(box opaque_frame_info);
                        let data = [raw_frame_info_pointer as *const c_void as usize as u64];
                        int_state.push_frame(top_level_exit_ptr.as_ptr(), None, wrapped_opaque_id.to_native(), data.as_slice(), ir_vm_state)
                    }
                };
                FramePushGuard {
                    _correctly_exited: false,
                    _prev_stack_location: JavaStackPosition::Top,
                    ir_frame_push_guard,
                }
            }
        };
        frame_push_guard
    }

    pub fn pop_frame(&mut self, jvm: &'gc JVMState<'gc>, mut frame_push_guard: FramePushGuard, was_exception: bool) {
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

    pub fn current_pc(&self) -> Option<ByteCodeOffset> {
        match self {
            InterpreterStateGuard::RemoteInterpreterState { .. } => todo!(),
            InterpreterStateGuard::LocalInterpreterState { current_exited_pc, .. } => {
                *current_exited_pc
            }
        }
    }

    pub fn set_current_pc(&mut self, pc: Option<ByteCodeOffset>) {
        match self {
            InterpreterStateGuard::RemoteInterpreterState { .. } => todo!(),
            InterpreterStateGuard::LocalInterpreterState { current_exited_pc, .. } => {
                *current_exited_pc = pc;
            }
        }
    }

    pub fn current_method_i(&self, jvm: &'gc JVMState<'gc>) -> CPIndex {
        self.current_frame().method_i(jvm)
    }

    pub fn frame_iter(&self) -> JavaFrameIterRef<'_, '_, 'gc, ()> {
        match self {
            InterpreterStateGuard::RemoteInterpreterState { .. } => todo!(),
            InterpreterStateGuard::LocalInterpreterState { int_state, jvm, .. } => {
                JavaFrameIterRef {
                    ir: int_state.frame_iter(&jvm.java_vm_state.ir),
                    jvm,
                    current_pc: self.current_pc(),
                }
            }
        }
    }


    pub fn debug_print_stack_trace(&self, jvm: &'gc JVMState<'gc>) {
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
                    let loader_name = stack_entry.loader(jvm);
                    let program_counter = stack_entry.pc.map(|offset| offset.0 as i32).unwrap_or(-1);
                    println!("{:?}.{} {} {} {} {}", type_.unwrap_class_type().0.to_str(&jvm.string_pool), meth_name, method_desc_str, i, loader_name, program_counter);
                    if full {
                        // if stack_entry.pc.is_some() {
                        // dump_frame_contents_impl(jvm, self);
                        // } else {
                        stack_entry.ir_stack_entry_debug_print();
                        // }
                    }
                }
            } else {
                println!("missing");
            }
        }
    }

    pub fn cloned_stack_snapshot(&self, jvm: &'gc JVMState<'gc>) -> Vec<StackEntry> {
        self.frame_iter().map(|frame| {
            if frame.is_opaque() {
                StackEntry::Opaque { opaque_id: OpaqueFrameIdOrMethodID::from_native(frame.frame_view.ir_ref.raw_method_id()).unwrap_opaque().unwrap() }
            } else if frame.is_native_method() {
                StackEntry::Native {
                    method_id: frame.frame_view.ir_ref.method_id().unwrap(),
                }
            } else {
                StackEntry::Java { method_id: frame.frame_view.ir_ref.method_id().unwrap() }
            }
        }).collect_vec()
    }

    pub fn frame_state_assert_save_from(&self, from_inclusive: NonNullConst<c_void>) -> SavedAssertState {
        match self {
            InterpreterStateGuard::RemoteInterpreterState { .. } => todo!(),
            InterpreterStateGuard::LocalInterpreterState { int_state, thread, registered, jvm, current_exited_pc, .. } => {
                let mmaped_top = int_state.owned_ir_stack.native.mmaped_top;
                let curent_rsp = int_state.current_rsp;
                let curent_rbp = int_state.current_rbp;
                let slice = unsafe { slice_from_raw_parts(from_inclusive.as_ptr() as *const u64, mmaped_top.as_ptr().offset_from(from_inclusive.as_ptr()).abs() as usize / size_of::<u64>() + 1) };
                let data = unsafe { slice.as_ref() }.unwrap().iter().cloned().map(|elem| elem as usize as *const c_void).collect();
                SavedAssertState {
                    frame_pointer: curent_rbp,
                    stack_pointer: curent_rsp,
                    data,
                }
            }
        }
    }

    pub fn saved_assert_frame_from(&self, previous: SavedAssertState, from_inclusive: NonNullConst<c_void>) {
        let current = self.frame_state_assert_save_from(from_inclusive);
        // dbg!(&current.data);
        // dbg!(&previous.data);
        assert_eq!(current, previous);
    }

    pub fn saved_assert_frame_same(&self, previous: SavedAssertState) {
        match self {
            InterpreterStateGuard::RemoteInterpreterState { .. } => todo!(),
            InterpreterStateGuard::LocalInterpreterState { int_state, .. } => {
                let from = unsafe { int_state.current_rsp.as_ptr().add(size_of::<u64>()) };
                self.saved_assert_frame_from(previous, NonNullConst::new(from).unwrap())
            }
        }
    }
}

#[must_use]
#[derive(Eq, PartialEq, Debug)]
pub struct SavedAssertState {
    frame_pointer: NonNull<c_void>,
    stack_pointer: NonNull<c_void>,
    data: Vec<*const c_void>,
}

pub struct JavaFrameIterRef<'l, 'h, 'vm, ExtraData: 'vm> {
    ir: IRFrameIterRef<'l, 'h, 'vm, ExtraData>,
    jvm: &'vm JVMState<'vm>,
    current_pc: Option<ByteCodeOffset>,
}

impl<'l, 'h, 'vm, ExtraData> Iterator for JavaFrameIterRef<'l, 'h, 'vm, ExtraData> {
    type Item = StackEntryRef<'vm, 'l>;

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
pub struct OpaqueFrameInfo<'gc> {
    pub native_local_refs: Vec<HashSet<jobject>>,
    pub operand_stack: Vec<JavaValue<'gc>>,
}

#[derive(Debug, Clone)]
pub struct NativeFrameInfo<'gc> {
    pub method_id: usize,
    pub loader: LoaderName,
    pub native_local_refs: Vec<HashSet<jobject>>,
    // pub local_vars: Vec<NativeJavaValue<'gc>>,
    pub operand_stack: Vec<NativeJavaValue<'gc>>,
}

pub enum AddFrameNotifyError {
    Opaque,
    NothingAtDepth,
}

#[must_use = "Must handle frame push guard. "]
pub struct FramePushGuard {
    _correctly_exited: bool,
    _prev_stack_location: JavaStackPosition,
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