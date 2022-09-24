use std::sync::Arc;
use another_jit_vm_ir::ir_stack::{IRFrameMut, IRFrameRef, IsOpaque};
use runtime_class_stuff::RuntimeClass;
use rust_jvm_common::ByteCodeOffset;

use crate::better_java_stack::{FramePointer, JavaStackGuard};
use crate::better_java_stack::frames::{HasFrame};
use crate::JVMState;

pub struct RemoteFrame<'gc, 'k> {
    java_stack: &'k mut JavaStackGuard<'gc>,
    frame_ptr: FramePointer,
}
// don't have the function call vec thing

impl<'gc, 'k> RemoteFrame<'gc, 'k> {
    pub fn new(java_stack: &'k mut JavaStackGuard<'gc>, frame_ptr: FramePointer) -> Self {
        Self {
            java_stack,
            frame_ptr,
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

    fn class_pointer(&self) -> Result<Arc<RuntimeClass<'gc>>, IsOpaque> {
        todo!()
    }

    fn try_current_frame_pc(&self) -> Option<ByteCodeOffset> {
        //todo we can provide accurate values here by converting the rip to bytecode
        None
    }

    fn java_stack_ref(&self) -> &JavaStackGuard<'gc> {
        &self.java_stack
    }

    fn java_stack_mut(&mut self) -> &mut JavaStackGuard<'gc> {
        &mut self.java_stack
    }
}

