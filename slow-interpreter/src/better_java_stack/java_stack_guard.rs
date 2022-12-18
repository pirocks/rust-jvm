use std::ffi::c_void;
use std::mem::transmute;
use std::ptr::NonNull;
use std::sync::{Arc, Mutex, MutexGuard};

use itertools::Itertools;

use another_jit_vm_ir::HasRBPAndRSP;
use another_jit_vm_ir::ir_stack::OwnedIRStack;
use rust_jvm_common::ByteCodeOffset;
use rust_jvm_common::loading::LoaderName;
use thread_signal_handler::SignalAccessibleJavaStackData;

use crate::{JavaValueCommon, JVMState, MethodResolverImpl};
use crate::better_java_stack::{FramePointer, InterpreterFrameState, JavaStack, StackDepth};
use crate::better_java_stack::frames::HasFrame;
use crate::better_java_stack::interpreter_frame::JavaInterpreterFrame;
use crate::better_java_stack::native_frame::NativeFrame;
use crate::better_java_stack::opaque_frame::OpaqueFrame;
use crate::exceptions::WasException;
use crate::interpreter_state::{NativeFrameInfo, OpaqueFrameInfo};
use crate::ir_to_java_layer::java_stack::OpaqueFrameIdOrMethodID;
use crate::rust_jni::PerStackInterfaces;
use crate::stack_entry::{JavaFramePush, NativeFramePush, OpaqueFramePush};
use crate::threading::java_thread::JavaThread;
use std::cell::RefCell;
use jvmti_jni_bindings::jmm_interface::JMMInterfaceNamedReservedPointers;

thread_local! {
    pub static JMM: RefCell<Option<*mut JMMInterfaceNamedReservedPointers>> = RefCell::new(None)
}

pub struct JavaStackGuard<'vm> {
    stack: &'vm Mutex<JavaStack<'vm>>,
    guard: Option<MutexGuard<'vm, JavaStack<'vm>>>,
    jvm: &'vm JVMState<'vm>,
    pub java_thread: Arc<JavaThread<'vm>>,
    _current_frame_pointer: FramePointer,
}

impl<'vm> JavaStackGuard<'vm> {
    pub(crate) fn ir_stack_ref(&self) -> &OwnedIRStack {
        &self.guard.as_ref().unwrap().owned_ir_stack
    }

    pub(crate) fn ir_stack_mut(&mut self) -> &mut OwnedIRStack {
        &mut self.guard.as_mut().unwrap().owned_ir_stack
    }

    pub(crate) fn stack_jni_interface(&mut self) -> &mut PerStackInterfaces {
        &mut self.guard.as_mut().unwrap().per_stack_interface
    }

    pub(crate) fn has_been_used(&self) -> bool {
        self.guard.as_ref().unwrap().has_been_used
    }

    pub(crate) fn set_has_been_used(&mut self) {
        self.guard.as_mut().unwrap().has_been_used = true
    }

    pub(crate) fn ir_stack(&self) -> &OwnedIRStack {
        &self.guard.as_ref().unwrap().owned_ir_stack
    }


    fn configure_jmm(jvm: &'vm JVMState<'vm>, per_stack_interface: &mut PerStackInterfaces){
        JMM.with(|jmm|{
            let mut jmm_base = jvm.default_per_stack_initial_interfaces.jmm.clone();
            unsafe { jmm_base.jvm_state = transmute(jvm); }
            per_stack_interface.jmm = jmm_base;
            assert!(jmm.borrow().is_none());
            jmm.replace(Some(per_stack_interface.jmm_inner_mut_raw()));
        })
    }

    //todo I really need an init function which just creates the mutex and everything in one place
    pub fn new_from_empty_stack<T>(
        jvm: &'vm JVMState<'vm>,
        java_thread: Arc<JavaThread<'vm>>,
        stack: &'vm Mutex<JavaStack<'vm>>,
        with_initial_opaque_frame: impl for<'l, 'k> FnOnce(&'l mut OpaqueFrame<'vm, 'k>) -> Result<T, WasException<'vm>> + 'vm
    ) -> Result<T, WasException<'vm>> {
        let guard = stack.lock().unwrap();
        if guard.has_been_used {
            panic!()
        }
        let mmapped_top = guard.owned_ir_stack.native.mmaped_top;
        let mut res = Self {
            stack,
            guard: Some(guard),
            jvm,
            java_thread,
            _current_frame_pointer: FramePointer(mmapped_top),
        };
        let mut opaque_frame = OpaqueFrame::new_from_empty_stack(&mut res);
        Self::configure_jmm(jvm, opaque_frame.java_stack_mut().stack_jni_interface());
        with_initial_opaque_frame(&mut opaque_frame)
    }

