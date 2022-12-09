use std::mem::size_of;
use std::ops::Deref;
use std::ptr::NonNull;
use std::sync::Arc;

use libc::c_void;

use another_jit_vm::FramePointerOffset;
use another_jit_vm_ir::ir_stack::{IRFrameMut, IRFrameRef, IsOpaque};
use gc_memory_layout_common::frame_layout::FRAME_HEADER_END_OFFSET;
use java5_verifier::SimplifiedVType;
use runtime_class_stuff::RuntimeClass;
use rust_jvm_common::{ByteCodeOffset, StackNativeJavaValue};
use rust_jvm_common::vtype::VType;

use crate::{JVMState, OpaqueFrame, StackEntryPush, WasException};
use crate::better_java_stack::{FramePointer, JavaStackGuard};
use crate::better_java_stack::frame_iter::JavaFrameIterRefNew;
use crate::better_java_stack::frames::{HasFrame, PushableFrame};
use crate::better_java_stack::interpreter_frame::JavaInterpreterFrame;
use crate::better_java_stack::native_frame::NativeFrame;
use crate::stack_entry::{JavaFramePush, NativeFramePush, OpaqueFramePush};

pub struct JavaExitFrame<'gc, 'k> {
    // Interpreter{
    java_stack: &'k mut JavaStackGuard<'gc>,
    frame_pointer: FramePointer,
    stack_pointer: NonNull<c_void>,
    current_pc: Option<ByteCodeOffset>,
    // num_locals: u16,
    // max_stack: u16,
    // stack_depth: Option<StackDepth>,
    // }
    // are there any other possible exits?
}

impl<'gc, 'k> JavaExitFrame<'gc, 'k> {
    pub fn new(java_stack_guard: &'k mut JavaStackGuard<'gc>, frame_pointer: FramePointer, stack_pointer: NonNull<c_void>, current_pc: Option<ByteCodeOffset>) -> Self {
        Self {
            java_stack: java_stack_guard,
            frame_pointer,
            // num_locals: todo!(),
            // max_stack: todo!(),
            // stack_depth: todo!()
            stack_pointer,
            current_pc,
        }
    }

    pub fn to_interpreter_frame<T>(&mut self, within_interpreter: impl for<'k2> FnOnce(&mut JavaInterpreterFrame<'gc, 'k2>) -> T) -> T {
        JavaInterpreterFrame::from_frame_pointer_interpreter(self.java_stack, self.frame_pointer, |frame| { Ok(within_interpreter(frame)) }).unwrap()
    }

    pub fn read_target(&self, frame_point_offset: FramePointerOffset) -> StackNativeJavaValue<'gc> {
        unsafe { self.frame_pointer.as_const_ptr().sub(frame_point_offset.0).cast::<StackNativeJavaValue<'gc>>().read() }
    }

    pub fn assert_current_pc_is(&self, current_pc: Option<ByteCodeOffset>) {
        assert_eq!(self.current_pc, current_pc);
    }

    //todo duplication:

    pub fn full_frame_available(&self, jvm: &'gc JVMState<'gc>) -> bool {
        let method_id = self.frame_ref().method_id().unwrap();
        let pc = self.current_pc.unwrap();
        let read_guard = &jvm.function_frame_type_data.read().unwrap().tops;
        let function_frame_type = read_guard.get(&method_id).unwrap();
        let frame = function_frame_type.get(&pc).unwrap();
        frame.try_unwrap_full_frame().is_some()
    }

    pub fn local_var_simplified_types(&self, jvm: &'gc JVMState<'gc>) -> Vec<SimplifiedVType> {
        let method_id = self.frame_ref().method_id().unwrap();
        let pc = self.current_pc.unwrap();
        let read_guard = &jvm.function_frame_type_data.read().unwrap().tops;
        let function_frame_type = read_guard.get(&method_id).unwrap();
        function_frame_type.get(&pc).unwrap().unwrap_partial_inferred_frame().local_vars.clone()
    }

    pub fn local_var_types(&self, jvm: &'gc JVMState<'gc>) -> Vec<VType> {
        let method_id = self.frame_ref().method_id().unwrap();
        let pc = self.current_pc.unwrap();
        let read_guard = &jvm.function_frame_type_data.read().unwrap().tops;
        let function_frame_type = read_guard.get(&method_id).unwrap();
        function_frame_type.get(&pc).unwrap().unwrap_full_frame().locals.deref().clone()
    }

    pub fn operand_stack_simplified_types(&self, jvm: &'gc JVMState<'gc>) -> Vec<SimplifiedVType> {
        let method_id = self.frame_ref().method_id().expect("local vars should have method id probably");
        let pc = self.current_pc.unwrap();
        let function_frame_data_guard = &jvm.function_frame_type_data.read().unwrap().no_tops;
        let function_frame_data = function_frame_data_guard.get(&method_id).unwrap();
        let frame = function_frame_data.get(&pc).unwrap();//todo this get frame thing is duped in a bunch of places
        frame.unwrap_partial_inferred_frame().operand_stack.iter().rev().map(|vtype| *vtype).collect()
    }

