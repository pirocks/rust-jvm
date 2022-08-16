use std::mem::size_of;
use std::ptr::NonNull;
use std::sync::{Arc, MutexGuard};

use itertools::Itertools;
use libc::c_void;

use another_jit_vm_ir::ir_stack::{IRFrameMut, IRFrameRef, OwnedIRStack};
use gc_memory_layout_common::layout::FRAME_HEADER_END_OFFSET;
use rust_jvm_common::{ByteCodeOffset, NativeJavaValue};
use rust_jvm_common::loading::LoaderName;
use rust_jvm_common::runtime_type::RuntimeType;

use crate::{AllocatedHandle, JavaValueCommon, JVMState, MethodResolverImpl, NewJavaValue, NewJavaValueHandle, StackEntryPush};
use crate::better_java_stack::thread_remote_read_mechanism::SignalAccessibleJavaStackData;
use crate::interpreter::real_interpreter_state::InterpreterJavaValue;
use crate::interpreter_state::{NativeFrameInfo, OpaqueFrameInfo};
use crate::ir_to_java_layer::java_stack::OpaqueFrameIdOrMethodID;
use crate::java_values::native_to_new_java_value_rtype;

#[cfg(test)]
pub mod test;
pub mod thread_remote_read_mechanism;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Ord, PartialOrd)]
pub struct FramePointer(pub NonNull<c_void>);

impl FramePointer {
    pub fn as_ptr(&self) -> *mut c_void {
        self.0.as_ptr()
    }

    pub fn as_const_ptr(&self) -> *const c_void {
        self.0.as_ptr() as *const c_void
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct StackDepth(u16);

//needs to keep track of operand stack for interpreter
//      needs to be viewable by other threads
//needs to have same underlying for interpreter and not-interpreter
//      follows that there needs to be a mechanism for non-interpreter frames in exits to know
//      operand stack depth
//needs to be fast
// one per java thread, needs to be
// maybe built on top of ir stack
//todo needs to be interruptable and viewable once interrupted
// todo if in guest then can send stack pointer.
// need a in guest/not in guest atomic, per thread atomic.
pub struct JavaStack<'gc> {
    jvm: &'gc JVMState<'gc>,
    owned_ir_stack: OwnedIRStack,
    interpreter_frame_operand_stack_depths: Vec<(FramePointer, InterpreterFrameState)>,
    throw: Option<AllocatedHandle<'gc>>,
    //todo this should probably be in some kind of thread state thing
    thread_stack_data: Arc<SignalAccessibleJavaStackData>,
}

#[derive(Copy, Clone, Debug)]
pub struct InterpreterFrameState {
    stack_depth: StackDepth,
    current_pc: ByteCodeOffset,
}

impl<'gc> JavaStack<'gc> {
    pub fn new(jvm: &'gc JVMState<'gc>, owned_ir_stack: OwnedIRStack, thread_stack_data: Arc<SignalAccessibleJavaStackData>) -> Self {
        Self {
            jvm,
            owned_ir_stack,
            interpreter_frame_operand_stack_depths: vec![],
            throw: None,
            thread_stack_data,
        }
    }
}

pub struct JavaStackGuard<'vm, 'l> {
    guard: MutexGuard<'l, JavaStack<'vm>>,
    jvm: &'vm JVMState<'vm>
}

impl <'vm, 'l> JavaStackGuard<'vm, 'l> {
    pub fn mmaped_top(&self) -> FramePointer {
        FramePointer(todo!()/*self.owned_ir_stack.native.mmaped_top*/)
    }

    fn assert_interpreter_frame_operand_stack_depths_sorted(&self) {
        todo!()
        /*assert!(self.interpreter_frame_operand_stack_depths.iter().map(|(frame_ptr, _)| *frame_ptr).collect_vec().is_sorted());*/
    }

    pub fn new_interpreter_frame<'k>(&'k mut self, frame_pointer: FramePointer) -> JavaInterpreterFrame<'vm, 'l, 'k> {
        //todo should take an fn
        JavaInterpreterFrame::from_frame_pointer_interpreter(self, frame_pointer)
    }

    pub fn exit_frame<'k>(&'k mut self, frame_pointer: FramePointer, stack_depth: Option<StackDepth>) -> JavaExitFrame<'vm, 'l, 'k> {
        JavaExitFrame { java_stack: self, frame_pointer, num_locals: todo!(), max_stack: todo!(), stack_depth }
    }

    pub fn push_frame(&mut self, frame_to_write: StackEntryPush) {
        //todo should take an fn
        todo!()
    }
}

//need enter and exit native functions, enter taking an operand stack depth?

