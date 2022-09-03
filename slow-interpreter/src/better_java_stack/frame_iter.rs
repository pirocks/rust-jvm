use std::hash::Hash;

use another_jit_vm_ir::ir_stack::{IRFrameMut, IRFrameRef};

use crate::better_java_stack::FramePointer;
use crate::better_java_stack::frames::HasFrame;
use crate::better_java_stack::java_stack_guard::JavaStackGuard;
use crate::JVMState;

pub struct FrameIterFrameRef<'gc, 'k> {
    java_stack: &'k JavaStackGuard<'gc>,
    frame_pointer: FramePointer,
}

impl<'gc, 'k> HasFrame<'gc> for FrameIterFrameRef<'gc, 'k> {
    fn frame_ref(&self) -> IRFrameRef {
        todo!()
    }

    fn frame_mut(&mut self) -> IRFrameMut {
        todo!()
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

    fn frame_iter(&self) -> JavaFrameIter {
        todo!()
    }
}
