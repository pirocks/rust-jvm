use std::mem::size_of;
use std::ptr::NonNull;
use another_jit_vm_ir::ir_stack::{IRFrameMut, IRFrameRef};

use gc_memory_layout_common::layout::FRAME_HEADER_END_OFFSET;
use rust_jvm_common::NativeJavaValue;
use crate::better_java_stack::{FramePointer, JavaStackGuard, StackDepth};
use crate::{JVMState, OpaqueFrame, StackEntryPush, WasException};
use crate::better_java_stack::frame_iter::JavaFrameIterRefNew;
use crate::better_java_stack::frames::{HasFrame, PushableFrame};
use crate::better_java_stack::interpreter_frame::JavaInterpreterFrame;
use crate::better_java_stack::native_frame::NativeFrame;
use crate::stack_entry::{JavaFramePush, NativeFramePush, OpaqueFramePush};

pub struct JavaExitFrame<'gc, 'k> {
    // Interpreter{
        java_stack: &'k mut JavaStackGuard<'gc>,
        frame_pointer: FramePointer,
        num_locals: u16,
        max_stack: u16,
        stack_depth: Option<StackDepth>,
    // }
    // are there any other possible exits?
}


impl<'gc, 'k> HasFrame<'gc> for JavaExitFrame<'gc, 'k> {
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
        self.java_stack.jvm()
    }

    fn num_locals(&self) -> u16 {
        self.num_locals
    }

    fn max_stack(&self) -> u16 {
        self.max_stack
    }

    fn next_frame_pointer(&self) -> FramePointer {
        unsafe {
            FramePointer(NonNull::new(self.frame_pointer.0.as_ptr()
                .sub(FRAME_HEADER_END_OFFSET)
                .sub((self.num_locals as usize * size_of::<NativeJavaValue<'gc>>()) as usize)
                .sub((self.max_stack as usize * size_of::<NativeJavaValue<'gc>>()) as usize)).unwrap())
        }
    }

    fn debug_assert(&self) {
        self.java_stack.debug_assert();
    }

    fn frame_iter(&self) -> JavaFrameIterRefNew<'gc, '_> {
        todo!()
    }
}

impl<'gc, 'k> PushableFrame<'gc> for JavaExitFrame<'gc, 'k> {
    fn push_frame<T>(&mut self, frame_to_write: StackEntryPush, within_push: impl FnOnce(&mut JavaStackGuard<'gc>) -> Result<T, WasException<'gc>>) -> Result<T, WasException<'gc>> {
        todo!()
    }

    fn push_frame_opaque<T>(&mut self, opaque_frame_push: OpaqueFramePush, within_push: impl for<'l> FnOnce(&mut OpaqueFrame<'gc, 'l>) -> Result<T, WasException<'gc>>) -> Result<T, WasException<'gc>> {
        self.java_stack.push_opaque_frame(self.frame_pointer, self.next_frame_pointer(), opaque_frame_push, |opaque_frame| {
            within_push(opaque_frame)
        })
    }

    fn push_frame_java<T>(&mut self, java_frame: JavaFramePush, within_push: impl for<'l> FnOnce(&mut JavaInterpreterFrame<'gc, 'l>) -> Result<T, WasException<'gc>>) -> Result<T, WasException<'gc>> {
        todo!()
    }

    fn push_frame_native<T>(&mut self, java_frame: NativeFramePush, within_push: impl for<'l> FnOnce(&mut NativeFrame<'gc, 'l>) -> Result<T, WasException<'gc>>) -> Result<T, WasException<'gc>> {
        todo!()
    }
}