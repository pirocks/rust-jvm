use std::mem::size_of;
use std::ptr::NonNull;

use libc::c_void;

use another_jit_vm_ir::ir_stack::{IRFrameMut, IRFrameRef};

use gc_memory_layout_common::layout::FRAME_HEADER_END_OFFSET;
use jvmti_jni_bindings::jlong;

use crate::{JVMState, OpaqueFrame, StackEntryPush, WasException};
use crate::better_java_stack::frame_iter::JavaFrameIterRefNew;
use crate::better_java_stack::FramePointer;
use crate::better_java_stack::frames::{HasFrame, IsOpaque, PushableFrame};
use crate::better_java_stack::interpreter_frame::JavaInterpreterFrame;
use crate::better_java_stack::java_stack_guard::JavaStackGuard;
use crate::interpreter_state::NativeFrameInfo;
use crate::rust_jni::interface::PerStackInterfaces;
use crate::stack_entry::{JavaFramePush, NativeFramePush, OpaqueFramePush};

pub struct NativeFrame<'gc, 'k> {
    java_stack: &'k mut JavaStackGuard<'gc>,
    frame_pointer: FramePointer,
    num_locals: u16,
}

impl<'gc, 'k> NativeFrame<'gc, 'k> {
    pub fn new_from_pointer(java_stack_guard: &'k mut JavaStackGuard<'gc>, frame_pointer: FramePointer, num_locals: u16) -> Self {
        let res = Self {
            java_stack: java_stack_guard,
            frame_pointer,
            num_locals,
        };
        res.debug_assert();
        res
    }

    pub fn debug_assert(&self) {
        let method_id = self.frame_ref().method_id().unwrap();
        self.java_stack.jvm().is_native_by_method_id(method_id);
        self.java_stack.debug_assert();
        assert!(self.frame_info_ref().native_local_refs.len() > 0);
    }

    pub fn frame_info_mut(&mut self) -> &mut NativeFrameInfo<'gc> {
        let num_locals = self.num_locals as usize;
        unsafe { (self.frame_ref().data(num_locals) as *mut c_void as *mut NativeFrameInfo<'gc>).as_mut().unwrap() }
    }

    pub fn frame_info_ref(&self) -> &NativeFrameInfo<'gc> {
        let num_locals = self.num_locals as usize;
        unsafe {
            (self.frame_ref().data(num_locals) as *const c_void as *const NativeFrameInfo<'gc>).as_ref().unwrap()
        }
    }

    pub(crate) fn stack_jni_interface(&mut self) -> &mut PerStackInterfaces {
        self.java_stack.stack_jni_interface()
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

    fn num_locals(&self) -> Result<u16,IsOpaque> {
        todo!()
    }

    fn max_stack(&self) -> u16 {
        todo!()
    }

    fn next_frame_pointer(&self) -> FramePointer {
        let raw = unsafe {
            self.frame_pointer.0.as_ptr()
                .sub(FRAME_HEADER_END_OFFSET) //header
                .sub((self.num_locals as usize * size_of::<jlong>()) as usize)//locals
                .sub(1 * size_of::<*const c_void>())
        };//frame info pointer
        FramePointer(NonNull::new(raw).unwrap())
    }

    fn debug_assert(&self) {
        todo!()
    }

    fn frame_iter(&self) -> JavaFrameIterRefNew<'gc, '_> {
        JavaFrameIterRefNew::new(self.java_stack, self.frame_pointer)
    }
}

impl<'gc, 'k> PushableFrame<'gc> for NativeFrame<'gc, 'k> {
    fn push_frame<T>(&mut self, frame_to_write: StackEntryPush, within_push: impl FnOnce(&mut JavaStackGuard<'gc>) -> Result<T, WasException<'gc>>) -> Result<T, WasException<'gc>> {
        todo!()
    }

    fn push_frame_opaque<T>(&mut self, opaque_frame: OpaqueFramePush, within_push: impl for<'l> FnOnce(&mut OpaqueFrame<'gc, 'l>) -> Result<T, WasException<'gc>>) -> Result<T, WasException<'gc>> {
        todo!()
    }

    fn push_frame_java<T>(&mut self, java_frame_push: JavaFramePush, within_push: impl for<'l> FnOnce(&mut JavaInterpreterFrame<'gc, 'l>) -> Result<T, WasException<'gc>>) -> Result<T, WasException<'gc>> {
        self.java_stack.push_java_frame(self.frame_pointer, self.next_frame_pointer(), java_frame_push, |java_frame| within_push(java_frame))
    }

    fn push_frame_native<T>(&mut self, native_frame_push: NativeFramePush, within_push: impl for<'l> FnOnce(&mut NativeFrame<'gc, 'l>) -> Result<T, WasException<'gc>>) -> Result<T, WasException<'gc>> {
        self.java_stack.push_frame_native(self.frame_pointer, self.next_frame_pointer(), native_frame_push, |native_frame| within_push(native_frame))
    }
}