pub struct JavaExitFrame<'gc, 'l, 'k> {
    java_stack: &'k mut JavaStackGuard<'gc, 'l>,
    frame_pointer: FramePointer,
    num_locals: u16,
    max_stack: u16,
    stack_depth: Option<StackDepth>,
    //get/set/etc
}

pub trait HasFrame<'gc> {
    fn frame_ref(&self) -> IRFrameRef;
    fn frame_mut(&mut self) -> IRFrameMut;
    fn jvm(&self) -> &'gc JVMState<'gc>;
    fn num_locals(&self) -> u16;
    fn max_stack(&self) -> u16;
    fn local_get(&self, i: u16, expected_type: RuntimeType) -> NewJavaValueHandle<'gc> {
        assert!(i < self.num_locals());
        let jvm = self.jvm();
        let ir_frame_ref = self.frame_ref();
        let data = ir_frame_ref.data(i as usize);//todo replace this with a layout lookup thing again
        let native_jv = NativeJavaValue { as_u64: data };
        native_to_new_java_value_rtype(native_jv, expected_type, jvm)
    }

    fn local_set(&mut self, i: u16, njv: NewJavaValue<'gc, '_>) {
        assert!(i < self.num_locals());
        let native_jv = njv.to_native();
        let ir_frame_mut = self.frame_mut();
        ir_frame_mut.write_data(i as usize, unsafe { native_jv.as_u64 });
    }

    fn os_set_from_start(&mut self, from_start: u16, njv: NewJavaValue<'gc, '_>) {
        let native_jv = njv.to_native();
        self.os_set_from_start_raw(from_start, unsafe { native_jv.as_u64 })
    }

    fn os_set_from_start_raw(&mut self, from_start: u16, raw: u64) {
        assert!(from_start < self.max_stack());
        let num_locals = self.num_locals() as usize;
        let ir_frame_mut = self.frame_mut();
        ir_frame_mut.write_data(num_locals + from_start as usize, raw);
    }

    fn os_get_from_start(&mut self, from_start: u16, expected_type: RuntimeType) -> NewJavaValueHandle<'gc> {
        assert!(from_start < self.max_stack());
        let ir_frame_ref = self.frame_ref();
        let num_locals = self.num_locals() as usize;
        let data = ir_frame_ref.data(num_locals + from_start as usize);//todo replace this with a layout lookup thing again
        let native_jv = NativeJavaValue { as_u64: data };
        native_to_new_java_value_rtype(native_jv, expected_type, self.jvm())
    }
}

impl<'gc, 'l, 'k> HasFrame<'gc> for JavaExitFrame<'gc, 'l, 'k> {
    fn frame_ref(&self) -> IRFrameRef {
        IRFrameRef {
            ptr: self.frame_pointer.0.into(),
            _ir_stack: todo!()/*&self.java_stack.owned_ir_stack*/,
        }
    }

    fn frame_mut(&mut self) -> IRFrameMut {
        IRFrameMut {
            ptr: self.frame_pointer.0,
            ir_stack: todo!()/*&mut self.java_stack.owned_ir_stack*/,
        }
    }

    fn jvm(&self) -> &'gc JVMState<'gc> {
        self.java_stack.jvm
    }

    fn num_locals(&self) -> u16 {
        self.num_locals
    }

    fn max_stack(&self) -> u16 {
        self.max_stack
    }
}


impl<'gc, 'l, 'k> JavaExitFrame<'gc, 'l, 'k> {}

pub struct JavaInterpreterFrame<'gc, 'l, 'k> {
    java_stack: &'k mut JavaStackGuard<'gc, 'l>,
    frame_ptr: FramePointer,
    num_locals: u16,
    max_stack: u16,
    current_operand_stack_depth: u16,
    //push, pop etc
}

impl<'gc, 'l, 'k> HasFrame<'gc> for JavaInterpreterFrame<'gc, 'l, 'k> {
    fn frame_ref(&self) -> IRFrameRef {
        IRFrameRef {
            ptr: self.frame_ptr.0.into(),
            _ir_stack: todo!()/*&self.java_stack.owned_ir_stack*/,
        }
    }

    fn frame_mut(&mut self) -> IRFrameMut {
        IRFrameMut {
            ptr: self.frame_ptr.0,
            ir_stack: todo!()/*&mut self.java_stack.owned_ir_stack*/,
        }
    }

    fn jvm(&self) -> &'gc JVMState<'gc> {
        self.java_stack.jvm
    }

    fn num_locals(&self) -> u16 {
        self.num_locals
    }

    fn max_stack(&self) -> u16 {
        self.max_stack
    }
}

impl<'gc, 'l, 'k> JavaInterpreterFrame<'gc, 'l, 'k> {
    pub fn pop_frame(self) {}