    pub fn new_from_prev_with_new_frame_pointer(old: Self, new_frame_pointer: FramePointer) -> Self {
        let Self { stack, guard, jvm, java_thread, _current_frame_pointer:_ } = old;
        Self {
            stack,
            guard,
            jvm,
            java_thread,
            _current_frame_pointer: new_frame_pointer,
        }
    }

    pub fn new_remote_with_frame_pointer(jvm: &'vm JVMState<'vm>, stack: &'vm Mutex<JavaStack<'vm>>, java_thread: Arc<JavaThread<'vm>>, new_frame_pointer: FramePointer) -> Self {
        Self {
            stack,
            guard: Some(stack.lock().unwrap()),
            jvm,
            java_thread,
            _current_frame_pointer: new_frame_pointer,
        }
    }

    pub fn debug_assert(&self) {
        self.assert_interpreter_frame_operand_stack_depths_sorted();
    }

    fn assert_interpreter_frame_operand_stack_depths_sorted(&self) {
        self.guard.as_ref().unwrap().assert_interpreter_frame_operand_stack_depths_sorted();
    }

    pub fn jvm(&self) -> &'vm JVMState<'vm> {
        self.jvm
    }


    pub fn push_java_frame<'k, T>(&'k mut self,
                                  current_frame_pointer: FramePointer,
                                  next_frame_pointer: FramePointer,
                                  java_stack_entry: JavaFramePush,
                                  within_pushed: impl for<'l> FnOnce(&mut JavaInterpreterFrame<'vm, 'l>) -> Result<T, WasException<'vm>>,
    ) -> Result<T, WasException<'vm>> {
        let JavaFramePush { method_id, local_vars, operand_stack } = java_stack_entry;
        let jvm = self.jvm();
        let top_level_exit_ptr = get_top_level_exit_ptr(jvm);
        assert_eq!(jvm.num_local_var_slots(method_id) as usize, local_vars.len());
        let ir_method_id = jvm.java_vm_state.try_lookup_method_ir_method_id(method_id);
        let mut data = vec![];
        for local_var in local_vars {
            if let Some(Some(obj)) = local_var.try_unwrap_object_alloc() {
                jvm.gc.memory_region.lock().unwrap().find_object_allocated_type(obj.ptr());
            }
            data.push(unsafe { local_var.to_stack_native().as_u64 });
        }
        for jv in operand_stack {
            data.push(unsafe { jv.to_stack_native().as_u64 });
        }
        let wrapped_method_id = OpaqueFrameIdOrMethodID::Method { method_id: method_id as u64 };
        unsafe {
            self.guard.as_mut().unwrap().owned_ir_stack.write_frame(
                next_frame_pointer.0,
                top_level_exit_ptr.as_ptr(),
                current_frame_pointer.as_ptr(),
                ir_method_id,
                wrapped_method_id.to_native(),
                data.as_slice(),
            );
        }
        self.notify_frame_push(next_frame_pointer);
        let res = JavaInterpreterFrame::from_frame_pointer_interpreter(self, next_frame_pointer, |within| {
            within_pushed(within)
        });
        self.notify_frame_pop(next_frame_pointer);
        res
    }

    pub fn push_frame_native<'k, 'gc, T>(&'k mut self,
                                         current_frame_pointer: FramePointer,
                                         next_frame_pointer: FramePointer,
                                         stack_entry: NativeFramePush,
                                         within_pushed: impl for<'k2> FnOnce(&mut NativeFrame<'vm, 'k2>) -> Result<T, WasException<'vm>>,
    ) -> Result<T, WasException<'vm>> {
        let NativeFramePush { method_id, native_local_refs, local_vars, operand_stack } = stack_entry;
        let jvm = self.jvm();
        let top_level_exit_ptr = get_top_level_exit_ptr(jvm);
        let method_resolver_impl = MethodResolverImpl { jvm, loader: LoaderName::BootstrapLoader/*todo fix*/ };
        jvm.java_vm_state.add_method_if_needed(jvm, &method_resolver_impl, method_id, false);
        let ir_method_id = jvm.java_vm_state.lookup_method_ir_method_id(method_id);
        let (rc, _) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
        let loader = jvm.classes.read().unwrap().get_initiating_loader(&rc);
        assert_eq!(jvm.num_local_vars_native(method_id) as usize, local_vars.len());
        assert!(native_local_refs.len() >= 1);
        let native_frame_info = NativeFrameInfo {
            method_id,
            loader,
            native_local_refs,
            // local_vars: local_vars.iter().map(|njv|njv.to_native()).collect(),
            operand_stack: operand_stack.iter().map(|njv| njv.to_stack_native()).collect(),
        };
        let raw_frame_info_pointer = Box::into_raw(box native_frame_info);
        let wrapped_method_id = OpaqueFrameIdOrMethodID::Method { method_id: method_id as u64 };
        //todo use NativeStackframeMemoryLayout for this
        let mut data = local_vars.iter().map(|local_var| unsafe { local_var.to_stack_native().as_u64 }).collect_vec();
        data.push(raw_frame_info_pointer as *const c_void as usize as u64);
        unsafe {
            self.guard.as_mut().unwrap().owned_ir_stack.write_frame(
                next_frame_pointer.0,
                top_level_exit_ptr.as_ptr(),
                current_frame_pointer.as_ptr(),
                Some(ir_method_id),
                wrapped_method_id.to_native(),
                data.as_slice(),
            );
        }
        self.notify_frame_push(next_frame_pointer);
        let mut frame = NativeFrame::new_from_pointer(self, next_frame_pointer, local_vars.len() as u16);
        unsafe {
            let jvm = frame.jvm();
            if libc::rand() < 10_000 {
                frame.debug_print_stack_trace(jvm);
                dbg!(jvm.method_table.read().unwrap().lookup_method_string(method_id, &jvm.string_pool));
            }
        }
        let res: Result<T, WasException<'vm>> = within_pushed(&mut frame);
        self.notify_frame_pop(next_frame_pointer);
        let to_drop = unsafe { Box::from_raw(raw_frame_info_pointer as *mut NativeFrameInfo) };
        drop(to_drop);
        res
    }

    pub fn push_opaque_frame<'k, T>(&'k mut self,
                                    current_frame_pointer: FramePointer,
                                    next_frame_pointer: FramePointer,
                                    opaque_frame: OpaqueFramePush,
                                    within_pushed: impl for<'l> FnOnce(&mut OpaqueFrame<'vm, 'l>) -> Result<T, WasException<'vm>>,
    ) -> Result<T, WasException<'vm>> {
        let OpaqueFramePush { opaque_id, native_local_refs } = opaque_frame;
        let jvm = self.jvm();
        let top_level_exit_ptr = get_top_level_exit_ptr(jvm);
        let wrapped_opaque_id = OpaqueFrameIdOrMethodID::Opaque { opaque_id };
        let opaque_frame_info = OpaqueFrameInfo { native_local_refs, operand_stack: vec![] };
        let raw_frame_info_pointer = Box::into_raw(box opaque_frame_info);
        let data = [raw_frame_info_pointer as *const c_void as usize as u64];
        unsafe {
            self.guard.as_mut().unwrap().owned_ir_stack.write_frame(
                next_frame_pointer.0,
                top_level_exit_ptr.as_ptr(),
                current_frame_pointer.as_ptr(),
                None,
                wrapped_opaque_id.to_native(),
                data.as_slice(),
            );
        }
        self.notify_frame_push(next_frame_pointer);
        let mut frame = OpaqueFrame::new_from_frame_pointer(self, next_frame_pointer);
        let res = within_pushed(&mut frame);
        self.notify_frame_pop(next_frame_pointer);
        let to_drop = unsafe { Box::from_raw(raw_frame_info_pointer as *mut OpaqueFrameInfo) };
        drop(to_drop);
        //todo zero the rest
        res
    }

    pub fn signal_safe_data(&self) -> &SignalAccessibleJavaStackData {
        self.guard.as_ref().unwrap().signal_safe_data()
    }

    pub fn lookup_interpreter_pc_offset_with_frame_pointer(&self, frame_pointer: FramePointer) -> Option<ByteCodeOffset> {
        let (_, res) = self.guard.as_ref().unwrap().interpreter_frame_operand_stack_depths.iter().find(|(current_frame_pointer, _)| current_frame_pointer == &frame_pointer)?.clone();
        Some(res.current_pc)
    }

    pub(crate) fn notify_frame_pop(&mut self, pop_to_inclusive: FramePointer) {
        // for _ in 0..self.guard.as_ref().unwrap().interpreter_frame_operand_stack_depths.len() {
        //     print!(" ");
        // }
        // println!("Pop: {}", method_name);
        self.assert_interpreter_frame_operand_stack_depths_sorted();
        let mut already_popped = false;
        loop {
            let last = self.guard.as_ref().unwrap().interpreter_frame_operand_stack_depths.last();
            match last {
                Some((last_frame, _)) => {
                    if pop_to_inclusive >= *last_frame {
                        self.guard.as_mut().unwrap().interpreter_frame_operand_stack_depths.pop().unwrap();
                        if already_popped {
                            todo!("mutli-pop?")
                        }
                        already_popped = true;
                    } else {
                        if !already_popped{
                            todo!("no frame to pop");
                        }
                        break;
                    }
                }
                None => break,
            }
        }
    }

    pub(crate) fn notify_frame_push(&mut self, next_frame_pointer: FramePointer) {
        // for _ in 0..self.guard.as_ref().unwrap().interpreter_frame_operand_stack_depths.len() {
        //     print!(" ");
        // }
        // println!("Push: {}", method_name);
        if let Some((frame_pointer, _)) = self.guard.as_ref().unwrap().interpreter_frame_operand_stack_depths.last() {
            assert!(*frame_pointer > next_frame_pointer);
        }
        self.guard.as_mut().unwrap().interpreter_frame_operand_stack_depths.push((next_frame_pointer, InterpreterFrameState {
            stack_depth: StackDepth(0),
            current_pc: ByteCodeOffset(0),
        }))
    }

    pub(crate) fn update_stack_depth(&mut self, current_pc: ByteCodeOffset, _frame_pointer: FramePointer, stack_depth: StackDepth) {
        let (_current_frame_pointer, state_mut) = self.guard.as_mut().unwrap().interpreter_frame_operand_stack_depths.last_mut().unwrap();
        state_mut.stack_depth = stack_depth;
        state_mut.current_pc = current_pc;
    }

    pub(crate) fn drop_guard(&mut self) {
        self.guard = None;
    }

    pub(crate) fn reacquire(&mut self) {
        self.guard = Some(self.stack.lock().unwrap());
    }

    pub fn within_guard<T>(&mut self, within: impl FnOnce() -> T) -> T {
        self.drop_guard();
        let res = within();
        self.reacquire();
        res
    }

    pub fn set_should_be_tracing_function_calls(&mut self) {
        self.guard.as_mut().unwrap().should_be_tracing_function_calls = true;
    }

    pub fn should_be_tracing_function_calls(&self) -> bool {
        self.guard.as_ref().unwrap().should_be_tracing_function_calls
    }

    pub fn thread_name_cached(&self) -> String{
        self.guard.as_ref().unwrap().thread_name_cached.clone()
    }
}

impl<'vm> HasRBPAndRSP for JavaStackGuard<'vm> {
    fn notify_guest_exit(&mut self, _rbp: NonNull<c_void>, _rsp: NonNull<c_void>) {
        self.reacquire()
    }

    fn notify_guest_enter(&mut self) {
        self.drop_guard();
    }

    fn rsp(&self) -> NonNull<c_void> {
        todo!()
    }

    fn rbp(&self) -> NonNull<c_void> {
        todo!()
    }

    fn ir_stack_ref(&self) -> &OwnedIRStack {
        &self.guard.as_ref().unwrap().owned_ir_stack
    }

    fn ir_stack_mut(&mut self) -> &mut OwnedIRStack {
        &mut self.guard.as_mut().unwrap().owned_ir_stack
    }
}


fn get_top_level_exit_ptr<'vm>(jvm: &'vm JVMState<'vm>) -> NonNull<c_void> {
    let ir_vm_state = &jvm.java_vm_state.ir;
    let top_level_ir_method_id = ir_vm_state.get_top_level_return_ir_method_id();
    ir_vm_state.lookup_ir_method_id_pointer(top_level_ir_method_id)
}