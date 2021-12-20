use std::cell::RefCell;
use std::collections::HashSet;
use std::mem::transmute;
use std::ops::{Deref, DerefMut};
use std::ptr::null_mut;
use std::sync::{Arc, MutexGuard, RwLockWriteGuard};

use iced_x86::CC_b::c;
use itertools::Itertools;

use classfile_parser::code::InstructionTypeNum::new;
use classfile_view::view::{ClassView, HasAccessFlags};
use jvmti_jni_bindings::jvalue;
use rust_jvm_common::classfile::CPIndex;
use rust_jvm_common::loading::LoaderName;
use rust_jvm_common::runtime_type::RuntimeType;

use crate::gc_memory_layout_common::{FrameBackedStackframeMemoryLayout, FrameInfo, FramePointerOffset, FullyOpaqueFrame, NativeStackframeMemoryLayout};
use crate::interpreter_state::AddFrameNotifyError::{NothingAtDepth, Opaque};
use crate::ir_to_java_layer::java_stack::{JavaStackPosition, NativeFrameInfo, OpaqueFrameIdOrMethodID, OwnedJavaStack};
use crate::java_values::{GcManagedObject, JavaValue};
use crate::jit_common::java_stack::{JavaStack, JavaStatus};
use crate::jvm_state::JVMState;
use crate::rust_jni::native_util::{from_object, to_object};
use crate::stack_entry::{FrameView, NonNativeFrameData, OpaqueFrameOptional, StackEntry, StackEntryMut, StackEntryRef, StackIter};
use crate::threading::JavaThread;

pub struct InterpreterState<'gc_life> {
    call_stack: OwnedJavaStack<'gc_life>,
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

pub struct InterpreterStateGuard<'vm_life, 'l> {
    //todo these internals need to change to reflect that we need to halt thread to get current rbp.
    pub(crate) int_state: Option<MutexGuard<'l, InterpreterState<'vm_life>>>,
    pub(crate) thread: Arc<JavaThread<'vm_life>>,
    pub(crate) registered: bool,
    pub(crate) jvm: &'vm_life JVMState<'vm_life>,
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

    pub fn self_check(&self, jvm: &'gc_life JVMState<'gc_life>) {
        for stack_entry in self.cloned_stack_snapshot(jvm) {
            for jv in stack_entry.operand_stack.iter().chain(stack_entry.local_vars.iter()) {
                jv.self_check();
            }
        }
    }

    pub fn register_interpreter_state_guard(&mut self, jvm: &'gc_life JVMState<'gc_life>) {
        let ptr = unsafe { transmute::<_, *mut InterpreterStateGuard<'static, 'static>>(self as *mut InterpreterStateGuard<'gc_life, '_>) };
        jvm.thread_state.int_state_guard.with(|refcell| refcell.replace(ptr.into()));
        jvm.thread_state.int_state_guard_valid.with(|refcell| refcell.replace(true));
        self.registered = true;
        assert!(self.thread.is_alive());
    }

    pub fn new(jvm: &'gc_life JVMState<'gc_life>, thread: Arc<JavaThread<'gc_life>>, int_state: MutexGuard<'interpreter_guard, InterpreterState<'gc_life>>) -> InterpreterStateGuard<'gc_life, 'interpreter_guard> {
        jvm.thread_state.int_state_guard_valid.with(|refcell| refcell.replace(false));
        Self { int_state: Some(int_state), thread: thread.clone(), registered: true, jvm }
    }

