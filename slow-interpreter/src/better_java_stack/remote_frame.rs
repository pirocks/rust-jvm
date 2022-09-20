use another_jit_vm_ir::ir_stack::{IRFrameMut, IRFrameRef};

use crate::better_java_stack::{FramePointer, JavaStackGuard};
use crate::better_java_stack::frame_iter::JavaFrameIterRefNew;
use crate::better_java_stack::frames::{HasFrame, IsOpaque};
use crate::JVMState;

pub struct RemoteFrame<'gc, 'k> {
    java_stack: &'k mut JavaStackGuard<'gc>,
    frame_ptr: FramePointer,
}
// don't have the function call vec thing

impl <'gc, 'k> RemoteFrame<'gc,'k>{
    pub fn new(java_stack: &'k mut JavaStackGuard<'gc>, frame_ptr: FramePointer) -> Self{
        Self {
            java_stack,
            frame_ptr
        }
    }
}

impl<'gc, 'k> HasFrame<'gc> for RemoteFrame<'gc, 'k> {
    fn frame_ref(&self) -> IRFrameRef {
        IRFrameRef {
            ptr: self.frame_ptr.0.into(),
            _ir_stack: self.java_stack.ir_stack_ref(),
        }
    }

    fn frame_mut(&mut self) -> IRFrameMut {
        IRFrameMut {
            ptr: self.frame_ptr.0,
            ir_stack: self.java_stack.ir_stack_mut(),
        }
    }

    fn jvm(&self) -> &'gc JVMState<'gc> {
        self.java_stack.jvm()
    }

    fn num_locals(&self) -> Result<u16, IsOpaque> {
        todo!()
    }

    fn max_stack(&self) -> u16 {
        todo!()
    }

    fn next_frame_pointer(&self) -> FramePointer {
        todo!()
    }

    fn debug_assert(&self) {
        self.java_stack.debug_assert();
    }

    fn frame_iter(&self) -> JavaFrameIterRefNew<'gc, '_> {
        todo!()
    }
}
