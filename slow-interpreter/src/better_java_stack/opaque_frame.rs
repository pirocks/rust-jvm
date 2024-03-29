use std::ptr::NonNull;
use std::sync::Arc;

use another_jit_vm_ir::ir_stack::{IRFrameMut, IRFrameRef, IsOpaque};
use gc_memory_layout_common::frame_layout::NativeStackframeMemoryLayout;
use runtime_class_stuff::RuntimeClass;
use rust_jvm_common::ByteCodeOffset;

use crate::{JVMState, StackEntryPush, WasException};
use crate::better_java_stack::{FramePointer};
use crate::better_java_stack::frame_iter::JavaFrameIterRefNew;
use crate::better_java_stack::frames::{HasFrame, PushableFrame};
use crate::better_java_stack::interpreter_frame::JavaInterpreterFrame;
use crate::better_java_stack::java_stack_guard::JavaStackGuard;
use crate::better_java_stack::native_frame::NativeFrame;
use crate::stack_entry::{JavaFramePush, NativeFramePush, OpaqueFramePush};

pub struct OpaqueFrame<'gc, 'k> {
    java_stack: &'k mut JavaStackGuard<'gc>,
    frame_pointer: FramePointer,
    //todo opaque frames might need operand stack at some point
    // stack_depth: Option<StackDepth>,
    //get/set/etc
}

impl<'gc, 'k> OpaqueFrame<'gc, 'k> {
    pub fn new_from_empty_stack(java_stack: &'k mut JavaStackGuard<'gc>) -> Self {
        assert!(!java_stack.has_been_used());
        java_stack.set_has_been_used();
        let frame_ptr = FramePointer(java_stack.ir_stack().native.mmaped_top);
        java_stack.debug_assert();
        Self {
            java_stack,
            frame_pointer: frame_ptr,
            // stack_depth: None,
        }
    }

    pub fn new_from_frame_pointer(java_stack: &'k mut JavaStackGuard<'gc>, frame_pointer: FramePointer) -> Self {
        Self {
            java_stack,
            frame_pointer,
            // stack_depth: None,
        }
    }
}


impl<'gc, 'k> HasFrame<'gc> for OpaqueFrame<'gc, 'k> {
    fn frame_ref(&self) -> IRFrameRef {
        todo!()
    }

    fn frame_mut(&mut self) -> IRFrameMut {
        todo!()
    }

    fn jvm(&self) -> &'gc JVMState<'gc> {
        self.java_stack.jvm()
    }

    fn num_locals(&self) -> Result<u16, IsOpaque> {
        Err(IsOpaque {})
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

    fn frame_iter(&self) -> JavaFrameIterRefNew<'gc, '_> {
        todo!()
    }

    fn class_pointer(&self) -> Result<Arc<RuntimeClass<'gc>>, IsOpaque> {
        todo!()
    }

    fn try_current_frame_pc(&self) -> Option<ByteCodeOffset> {
        todo!()
    }

    fn java_stack_ref(&self) -> &JavaStackGuard<'gc> {
        todo!()
    }

    fn java_stack_mut(&mut self) -> &mut JavaStackGuard<'gc> {
        &mut self.java_stack
    }
}

impl<'gc, 'k> PushableFrame<'gc> for OpaqueFrame<'gc, 'k> {
    fn push_frame<T>(&mut self, frame_to_write: StackEntryPush, within_push: impl FnOnce(&mut JavaStackGuard<'gc>) -> Result<T, WasException<'gc>>) -> Result<T, WasException<'gc>> {
        match frame_to_write {
            StackEntryPush::Java(java_frame) => {
                self.java_stack.push_java_frame(self.frame_pointer, self.next_frame_pointer(), java_frame, |frame| within_push(todo!()/*frame*/))
            }
            StackEntryPush::Native(_) => todo!(),
            StackEntryPush::Opaque(opaque) => {
                todo!()
                /*                self.java_stack.push_opaque_frame(self.frame_pointer, self.next_frame_pointer(), opaque,|java_stack_guard|within_push(java_stack_guard))*/
            }
        }
    }

    fn push_frame_opaque<T>(&mut self, opaque_frame_push: OpaqueFramePush, within_push: impl for<'l> FnOnce(&mut OpaqueFrame<'gc, 'l>) -> Result<T, WasException<'gc>>) -> Result<T, WasException<'gc>> {
        self.java_stack.push_opaque_frame(self.frame_pointer, self.next_frame_pointer(), opaque_frame_push, |opaque_frame| within_push(opaque_frame))
    }

    fn push_frame_java<T>(&mut self, java_frame_push: JavaFramePush, within_push: impl for<'l> FnOnce(&mut JavaInterpreterFrame<'gc, 'l>) -> Result<T, WasException<'gc>>) -> Result<T, WasException<'gc>> {
        self.java_stack.push_java_frame(self.frame_pointer, self.next_frame_pointer(), java_frame_push, |java_frame| within_push(java_frame))
    }

    fn push_frame_native<T>(&mut self, java_frame_push: NativeFramePush, within_push: impl for<'l> FnOnce(&mut NativeFrame<'gc, 'l>) -> Result<T, WasException<'gc>>) -> Result<T, WasException<'gc>> {
        self.java_stack.push_frame_native(self.frame_pointer, self.next_frame_pointer(), java_frame_push, |java_frame| within_push(java_frame))
    }
}
