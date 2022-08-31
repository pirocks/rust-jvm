use another_jit_vm_ir::ir_stack::{IRFrameMut, IRFrameRef};

use crate::better_java_stack::FramePointer;
use crate::better_java_stack::frames::HasFrame;
use crate::better_java_stack::java_stack_guard::JavaStackGuard;
use crate::JVMState;

pub struct NativeFrame<'gc, 'k> {
    java_stack: &'k mut JavaStackGuard<'gc>,
    frame_pointer: FramePointer,
}

impl <'gc,'k> NativeFrame<'gc,'k>{
    pub fn new_from_pointer(java_stack_guard: &'k mut JavaStackGuard<'gc>, frame_pointer: FramePointer) -> Self{
        Self{
            java_stack: java_stack_guard,
            frame_pointer
        }
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