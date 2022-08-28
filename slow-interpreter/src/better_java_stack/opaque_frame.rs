use std::ptr::NonNull;
use another_jit_vm_ir::ir_stack::{IRFrameMut, IRFrameRef};
use another_jit_vm_ir::WasException;
use gc_memory_layout_common::layout::NativeStackframeMemoryLayout;
use crate::better_java_stack::{FramePointer, StackDepth};
use crate::better_java_stack::frames::{HasFrame, PushableFrame};
use crate::better_java_stack::java_stack_guard::JavaStackGuard;
use crate::{JVMState, StackEntryPush};

pub struct OpaqueFrame<'gc, 'k> {
    java_stack: &'k mut JavaStackGuard<'gc>,
    frame_pointer: FramePointer,
    stack_depth: Option<StackDepth>,
    //get/set/etc
}

impl <'gc, 'k> OpaqueFrame<'gc, 'k> {
    pub fn new_from_empty_stack(java_stack: &'k mut JavaStackGuard<'gc>) -> Self {
        assert!(!java_stack.has_been_used());
        java_stack.set_has_been_used();
        let frame_ptr = FramePointer(java_stack.ir_stack().native.mmaped_top);
        java_stack.debug_assert();
        Self{
            java_stack,
            frame_pointer: frame_ptr,
            stack_depth: None
        }
    }
}


impl <'gc, 'k> HasFrame<'gc> for OpaqueFrame<'gc, 'k>{
    fn frame_ref(&self) -> IRFrameRef {
        todo!()
    }

    fn frame_mut(&mut self) -> IRFrameMut {
        todo!()
    }

    fn jvm(&self) -> &'gc JVMState<'gc> {
        self.java_stack.jvm()
    }

    fn num_locals(&self) -> u16 {
        0
    }

    fn max_stack(&self) -> u16 {
        todo!()
    }

    fn next_frame_pointer(&self) -> FramePointer {
        unsafe { FramePointer(NonNull::new(self.frame_pointer.0.as_ptr().sub(NativeStackframeMemoryLayout { num_locals: 0 }.full_frame_size())).unwrap()) }
    }

    fn debug_assert(&self) {
        self.java_stack.debug_assert();
    }
}

impl <'gc, 'k> PushableFrame<'gc> for OpaqueFrame<'gc, 'k>{
    fn push_frame<T>(&mut self, frame_to_write: StackEntryPush, within_push: impl FnOnce(&mut JavaStackGuard<'gc>) -> Result<T, WasException>) -> Result<T, WasException> {
        match frame_to_write {
            StackEntryPush::Java(java_frame) => {
                self.java_stack.push_java_frame(self.frame_pointer,self.next_frame_pointer(),java_frame,|java_stack_guard|within_push(java_stack_guard))
            }
            StackEntryPush::Native(_) => todo!(),
            StackEntryPush::Opaque(_) => todo!(),
        }
    }

}