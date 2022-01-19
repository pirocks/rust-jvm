use std::arch::x86_64::_mm256_testc_pd;
use std::cell::RefCell;
use std::collections::HashSet;
use std::ffi::c_void;
use std::mem::transmute;
use std::ops::{Deref, DerefMut};
use std::ptr::null_mut;
use std::sync::{Arc, MutexGuard, RwLockWriteGuard};

use iced_x86::CC_b::c;
use itertools::Itertools;

use another_jit_vm_ir::ir_stack::{IRPushFrameGuard, IRStackMut, OwnedIRStack};
use another_jit_vm_ir::IRMethodID;
use classfile_view::view::{ClassView, HasAccessFlags};
use gc_memory_layout_common::FramePointerOffset;
use jvmti_jni_bindings::{jobject, jvalue};
use rust_jvm_common::classfile::CPIndex;
use rust_jvm_common::classfile::InstructionInfo::ireturn;
use rust_jvm_common::ByteCodeOffset;
use rust_jvm_common::loading::LoaderName;
use rust_jvm_common::runtime_type::RuntimeType;

use crate::interpreter_state::AddFrameNotifyError::{NothingAtDepth, Opaque};
use crate::ir_to_java_layer::java_stack::{JavaStackPosition, OpaqueFrameIdOrMethodID, OwnedJavaStack, RuntimeJavaStackFrameMut, RuntimeJavaStackFrameRef};
use crate::java_values::{GcManagedObject, JavaValue};
use crate::jit::MethodResolver;
use crate::jit_common::java_stack::{JavaStack, JavaStatus};
use crate::jvm_state::JVMState;
use crate::rust_jni::native_util::{from_object, to_object};
use crate::stack_entry::{FrameView, NonNativeFrameData, OpaqueFrameOptional, StackEntry, StackEntryMut, StackEntryRef, StackIter};
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
        current_exited_pc: Option<ByteCodeOffset>
    },
}

thread_local! {
pub static CURRENT_INT_STATE_GUARD_VALID :RefCell<bool> = RefCell::new(false);
}

