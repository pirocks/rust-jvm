use std::cell::RefCell;
use std::collections::HashSet;
use std::ffi::c_void;
use std::mem::transmute;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, RwLockWriteGuard};

use itertools::{Either, Itertools};

use classfile_view::loading::{ClassWithLoader, LoaderName};
use classfile_view::view::{ClassView, HasAccessFlags};
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use classfile_view::vtype::VType;
use gc_memory_layout_common::{FrameBackedStackframeMemoryLayout, FrameInfo, FullyOpaqueFrame, NativeStackframeMemoryLayout, StackframeMemoryLayout};
use jit_common::java_stack::{JavaStack, JavaStatus};
use rust_jvm_common::classfile::CPIndex;
use verification::OperandStack;

use crate::interpreter_state::AddFrameNotifyError::{NothingAtDepth, Opaque};
use crate::java_values::{JavaValue, Object};
use crate::jvm_state::JVMState;
use crate::rust_jni::native_util::from_object;
use crate::stack_entry::{FrameView, NonNativeFrameData, OpaqueFrameOptional, StackEntry, StackEntryMut, StackEntryRef, StackIter};
use crate::threading::JavaThread;

#[derive(Debug)]
pub enum InterpreterState {
    // LegacyInterpreter {
    //     throw: Option<Arc<Object>>,
    //     function_return: bool,
    //     call_stack: Vec<StackEntry>,
    //     should_frame_pop_notify: HashSet<usize>,
    // },
    Jit {
        call_stack: JavaStack,
    },
}

impl InterpreterState {
    pub(crate) fn new(thread_status_ptr: *mut JavaStatus) -> Self {
        let call_stack = JavaStack::new(0, thread_status_ptr);
        InterpreterState::Jit {
            call_stack,
        }
    }
}

pub struct InterpreterStateGuard<'vm_life: 'l, 'l> {
    pub(crate) int_state: Option<RwLockWriteGuard<'l, InterpreterState>>,
    pub(crate) thread: Arc<JavaThread<'vm_life>>,
    pub(crate) registered: bool,
}


thread_local! {
pub static CURRENT_INT_STATE_GUARD_VALID :RefCell<bool> = RefCell::new(false);
}

thread_local! {
pub static CURRENT_INT_STATE_GUARD :RefCell<Option<*mut InterpreterStateGuard<'static,'static>>> = RefCell::new(None);
}


impl<'gc_life, 'm> InterpreterStateGuard<'gc_life, 'm> {
    pub fn register_interpreter_state_guard(&mut self, jvm: &'_ JVMState<'gc_life>) {
        let ptr = unsafe { transmute::<_, *mut InterpreterStateGuard<'static, 'static>>(self as *mut InterpreterStateGuard<'gc_life, '_>) };
        jvm.thread_state.int_state_guard.with(|refcell| refcell.replace(ptr.into()));
        jvm.thread_state.int_state_guard_valid.with(|refcell| refcell.replace(true));
        self.registered = true;
        assert!(self.thread.is_alive());
    }


    pub fn new(jvm: &'gc_life JVMState<'gc_life>, thread: Arc<JavaThread<'gc_life>>, option: Option<RwLockWriteGuard<'m, InterpreterState>>) -> InterpreterStateGuard<'gc_life, 'm> {
        jvm.thread_state.int_state_guard_valid.with(|refcell| refcell.replace(false));
        Self {
            int_state: option,
            thread: thread.clone(),
            registered: true,
        }
    }

    pub fn current_loader(&self) -> LoaderName {
        self.current_frame().loader()
    }

    pub fn current_class_view(&self, jvm: &'_ JVMState<'gc_life>) -> Arc<dyn ClassView> {
        self.current_frame().try_class_pointer(jvm).unwrap().view()
    }


