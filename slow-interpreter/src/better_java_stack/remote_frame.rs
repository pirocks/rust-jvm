use another_jit_vm_ir::ir_stack::{IRFrameMut, IRFrameRef};
use crate::better_java_stack::{FramePointer, JavaStackGuard};
use crate::better_java_stack::frames::HasFrame;
use crate::JVMState;

pub struct RemoteFrame<'gc, 'k> {
    java_stack: &'k mut JavaStackGuard<'gc>,
    frame_ptr: FramePointer,
    num_locals: u16,
    max_stack: u16,
    current_operand_stack_depth: u16,
}
// don't have the function call vec thing

impl<'gc, 'k> HasFrame<'gc> for RemoteFrame<'gc, 'k> {
    fn frame_ref(&self) -> IRFrameRef {
        IRFrameRef {
            ptr: self.frame_ptr.0.into(),
            _ir_stack: todo!(),
        }
    }

    fn frame_mut(&mut self) -> IRFrameMut {
        IRFrameMut {
            ptr: self.frame_ptr.0,
            ir_stack: todo!(),
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
        todo!()
    }

    fn debug_assert(&self) {
        self.java_stack.debug_assert();
    }
}