thread_local! {
pub static CURRENT_INT_STATE_GUARD :RefCell<Option<*mut InterpreterStateGuard<'static,'static>>> = RefCell::new(None);
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

    pub fn register_interpreter_state_guard(&mut self, jvm: &'gc_life JVMState<'gc_life>) {
        let ptr = unsafe { transmute::<_, *mut InterpreterStateGuard<'static, 'static>>(self as *mut InterpreterStateGuard<'gc_life, '_>) };
        jvm.thread_state.int_state_guard.with(|refcell| refcell.replace(ptr.into()));
        jvm.thread_state.int_state_guard_valid.with(|refcell| refcell.replace(true));
        match self {
            InterpreterStateGuard::RemoteInterpreterState { registered, .. } => {
                *registered = true;
            }
            InterpreterStateGuard::LocalInterpreterState { registered, .. } => {
                *registered = true;
            }
        }
        assert!(self.thread().is_alive());
    }

    pub fn new(jvm: &'gc_life JVMState<'gc_life>, thread: Arc<JavaThread<'gc_life>>, int_state: MutexGuard<'interpreter_guard, InterpreterState<'gc_life>>) -> InterpreterStateGuard<'gc_life, 'interpreter_guard> {
        jvm.thread_state.int_state_guard_valid.with(|refcell| refcell.replace(false));
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
            InterpreterStateGuard::LocalInterpreterState { int_state, jvm, .. } => {
                return StackEntryRef {
                    frame_view: RuntimeJavaStackFrameRef {
                        ir_ref: int_state.current_frame_ref(),
                        jvm,
                    }
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

    pub fn previous_frame(&self) -> StackEntryRef<'gc_life, '_> {
        /*match self.int_state.as_ref().unwrap().deref() {
            /*InterpreterState::LegacyInterpreter { call_stack, .. } => {
                let len = call_stack.len();
                StackEntryRef::LegacyInterpreter { entry: &call_stack[len - 2] }
            }*/
            InterpreterState::Jit { call_stack, jvm } => StackEntryRef::Jit { frame_view: FrameView::new(call_stack.previous_frame_ptr(), call_stack, null_mut()) },
        }*/
        todo!()
    }

    pub fn set_throw(&mut self, val: Option<GcManagedObject<'gc_life>>) {
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

    pub fn push_frame(&mut self, frame: StackEntry<'gc_life>) -> FramePushGuard {
        let frame_push_guard = match self {
            InterpreterStateGuard::RemoteInterpreterState { .. } => {
                todo!()
            }
            InterpreterStateGuard::LocalInterpreterState { int_state, jvm, .. } => {
                let ir_vm_state = &jvm.java_vm_state.ir;
                let top_level_ir_method_id = ir_vm_state.get_top_level_return_ir_method_id();
                let top_level_exit_ptr = ir_vm_state.lookup_ir_method_id_pointer(top_level_ir_method_id);
                let ir_frame_push_guard = match frame {
                    StackEntry::Java { operand_stack, local_vars, method_id } => {
                        let ir_method_id = jvm.java_vm_state.lookup_method_ir_method_id(method_id);
                        let mut data = vec![];
                        for local_var in local_vars {
                            data.push(unsafe { local_var.to_native().as_u64 });
                        }
                        for jv in operand_stack {
                            data.push(unsafe { jv.to_native().as_u64 });
                        }
                        let wrapped_method_id = OpaqueFrameIdOrMethodID::Method { method_id: method_id as u64 };
                        int_state.push_frame(top_level_exit_ptr, Some(ir_method_id), wrapped_method_id.to_native(), data.as_slice(), ir_vm_state)
                    }
                    StackEntry::Native { method_id, native_local_refs, local_vars, operand_stack } => {
                        let (rc, _) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
                        let loader = jvm.classes.read().unwrap().get_initiating_loader(&rc);
                        let native_frame_info = NativeFrameInfo {
                            method_id,
                            loader,
                            native_local_refs,
                            local_vars,
                            operand_stack,
                        };
                        let raw_frame_info_pointer = Box::into_raw(box native_frame_info);
                        let wrapped_method_id = OpaqueFrameIdOrMethodID::Method { method_id: method_id as u64 };
                        let data = [raw_frame_info_pointer as *const c_void as usize as u64];
                        int_state.push_frame(top_level_exit_ptr, None, wrapped_method_id.to_native(), data.as_slice(), ir_vm_state)
                    }
                    StackEntry::Opaque { opaque_id, native_local_refs } => {
                        let wrapped_opaque_id = OpaqueFrameIdOrMethodID::Opaque { opaque_id };
                        let opaque_frame_info = OpaqueFrameInfo{ native_local_refs, operand_stack: vec![] };
                        let raw_frame_info_pointer = Box::into_raw(box opaque_frame_info);
                        let data = [raw_frame_info_pointer as *const c_void as usize as u64];
                        int_state.push_frame(top_level_exit_ptr, None, wrapped_opaque_id.to_native(),data.as_slice(),ir_vm_state)
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
        if self.current_frame().is_opaque(){
            unsafe { drop(Box::from_raw(self.current_frame().opaque_frame_ptr())) }
        }
        if self.current_frame().is_native_method(){
            unsafe { drop(Box::from_raw(self.current_frame().native_frame_ptr())) }
        }
        match self{
            InterpreterStateGuard::RemoteInterpreterState { .. } => todo!(),
            InterpreterStateGuard::LocalInterpreterState { int_state,.. } => {
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
        todo!()
        /*self.current_frame().pc(todo!())*/
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

    pub fn debug_print_stack_trace(&self, jvm: &'gc_life JVMState<'gc_life>) {
        /*let iter = match self.int_state.as_ref().unwrap().deref() {
            /*InterpreterState::LegacyInterpreter { call_stack, .. } => {
                Either::Left(call_stack.iter().cloned().enumerate().rev())
            }*/
            InterpreterState::Jit { call_stack, .. } => StackIter::new(jvm, call_stack).into_iter().enumerate(),
        };
        for (i, stack_entry) in iter {
            if stack_entry.try_method_i().is_some()
            /*&& stack_entry.method_i() > 0*/
            {
                let type_ = stack_entry.class_pointer().view().type_();
                let view = stack_entry.class_pointer().view();
                let method_view = view.method_view_i(stack_entry.method_i());
                let meth_name = method_view.name().0.to_str(&jvm.string_pool);
                let method_desc_str = method_view.desc_str().to_str(&jvm.string_pool);
                if method_view.is_native() {
                    println!("{:?}.{} {} {}", type_, meth_name, method_desc_str, i)
                } else {
                    println!("{:?}.{} {} {} pc: {} {}", type_.unwrap_class_type().0.to_str(&jvm.string_pool), meth_name, method_desc_str, i, stack_entry.pc(), stack_entry.loader())
                }
            } else {
                println!("missing");
            }
        }*/
        todo!()
    }

    pub fn cloned_stack_snapshot(&self, jvm: &'gc_life JVMState<'gc_life>) -> Vec<StackEntry<'gc_life>> {
        /*match self.int_state.as_ref().unwrap().deref() {
            /*InterpreterState::LegacyInterpreter { call_stack, .. } => call_stack.to_vec(),*/
            InterpreterState::Jit { call_stack, .. } => StackIter::new(jvm, call_stack).collect_vec().into_iter().rev().collect_vec(),
        }*/
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
}

#[derive(Debug, Clone)]
pub struct OpaqueFrameInfo<'gc_life>{
    pub native_local_refs: Vec<HashSet<jobject>>,
    pub operand_stack: Vec<JavaValue<'gc_life>>,
}

#[derive(Debug, Clone)]
pub struct NativeFrameInfo<'gc_life> {
    pub method_id: usize,
    pub loader: LoaderName,
    pub native_local_refs: Vec<HashSet<jobject>>,
    pub local_vars: Vec<JavaValue<'gc_life>>,
    pub operand_stack: Vec<JavaValue<'gc_life>>,
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
    ir_frame_push_guard: IRPushFrameGuard,
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