    pub fn java_stack(&mut self) -> &mut OwnedJavaStack<'gc_life> {
        &mut self.int_state.as_mut().unwrap().call_stack
    }

    pub fn current_loader(&self, jvm: &'gc_life JVMState<'gc_life>) -> LoaderName {
        self.current_frame().loader(jvm)
    }

    pub fn current_class_view(&self, jvm: &'gc_life JVMState<'gc_life>) -> Arc<dyn ClassView> {
        self.current_frame().try_class_pointer(jvm).unwrap().view()
    }

    pub fn current_frame(&'_ self) -> StackEntryRef<'gc_life, '_> {
        let interpreter_state = self.int_state.as_ref().unwrap().deref();
        match interpreter_state {
            InterpreterState { call_stack, jvm, current_stack_position } => {
                let frame_at = call_stack.frame_at(*current_stack_position, jvm);
                StackEntryRef {
                    frame_view: frame_at
                }
            }
        }
    }

    pub fn current_frame_mut(&'k mut self) -> StackEntryMut<'gc_life, 'k> {
        /*let interpreter_state = self.int_state.as_mut().unwrap().deref_mut();
        match interpreter_state {
            /*InterpreterState::LegacyInterpreter { call_stack, .. } => {
                StackEntryMut::LegacyInterpreter { entry: call_stack.last_mut().unwrap() }
            }*/
            InterpreterState::Jit { call_stack, jvm } => StackEntryMut::Jit {
                frame_view: FrameView::new(call_stack.current_frame_ptr(), call_stack, call_stack.saved_registers.unwrap().instruction_pointer),
                jvm,
            },
        }*/
        todo!()
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
        let int_state = self.int_state.as_mut().unwrap().deref_mut();
        let InterpreterState { call_stack, jvm, current_stack_position } = int_state;
        let StackEntry { loader, opaque_frame_id, opaque_frame_optional, non_native_data, local_vars, operand_stack, native_local_refs } = frame;
        assert!(non_native_data.is_none() || non_native_data.unwrap().pc == 0);
        let method_id = match opaque_frame_optional {
            Some(OpaqueFrameOptional { class_pointer, method_i }) => {
                OpaqueFrameIdOrMethodID::Method { method_id: jvm.method_table.write().unwrap().get_method_id(class_pointer, method_i) as u64 }
            }
            None => {
                OpaqueFrameIdOrMethodID::Opaque {
                    opaque_id: opaque_frame_id.unwrap()
                }
            }
        };
        let new_current_frame_position = call_stack.push_frame(*current_stack_position, method_id, local_vars, operand_stack);
        int_state.current_stack_position = new_current_frame_position;

        FramePushGuard { _correctly_exited: false }
    }

    pub fn pop_frame(&mut self, jvm: &'gc_life JVMState<'gc_life>, mut frame_push_guard: FramePushGuard, was_exception: bool) {
        frame_push_guard._correctly_exited = true;
        /*let depth = match self.int_state.as_mut().unwrap().deref_mut() {
            /*InterpreterState::LegacyInterpreter { call_stack, .. } => {
                call_stack.len()
            }*/
            InterpreterState::Jit { call_stack, .. } => unsafe { call_stack.call_stack_depth() },
        };
        let empty_hashset: HashSet<usize> = HashSet::new();
        if match self.int_state.as_mut().unwrap().deref_mut() {
            /*InterpreterState::LegacyInterpreter { should_frame_pop_notify, .. } => should_frame_pop_notify,*/
            InterpreterState::Jit { .. } => &empty_hashset, //todo fix this at later date
        }
            .contains(&depth)
        {
            let stack_entry_ref = self.current_frame();
            let runtime_class = stack_entry_ref.class_pointer(jvm);
            let method_i = self.current_method_i(jvm);
            let method_id = jvm.method_table.write().unwrap().get_method_id(runtime_class.clone(), method_i);
            jvm.jvmti_state().unwrap().built_in_jdwp.frame_pop(jvm, method_id, u8::from(was_exception), self)
        }
        match self.int_state.as_mut().unwrap().deref_mut() {
            /*InterpreterState::LegacyInterpreter { call_stack, .. } => {
                call_stack.pop();
            }*/
            InterpreterState::Jit { call_stack, .. } => unsafe {
                call_stack.pop_frame();
            },
        };*/
        if self.current_frame().is_native() {
            unsafe { drop(Box::from_raw(self.current_frame().frame_view.ir_ref.data(1)[0] as usize as *mut NativeFrameInfo)) }
        }
        assert!(self.thread.is_alive());
    }

    pub fn call_stack_depth(&self) -> usize {
        /*match self.int_state.as_ref().unwrap().deref() {
            /*InterpreterState::LegacyInterpreter { call_stack, .. } => call_stack.len(),*/
            InterpreterState::Jit { call_stack, .. } => unsafe { call_stack.call_stack_depth() },
        }*/
        todo!()
    }

    pub fn set_current_pc(&mut self, new_pc: u16) {
        self.current_frame_mut().set_pc(new_pc);
    }

    pub fn current_pc(&self) -> u16 {
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

    // pub fn verify_frame(&mut self, jvm: &JVMState) {
    //     if let Some(method_id) = self.current_frame().current_method_id(jvm) {
    //         let guard = jvm.function_frame_type_data.read().unwrap();
    //         let Frame { stack_map, locals, .. } = match guard.get(&method_id) {
    //             Some(x) => x,
    //             None => {
    //                 // eprintln!("Warning, missing verification data for: {:?}", self.current_class_view().name());
    //                 return;
    //             }
    //         }.get(&self.current_pc()).unwrap();
    //         let local_java_vals = self.current_frame().local_vars(jvm);
    //         let java_val_stack = self.current_frame().operand_stack();
    //         let stack_map = remove_tops(stack_map);
    //         if stack_map.len() != java_val_stack.len() {
    //             dbg!(&stack_map.data.iter().rev().collect_vec());
    //             dbg!(&java_val_stack);
    //             self.debug_print_stack_trace();
    //             dbg!(self.current_pc());
    //             panic!()
    //         }
    //         for (jv, type_) in java_val_stack.iter().zip(stack_map.data.iter().rev()) {
    //             if !compatible_with_type(jv, type_) {
    //                 dbg!(jv);
    //                 dbg!(type_);
    //                 dbg!(&stack_map.data.iter().rev().collect_vec());
    //                 dbg!(&java_val_stack);
    //                 self.debug_print_stack_trace();
    //                 dbg!(self.current_pc());
    //                 panic!()
    //             }
    //         }
    //         assert_eq!(local_java_vals.len(), locals.deref().len());
    //         for (jv, type_) in local_java_vals.iter().zip(locals.iter()) {
    //             if !compatible_with_type(jv, type_) {
    //                 dbg!(jv);
    //                 dbg!(type_);
    //                 dbg!(&local_java_vals);
    //                 dbg!(&local_java_vals.iter().map(|jv| jv.to_type()).collect_vec());
    //                 dbg!(&locals);
    //                 self.debug_print_stack_trace();
    //                 dbg!(self.current_pc());
    //                 panic!()
    //             }
    //         }
    //     }
    // }
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
}

impl Default for FramePushGuard {
    fn default() -> Self {
        FramePushGuard { _correctly_exited: false }
    }
}

impl Drop for FramePushGuard {
    fn drop(&mut self) {
        // assert!(self._correctly_exited)
    }
}

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