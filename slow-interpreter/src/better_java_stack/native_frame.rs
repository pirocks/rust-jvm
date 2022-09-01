use another_jit_vm_ir::ir_stack::{IRFrameMut, IRFrameRef};
use another_jit_vm_ir::WasException;

use crate::better_java_stack::FramePointer;
use crate::better_java_stack::frames::{HasFrame, PushableFrame};
use crate::better_java_stack::java_stack_guard::JavaStackGuard;
use crate::{JVMState, OpaqueFrame, StackEntryPush};
use crate::better_java_stack::interpreter_frame::JavaInterpreterFrame;
use crate::stack_entry::{JavaFramePush, NativeFramePush, OpaqueFramePush};

pub struct NativeFrame<'gc, 'k> {
    java_stack: &'k mut JavaStackGuard<'gc>,
    frame_pointer: FramePointer,
}

impl<'gc, 'k> NativeFrame<'gc, 'k> {
    pub fn new_from_pointer(java_stack_guard: &'k mut JavaStackGuard<'gc>, frame_pointer: FramePointer) -> Self {
        let res = Self {
            java_stack: java_stack_guard,
            frame_pointer,
        };
        res.debug_assert();
        res
    }

    pub fn debug_assert(&self) {
        let method_id = self.frame_ref().method_id().unwrap();
        self.java_stack.jvm().is_native_by_method_id(method_id);
        self.java_stack.debug_assert();
    }
}


impl<'gc, 'k> HasFrame<'gc> for NativeFrame<'gc, 'k> {
    fn frame_ref(&self) -> IRFrameRef {
        IRFrameRef {
            ptr: self.frame_pointer.0.into(),
            _ir_stack: self.java_stack.ir_stack_ref(),
        }
    }

    fn frame_mut(&mut self) -> IRFrameMut {
        IRFrameMut {
            ptr: self.frame_pointer.0.into(),
            ir_stack: self.java_stack.ir_stack_mut(),
        }
    }

    fn jvm(&self) -> &'gc JVMState<'gc> {
        todo!()
    }

    fn num_locals(&self) -> u16 {
        todo!()
    }

    fn max_stack(&self) -> u16 {
        todo!()
    }

    fn next_frame_pointer(&self) -> FramePointer {
        todo!()
    }

    fn debug_assert(&self) {
        todo!()
    }
}

impl<'gc, 'k> PushableFrame<'gc> for NativeFrame<'gc, 'k> {
    fn push_frame<T>(&mut self, frame_to_write: StackEntryPush, within_push: impl FnOnce(&mut JavaStackGuard<'gc>) -> Result<T, WasException>) -> Result<T, WasException> {
        todo!()
    }

    fn push_frame_opaque<T>(&mut self, opaque_frame: OpaqueFramePush, within_push: impl for<'l> FnOnce(&mut OpaqueFrame<'gc, 'l>) -> Result<T, WasException>) -> Result<T, WasException> {
        todo!()
    }

    fn push_frame_java<T>(&mut self, java_frame: JavaFramePush, within_push: impl for<'l> FnOnce(&mut JavaInterpreterFrame<'gc, 'l>) -> Result<T, WasException>) -> Result<T, WasException> {
        todo!()
    }

    fn push_frame_native<T>(&mut self, java_frame: NativeFramePush, within_push: impl for<'l> FnOnce(&mut NativeFrame<'gc, 'l>) -> Result<T, WasException>) -> Result<T, WasException> {
        todo!()
    }
}