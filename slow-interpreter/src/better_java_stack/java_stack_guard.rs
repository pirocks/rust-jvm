use std::ffi::c_void;
use std::ptr::NonNull;
use std::sync::{Mutex, MutexGuard};

use itertools::Itertools;

use another_jit_vm_ir::ir_stack::OwnedIRStack;
use another_jit_vm_ir::WasException;
use rust_jvm_common::loading::LoaderName;

use crate::{JavaValueCommon, JVMState, MethodResolverImpl};
use crate::better_java_stack::{FramePointer, JavaStack};
use crate::better_java_stack::interpreter_frame::JavaInterpreterFrame;
use crate::better_java_stack::native_frame::NativeFrame;
use crate::better_java_stack::opaque_frame::OpaqueFrame;
use crate::interpreter_state::{NativeFrameInfo, OpaqueFrameInfo};
use crate::ir_to_java_layer::java_stack::OpaqueFrameIdOrMethodID;
use crate::stack_entry::{JavaFramePush, NativeFramePush, OpaqueFramePush};

pub struct JavaStackGuard<'vm> {
    stack: &'vm Mutex<JavaStack<'vm>>,
    guard: Option<MutexGuard<'vm, JavaStack<'vm>>>,
    jvm: &'vm JVMState<'vm>,
    current_frame_pointer: FramePointer,
}

impl<'vm> JavaStackGuard<'vm> {
    pub(crate) fn ir_stack_ref(&self) -> &OwnedIRStack {
        &self.guard.as_ref().unwrap().owned_ir_stack
    }

    pub(crate) fn ir_stack_mut(&mut self) -> &mut OwnedIRStack {
        &mut self.guard.as_mut().unwrap().owned_ir_stack
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

    //todo I really need an init function which just creates the mutex and everything in one place
    pub fn new_from_empty_stack<T>(jvm: &'vm JVMState<'vm>, stack: &'vm Mutex<JavaStack<'vm>>, with_initial_opaque_frame: impl for<'l, 'k> FnOnce(&'l mut OpaqueFrame<'vm, 'k>) -> Result<T, WasException> + 'vm) -> Result<T, WasException> {
        let guard = stack.lock().unwrap();
        if guard.has_been_used {
            panic!()
        }
        let mmapped_top = guard.owned_ir_stack.native.mmaped_top;
        let mut res = Self {
            stack,
            guard: Some(guard),
            jvm,
            current_frame_pointer: FramePointer(mmapped_top),
        };
        let mut opaque_frame = OpaqueFrame::new_from_empty_stack(&mut res);
        with_initial_opaque_frame(&mut opaque_frame)
    }

    pub fn new_from_prev_with_new_frame_pointer(old: Self, new_frame_pointer: FramePointer) -> Self {
        let Self { stack, guard, jvm, current_frame_pointer } = old;
        Self {
            stack,
            guard,
            jvm,
            current_frame_pointer: new_frame_pointer,
        }
    }

    pub fn debug_assert(&self) {
        self.assert_interpreter_frame_operand_stack_depths_sorted();
    }

    fn assert_interpreter_frame_operand_stack_depths_sorted(&self) {
        self.guard.as_ref().unwrap().assert_interpreter_frame_operand_stack_depths_sorted();
    }

    fn enter_guest(&mut self) {
        todo!()
    }

    fn exit_guest(&mut self) {
        todo!()
    }

    // within guerst java
    pub fn within_guest<T>(&mut self, within_native: impl FnOnce(&mut JavaStackGuard<'vm>) -> Result<T, WasException>) {
        self.enter_guest();
        todo!();
        self.exit_guest();
    }

    pub fn current_loader(&self, jvm: &'vm JVMState<'vm>) -> LoaderName {
        LoaderName::BootstrapLoader
    }

    fn current_frame_ptr(&self) -> FramePointer {
        todo!("current_frame_pointer needs updating")
        /*self.current_frame_pointer*/
    }

    pub fn jvm(&self) -> &'vm JVMState<'vm> {
        self.jvm
    }


    pub fn push_java_frame<'k, T>(&'k mut self,
                                  current_frame_pointer: FramePointer,
                                  next_frame_pointer: FramePointer,
                                  java_stack_entry: JavaFramePush,
                                  within_pushed: impl for<'l> FnOnce(&mut JavaInterpreterFrame<'vm, 'l>) -> Result<T, WasException>,
    ) -> Result<T, WasException> {
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
            data.push(unsafe { local_var.to_native().as_u64 });
        }
        for jv in operand_stack {
            data.push(unsafe { jv.to_native().as_u64 });
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
        let (rc, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
        let view = rc.view();
        let method_view = view.method_view_i(method_i);
        let code = method_view.code_attribute().unwrap();
        JavaInterpreterFrame::from_frame_pointer_interpreter(jvm, self, next_frame_pointer, |within|{
            within_pushed(within)
        })
    }

    pub fn push_frame_native<'k, T>(&'k mut self,
                                current_frame_pointer: FramePointer,
                                next_frame_pointer: FramePointer,
                                stack_entry: NativeFramePush,
                                within_pushed: impl FnOnce(&mut NativeFrame<'vm,'k>) -> Result<T, WasException>,
    ) -> Result<T, WasException> {
        let NativeFramePush { method_id, native_local_refs, local_vars, operand_stack } = stack_entry;
        let jvm = self.jvm();
        let top_level_exit_ptr = get_top_level_exit_ptr(jvm);
        let method_resolver_impl = MethodResolverImpl { jvm, loader: LoaderName::BootstrapLoader/*todo fix*/ };
        jvm.java_vm_state.add_method_if_needed(jvm, &method_resolver_impl, method_id, false);
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
            self.guard.as_mut().unwrap().owned_ir_stack.write_frame(
                next_frame_pointer.0,
                top_level_exit_ptr.as_ptr(),
                current_frame_pointer.as_ptr(),
                Some(ir_method_id),
                wrapped_method_id.to_native(),
                data.as_slice(),
            );
        }
        let mut frame = NativeFrame::new_from_pointer(self, next_frame_pointer);
        let res = within_pushed(&mut frame)?;
        Ok(res)
    }

    pub fn push_opaque_frame<'k, T>(&'k mut self,
                                    current_frame_pointer: FramePointer,
                                    next_frame_pointer: FramePointer,
                                    opaque_frame: OpaqueFramePush,
                                    within_pushed: impl for<'l> FnOnce(&mut OpaqueFrame<'vm, 'l>) -> Result<T, WasException>,
    ) -> Result<T, WasException> {
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
        let mut frame = OpaqueFrame::new_from_frame_pointer(self, next_frame_pointer);
        let res = within_pushed(&mut frame)?;
        //todo zero the rest
        Ok(res)
    }
}


fn get_top_level_exit_ptr<'vm>(jvm: &'vm JVMState<'vm>) -> NonNull<c_void> {
    let ir_vm_state = &jvm.java_vm_state.ir;
    let top_level_ir_method_id = ir_vm_state.get_top_level_return_ir_method_id();
    ir_vm_state.lookup_ir_method_id_pointer(top_level_ir_method_id)
}