    pub fn operand_stack_types(&self, jvm: &'gc JVMState<'gc>) -> Vec<VType> {
        let method_id = self.frame_ref().method_id().expect("local vars should have method id probably");
        let pc = self.current_pc.unwrap();
        let function_frame_data_guard = &jvm.function_frame_type_data.read().unwrap().no_tops;
        let function_frame_data = function_frame_data_guard.get(&method_id).unwrap();
        let frame = function_frame_data.get(&pc).unwrap();//todo this get frame thing is duped in a bunch of places
        frame.unwrap_full_frame().stack_map.iter().rev().map(|vtype| *vtype).collect()
    }

    pub fn raw_local_var_get(&self, i: u16) -> u64 {
        //todo use layout
        unsafe {
            self.frame_pointer.as_const_ptr()
                .sub(FRAME_HEADER_END_OFFSET)
                .sub((i as usize * size_of::<StackNativeJavaValue>()) as usize)
                .cast::<u64>()
                .read()
        }
    }

    pub fn raw_operand_stack_get(&self, i: u16) -> u64 {
        //todo use layout
        let max_locals = self.jvm().max_locals_by_method_id(self.frame_ref().method_id().unwrap());
        unsafe {
            self.frame_pointer.as_const_ptr()
                .sub(FRAME_HEADER_END_OFFSET)
                .sub(((i as usize + max_locals as usize) * size_of::<StackNativeJavaValue>()) as usize)
                .cast::<u64>()
                .read()
        }
    }
}


impl<'gc, 'k> HasFrame<'gc> for JavaExitFrame<'gc, 'k> {
    fn java_stack_ref(&self) -> &JavaStackGuard<'gc> {
        todo!()
    }

    fn java_stack_mut(&mut self) -> &mut JavaStackGuard<'gc> {
        todo!()
    }

    fn frame_ref(&self) -> IRFrameRef {
        IRFrameRef {
            ptr: self.frame_pointer.0.into(),
            _ir_stack: self.java_stack.ir_stack(),
        }
    }

    fn frame_mut(&mut self) -> IRFrameMut {
        IRFrameMut {
            ptr: self.frame_pointer.0,
            ir_stack: todo!()/*&mut self.java_stack.owned_ir_stack*/,
        }
    }

    fn jvm(&self) -> &'gc JVMState<'gc> {
        self.java_stack.jvm()
    }

    fn num_locals(&self) -> Result<u16, IsOpaque> {
        let method_id = self.frame_ref().method_id()?;
        Ok(self.jvm().max_locals_by_method_id(method_id))
    }

    fn max_stack(&self) -> u16 {
        let jvm = self.jvm();
        let method_id = self.frame_ref().method_id().unwrap();
        let (rc, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
        let view = rc.view();
        let method_view = view.method_view_i(method_i);
        let code = method_view.code_attribute().unwrap();
        code.max_stack
    }

    fn next_frame_pointer(&self) -> FramePointer {
        FramePointer(self.stack_pointer)
        /*unsafe {
            FramePointer(NonNull::new(self.frame_pointer.0.as_ptr()
                .sub(FRAME_HEADER_END_OFFSET)
                .sub((self.num_locals as usize * size_of::<NativeJavaValue<'gc>>()) as usize)
                .sub((self.max_stack as usize * size_of::<NativeJavaValue<'gc>>()) as usize)).unwrap())
        }*/
    }

    fn debug_assert(&self) {
        self.java_stack.debug_assert();
    }

    fn class_pointer(&self) -> Result<Arc<RuntimeClass<'gc>>, IsOpaque> {
        todo!()
    }

    fn try_current_frame_pc(&self) -> Option<ByteCodeOffset> {
        todo!()
    }

    fn frame_iter(&self) -> JavaFrameIterRefNew<'gc, '_> {
        JavaFrameIterRefNew::new(self.java_stack, self.frame_pointer, self.current_pc)
    }
}

impl<'gc, 'k> PushableFrame<'gc> for JavaExitFrame<'gc, 'k> {
    fn push_frame<T>(&mut self, _frame_to_write: StackEntryPush, _within_push: impl FnOnce(&mut JavaStackGuard<'gc>) -> Result<T, WasException<'gc>>) -> Result<T, WasException<'gc>> {
        todo!()
    }

    fn push_frame_opaque<T>(&mut self, opaque_frame_push: OpaqueFramePush, within_push: impl for<'l> FnOnce(&mut OpaqueFrame<'gc, 'l>) -> Result<T, WasException<'gc>>) -> Result<T, WasException<'gc>> {
        self.java_stack.push_opaque_frame(self.frame_pointer, self.next_frame_pointer(), opaque_frame_push, |opaque_frame| {
            within_push(opaque_frame)
        })
    }

    fn push_frame_java<T>(&mut self, java_frame_push: JavaFramePush, within_push: impl for<'l> FnOnce(&mut JavaInterpreterFrame<'gc, 'l>) -> Result<T, WasException<'gc>>) -> Result<T, WasException<'gc>> {
        self.java_stack.push_java_frame(self.frame_pointer, self.next_frame_pointer(), java_frame_push, |java_frame| within_push(java_frame))
    }

    fn push_frame_native<T>(&mut self, native_frame_push: NativeFramePush, within_push: impl for<'l> FnOnce(&mut NativeFrame<'gc, 'l>) -> Result<T, WasException<'gc>>) -> Result<T, WasException<'gc>> {
        self.java_stack.push_frame_native(self.frame_pointer, self.next_frame_pointer(), native_frame_push, |native_frame| within_push(native_frame))
    }
}