    fn next_frame_pointer(&self) -> FramePointer {
        //todo need a better way of providing layout
        unsafe { FramePointer(NonNull::new(self.frame_ptr.as_ptr().sub(FRAME_HEADER_END_OFFSET + size_of::<u64>() * (self.num_locals as usize + self.max_stack as usize))).unwrap()) }
    }

    pub fn from_frame_pointer_interpreter(java_stack: &'k mut JavaStackGuard<'gc, 'l>, frame_pointer: FramePointer) -> Self {
        let mut res = Self {
            java_stack,
            frame_ptr: frame_pointer,
            num_locals: 0,
            max_stack: 0,
            current_operand_stack_depth: 0,
        };
        let method_id = res.frame_ref().method_id().unwrap();
        let jvm = res.jvm();
        let (rc, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
        let view = rc.view();
        let method_view = view.method_view_i(method_i);
        let code = method_view.code_attribute().unwrap();
        res.num_locals = code.max_locals;
        res.max_stack = code.max_stack;
        res
    }

    fn get_top_level_exit_ptr(&self) -> NonNull<c_void> {
        let ir_vm_state = &self.jvm().java_vm_state.ir;
        let top_level_ir_method_id = ir_vm_state.get_top_level_return_ir_method_id();
        ir_vm_state.lookup_ir_method_id_pointer(top_level_ir_method_id)
    }

    pub fn push_frame(&mut self, stack_entry: StackEntryPush) -> JavaInterpreterFrame<'gc, 'l, 'k> {
        let current_frame_pointer = self.frame_ptr;
        let next_frame_pointer = self.next_frame_pointer();
        let top_level_exit_ptr = self.get_top_level_exit_ptr();
        let jvm = self.jvm();
        match stack_entry {
            StackEntryPush::Java { operand_stack, local_vars, method_id } => {
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
                unsafe {
                    todo!()
                    /*self.java_stack.owned_ir_stack.write_frame(
                        next_frame_pointer.0,
                        top_level_exit_ptr.as_ptr(),
                        current_frame_pointer.as_ptr(),
                        ir_method_id,
                        wrapped_method_id.to_native(),
                        data.as_slice(),
                    );*/
                }
                let (rc, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
                let view = rc.view();
                let method_view = view.method_view_i(method_i);
                let code = method_view.code_attribute().unwrap();
                JavaInterpreterFrame {
                    java_stack: &mut self.java_stack,
                    frame_ptr: next_frame_pointer,
                    num_locals: code.max_locals,
                    max_stack: code.max_stack,
                    current_operand_stack_depth: 0,
                }
            }
            StackEntryPush::Native { method_id, native_local_refs, local_vars, operand_stack } => {
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
                unsafe {
                    todo!()
                    /*self.java_stack.owned_ir_stack.write_frame(
                        next_frame_pointer.0,
                        top_level_exit_ptr.as_ptr(),
                        current_frame_pointer.as_ptr(),
                        Some(ir_method_id),
                        wrapped_method_id.to_native(),
                        data.as_slice(),
                    );*/
                }
                panic!()
            }
            StackEntryPush::Opaque { opaque_id, native_local_refs } => {
                let wrapped_opaque_id = OpaqueFrameIdOrMethodID::Opaque { opaque_id };
                let opaque_frame_info = OpaqueFrameInfo { native_local_refs, operand_stack: vec![] };
                let raw_frame_info_pointer = Box::into_raw(box opaque_frame_info);
                let data = [raw_frame_info_pointer as *const c_void as usize as u64];
                unsafe {
                    todo!()
                    /*self.java_stack.owned_ir_stack.write_frame(
                        next_frame_pointer.0,
                        top_level_exit_ptr.as_ptr(),
                        current_frame_pointer.as_ptr(),
                        None,
                        wrapped_opaque_id.to_native(),
                        data.as_slice(),
                    );*/
                }
                panic!()
            }
        }
    }

    pub fn push_os(&mut self, njv: InterpreterJavaValue) {
        let current_depth = self.current_operand_stack_depth;
        self.os_set_from_start_raw(current_depth, njv.to_raw());
        self.current_operand_stack_depth += 1;
    }

    pub fn pop_os(&mut self, expected_type: RuntimeType) -> InterpreterJavaValue {
        if self.current_operand_stack_depth == 0 {
            panic!()
        }
        self.current_operand_stack_depth -= 1;
        let current_depth = self.current_operand_stack_depth;
        self.os_get_from_start(current_depth, expected_type).to_interpreter_jv()
    }
}


pub struct RemoteFrame {
    frame_ptr: FramePointer,
    num_locals: u16,
    max_stack: u16,
    current_operand_stack_depth: u16,
}
// don't have the function call vec thing