    pub fn current_frame(&'_ self) -> StackEntryRef<'gc_life> {
        let interpreter_state = self.int_state.as_ref().unwrap();
        match interpreter_state.deref() {
            /*InterpreterState::LegacyInterpreter { call_stack, .. } => {
                StackEntryRef::LegacyInterpreter { entry: call_stack.last().unwrap() }
            }*/
            InterpreterState::Jit { call_stack } => {
                StackEntryRef::Jit { frame_view: FrameView::new(call_stack.current_frame_ptr()) }
            }
        }
    }

    pub fn current_frame_mut(&mut self) -> StackEntryMut<'gc_life> {
        let interpreter_state = self.int_state.as_mut().unwrap().deref_mut();
        match interpreter_state {
            /*InterpreterState::LegacyInterpreter { call_stack, .. } => {
                StackEntryMut::LegacyInterpreter { entry: call_stack.last_mut().unwrap() }
            }*/
            InterpreterState::Jit { call_stack } => {
                StackEntryMut::Jit { frame_view: FrameView::new(call_stack.current_frame_ptr()) }
            }
        }
    }

    pub fn push_current_operand_stack(&mut self, jval: JavaValue<'gc_life>) {
        self.current_frame_mut().push(jval)
    }

    pub fn pop_current_operand_stack(&mut self, expected_type: PTypeView) -> JavaValue<'gc_life> {
        self.current_frame_mut().operand_stack_mut().pop(expected_type).unwrap()
    }

    pub fn previous_frame_mut(&mut self) -> StackEntryMut<'gc_life> {
        match self.int_state.as_mut().unwrap().deref_mut() {
            /*InterpreterState::LegacyInterpreter { call_stack, .. } => {
                let len = call_stack.len();
                StackEntryMut::LegacyInterpreter { entry: &mut call_stack[len - 2] }
            }*/
            InterpreterState::Jit { call_stack } => {
                StackEntryMut::Jit { frame_view: FrameView::new(call_stack.previous_frame_ptr()) }
            }
        }
    }

    pub fn previous_frame(&self) -> StackEntryRef<'gc_life> {
        match self.int_state.as_ref().unwrap().deref() {
            /*InterpreterState::LegacyInterpreter { call_stack, .. } => {
                let len = call_stack.len();
                StackEntryRef::LegacyInterpreter { entry: &call_stack[len - 2] }
            }*/
            InterpreterState::Jit { call_stack } => {
                StackEntryRef::Jit { frame_view: FrameView::new(call_stack.previous_frame_ptr()) }
            }
        }
    }

    pub fn set_throw(&mut self, val: Option<Arc<Object<'gc_life>>>) {
        match self.int_state.as_mut() {
            None => {
                let mut guard = self.thread.interpreter_state.write().unwrap();
                match guard.deref_mut() {
                    /*InterpreterState::LegacyInterpreter { throw, .. } => {
                        *throw = val;
                    }*/
                    InterpreterState::Jit { .. } => todo!()
                }
            }
            Some(val_mut) => {
                match val_mut.deref_mut() {
                    /*InterpreterState::LegacyInterpreter { throw, .. } => {
                        *throw = val;
                    }*/
                    InterpreterState::Jit { .. } => {
                        todo!()
                    }
                }
            }
        }
    }


    pub fn function_return(&mut self) -> bool {
        let int_state = self.int_state.as_mut().unwrap();
        match int_state.deref_mut() {
            /*InterpreterState::LegacyInterpreter { function_return, .. } => {
                *function_return
            }*/
            InterpreterState::Jit { call_stack, .. } => {
                unsafe { call_stack.saved_registers().status_register.as_mut() }.unwrap().function_return
            }
        }
    }


    pub fn set_function_return(&mut self, to: bool) {
        let int_state = self.int_state.as_mut().unwrap();
        match int_state.deref_mut() {
            /*InterpreterState::LegacyInterpreter { function_return, .. } => {
                *function_return = to;
            }*/
            InterpreterState::Jit { call_stack, .. } => {
                unsafe { call_stack.saved_registers().status_register.as_mut().unwrap().function_return = to; }
            }
        }
    }


    pub fn throw(&self) -> Option<Arc<Object<'gc_life>>> {
        match self.int_state.as_ref() {
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
                InterpreterState::Jit { call_stack, .. } => {
                    unsafe { from_object(call_stack.throw()) }
                }
            },
        }
    }

    pub fn push_frame(&mut self, frame: StackEntry<'gc_life>, jvm: &'_ JVMState<'gc_life>) -> FramePushGuard {
        let int_state = self.int_state.as_mut().unwrap().deref_mut();
        match int_state {
            /*InterpreterState::LegacyInterpreter { call_stack, .. } => {
                call_stack.push(frame)
            }*/
            InterpreterState::Jit { call_stack, .. } => {
                let StackEntry {
                    loader,
                    opaque_frame_optional,
                    non_native_data,
                    local_vars,
                    operand_stack,
                    native_local_refs
                } = frame;
                if let Some(NonNativeFrameData { pc, pc_offset }) = non_native_data {
                    if let Some(OpaqueFrameOptional { class_pointer, method_i }) = opaque_frame_optional {
                        let method_id = jvm.method_table.write().unwrap().get_method_id(class_pointer.clone(), method_i);
                        let class_view = class_pointer.view();
                        let method_view = class_view.method_view_i(method_i);
                        let code = method_view.code_attribute().unwrap();
                        let frame_vtype = &jvm.function_frame_type_data.read().unwrap()[&(method_id as usize)];//TODO MAKE SAFE TYPE WRAPPERS FOR METHOD ID AND I
                        let memory_layout = FrameBackedStackframeMemoryLayout::new(code.max_stack as usize, code.max_locals as usize, frame_vtype.clone());
                        assert!(operand_stack.is_empty());//todo setup operand stack
                        unsafe {
                            call_stack.push_frame(&memory_layout, FrameInfo::JavaFrame {
                                method_id,
                                num_locals: code.max_locals,
                                loader,
                                java_pc: pc,
                                pc_offset,
                                operand_stack_depth: operand_stack.len() as u16,
                            });
                        }
                        // dbg!(&local_vars);
                        // dbg!(method_view.name());
                        // dbg!(class_view.name());
                        for (i, local_var) in local_vars.into_iter().enumerate() {
                            self.current_frame_mut().local_vars_mut().set(i as u16, local_var);
                        }
                        jvm.stack_frame_layouts.write().unwrap().insert(method_id, memory_layout);
                    } else {
                        panic!()
                    }
                } else if let Some(OpaqueFrameOptional { class_pointer, method_i }) = opaque_frame_optional {
                    let method_id = jvm.method_table.write().unwrap().get_method_id(class_pointer.clone(), method_i);
                    let class_view = class_pointer.view();
                    let method_view = class_view.method_view_i(method_i);
                    unsafe {
                        call_stack.push_frame(&NativeStackframeMemoryLayout {}, FrameInfo::Native {
                            method_id,
                            loader,
                            operand_stack_depth: 0,
                            native_local_refs,
                        })
                    }
                } else {
                    unsafe {
                        call_stack.push_frame(&FullyOpaqueFrame { max_stack: 0, max_frame: 0 }, FrameInfo::FullyOpaque { loader, operand_stack_depth: 0 })
                    }
                }
            }
        };
        FramePushGuard::default()
    }

    pub fn pop_frame(&mut self, jvm: &'_ JVMState<'gc_life>, mut frame_push_guard: FramePushGuard, was_exception: bool) {
        frame_push_guard._correctly_exited = true;
        let depth = match self.int_state.as_mut().unwrap().deref_mut() {
            /*InterpreterState::LegacyInterpreter { call_stack, .. } => {
                call_stack.len()
            }*/
            InterpreterState::Jit { call_stack, .. } => unsafe {
                call_stack.call_stack_depth()
            }
        };
        let empty_hashset: HashSet<usize> = HashSet::new();
        if match self.int_state.as_mut().unwrap().deref_mut() {
            /*InterpreterState::LegacyInterpreter { should_frame_pop_notify, .. } => should_frame_pop_notify,*/
            InterpreterState::Jit { .. } => &empty_hashset//todo fix this at later date
        }.contains(&depth) {
            let stack_entry_ref = self.current_frame();
            let runtime_class = stack_entry_ref.class_pointer(jvm);
            let method_i = self.current_method_i(jvm);
            let method_id = jvm.method_table.write().unwrap().get_method_id(runtime_class.clone(), method_i);
            jvm.jvmti_state.as_ref().unwrap().built_in_jdwp.frame_pop(jvm, method_id, u8::from(was_exception), self)
        }
        match self.int_state.as_mut().unwrap().deref_mut() {
            /*InterpreterState::LegacyInterpreter { call_stack, .. } => {
                call_stack.pop();
            }*/
            InterpreterState::Jit { call_stack, .. } => {
                unsafe { call_stack.pop_frame(); }
            }
        };
        assert!(self.thread.is_alive());
    }

    pub fn call_stack_depth(&self) -> usize {
        match self.int_state.as_ref().unwrap().deref() {
            /*InterpreterState::LegacyInterpreter { call_stack, .. } => call_stack.len(),*/
            InterpreterState::Jit { call_stack, .. } => {
                unsafe { call_stack.call_stack_depth() }
            }
        }
    }

    pub fn set_current_pc(&mut self, new_pc: u16) {
        self.current_frame_mut().set_pc(new_pc);
    }

    pub fn current_pc(&self) -> u16 {
        self.current_frame().pc()
    }

    pub fn set_current_pc_offset(&mut self, new_offset: i32) {
        self.current_frame_mut().set_pc_offset(new_offset)
    }

    pub fn current_pc_offset(&self) -> i32 {
        self.current_frame().pc_offset()
    }

    pub fn current_method_i(&self, jvm: &'_ JVMState<'gc_life>) -> CPIndex {
        self.current_frame().method_i(jvm)
    }

    pub fn debug_print_stack_trace(&self, jvm: &'_ JVMState<'gc_life>) {
        let iter = match self.int_state.as_ref().unwrap().deref() {
            /*InterpreterState::LegacyInterpreter { call_stack, .. } => {
                Either::Left(call_stack.iter().cloned().enumerate().rev())
            }*/
            InterpreterState::Jit { call_stack, .. } => {
                StackIter::new(jvm, call_stack).into_iter().enumerate()
            }
        };
        for (i, stack_entry) in iter {
            if stack_entry.try_method_i().is_some() /*&& stack_entry.method_i() > 0*/ {
                let type_ = stack_entry.class_pointer().view().type_();
                let view = stack_entry.class_pointer().view();
                let method_view = view.method_view_i(stack_entry.method_i());
                let meth_name = method_view.name();
                if method_view.is_native() {
                    println!("{:?}.{} {} {}", type_, meth_name, method_view.desc_str(), i)
                } else {
                    println!("{:?}.{} {} {} pc: {} {}", type_
                             , meth_name,
                             method_view.desc_str(), i, stack_entry
                                 .pc(), stack_entry.loader())
                }
            } else {
                println!("missing");
            }
        }
    }

    pub fn cloned_stack_snapshot(&self, jvm: &'_ JVMState<'gc_life>) -> Vec<StackEntry<'gc_life>> {
        match self.int_state.as_ref().unwrap().deref() {
            /*InterpreterState::LegacyInterpreter { call_stack, .. } => call_stack.to_vec(),*/
            InterpreterState::Jit { call_stack, .. } => {
                StackIter::new(jvm, call_stack).collect_vec().into_iter().rev().collect_vec()
            }
        }
    }

    pub fn depth(&self) -> usize {
        match self.int_state.as_ref().unwrap().deref() {
            /*InterpreterState::LegacyInterpreter { call_stack, .. } => call_stack.len(),*/
            InterpreterState::Jit { .. } => todo!()
        }
    }

    pub fn add_should_frame_pop_notify(&mut self, depth: usize) -> Result<(), AddFrameNotifyError> {
        let call_stack_depth = self.call_stack_depth();
        let int_state = self.int_state.as_mut().unwrap().deref_mut();
        if depth >= call_stack_depth {
            return Err(NothingAtDepth);
        }
        let entry: &StackEntry = &match int_state {
            /*InterpreterState::LegacyInterpreter { call_stack, .. } => {
                &call_stack[depth]
            }*/
            InterpreterState::Jit { .. } => todo!()
        };
        if entry.is_native() || entry.try_class_pointer().is_none() {
            return Err(Opaque);
        }
        match int_state {
            /*InterpreterState::LegacyInterpreter { should_frame_pop_notify, .. } => {
                should_frame_pop_notify.insert(depth);
            }*/
            InterpreterState::Jit { .. } => todo!()
        };
        Ok(())
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
    //         let local_java_vals = self.current_frame().local_vars();
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
        Self {
            suspended: std::sync::Mutex::new(false),
            suspend_condvar: Default::default(),
        }
    }
}
