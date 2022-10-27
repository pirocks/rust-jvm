use std::mem::size_of;
use std::ptr::NonNull;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use another_jit_vm_ir::ir_stack::{IRFrameMut, IRFrameRef, IsOpaque};
use another_jit_vm_ir::RBPAndRSP;
use classfile_view::view::ClassView;
use gc_memory_layout_common::frame_layout::FRAME_HEADER_END_OFFSET;
use runtime_class_stuff::RuntimeClass;
use rust_jvm_common::{ByteCodeOffset, MethodI, StackNativeJavaValue};
use rust_jvm_common::loading::LoaderName;
use rust_jvm_common::runtime_type::RuntimeType;

use crate::{JVMState, OpaqueFrame, StackEntryPush, WasException};
use crate::better_java_stack::{FramePointer, JavaStackGuard, StackDepth};
use crate::better_java_stack::frames::{HasFrame, PushableFrame};
use crate::better_java_stack::native_frame::NativeFrame;
use crate::interpreter::real_interpreter_state::InterpreterJavaValue;
use crate::stack_entry::{JavaFramePush, NativeFramePush, OpaqueFramePush};

//todo need to merge real interpreter state into this and update operand stack depth as needed with java stack guard
pub struct JavaInterpreterFrame<'gc, 'k> {
    pub(crate) java_stack: &'k mut JavaStackGuard<'gc>,
    frame_ptr: FramePointer,
    num_locals: u16,
    max_stack: u16,
    current_operand_stack_depth: u16,
    //push, pop etc
}

impl<'vm, 'k> JavaInterpreterFrame<'vm, 'k> {
    fn enter_guest(&mut self) {
        //todo should unlock the guard
        self.debug_assert();
    }

    fn exit_guest(&mut self) {
        //todo should maybe relock the guard or not
        self.debug_assert();
    }

    // within guerst java
    pub fn within_guest<T>(&mut self, within_native: impl FnOnce(&mut JavaStackGuard<'vm>, RBPAndRSP) -> Result<T, WasException<'vm>>) -> Result<T, WasException<'vm>> {
        self.enter_guest();
        let rbp_and_rsp = RBPAndRSP {
            rbp: self.frame_ptr.as_nonnull(),
            rsp: self.next_frame_pointer().as_nonnull(),
        };
        let res = within_native(self.java_stack, rbp_and_rsp);
        self.exit_guest();
        res
    }

    pub(crate) fn update_stack_depth(&mut self, current_pc: ByteCodeOffset, stack_depth: StackDepth) {
        // let jvm = self.jvm();
        // let string_pool = &jvm.string_pool;
        // let view = self.class_pointer(jvm).view();
        // let method_view = view.method_view_i(self.method_i());
        // dbg!((method_view.name().0.to_str(string_pool), current_pc));
        self.java_stack.update_stack_depth(current_pc, self.frame_ptr, stack_depth);
    }

    pub fn safepoint_check(&mut self) {
        if self.java_stack.signal_safe_data().interpreter_should_safepoint_check.load(Ordering::SeqCst) {
            self.java_stack.drop_guard();
            todo!();
            self.java_stack.reacquire();
        }
    }
}

impl<'gc, 'k> HasFrame<'gc> for JavaInterpreterFrame<'gc, 'k> {
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
        Ok(self.num_locals)
    }

    fn max_stack(&self) -> u16 {
        self.max_stack
    }

    fn next_frame_pointer(&self) -> FramePointer {
        unsafe {
            FramePointer(NonNull::new(self.frame_ptr.0.as_ptr()
                .sub(FRAME_HEADER_END_OFFSET)
                .sub((self.num_locals as usize * size_of::<StackNativeJavaValue<'gc>>()) as usize)
                .sub((self.max_stack as usize * size_of::<StackNativeJavaValue<'gc>>()) as usize)).unwrap())
        }
    }

    fn debug_assert(&self) {
        self.java_stack.debug_assert();
    }

    fn class_pointer(&self) -> Result<Arc<RuntimeClass<'gc>>, IsOpaque> {
        todo!()
    }

    fn try_current_frame_pc(&self) -> Option<ByteCodeOffset> {
        None //todo should do better
    }

    fn java_stack_ref(&self) -> &JavaStackGuard<'gc> {
        &self.java_stack
    }

    fn java_stack_mut(&mut self) -> &mut JavaStackGuard<'gc> {
        &mut self.java_stack
    }
}

impl<'gc, 'k> PushableFrame<'gc> for JavaInterpreterFrame<'gc, 'k> {
    fn push_frame<T>(&mut self, frame_to_write: StackEntryPush, within_push: impl FnOnce(&mut JavaStackGuard<'gc>) -> Result<T, WasException<'gc>>) -> Result<T, WasException<'gc>> {
        todo!()
    }

    fn push_frame_opaque<T>(&mut self, opaque_frame: OpaqueFramePush, within_push: impl for<'l> FnOnce(&mut OpaqueFrame<'gc, 'l>) -> Result<T, WasException<'gc>>) -> Result<T, WasException<'gc>> {
        todo!()
    }

    fn push_frame_java<T>(&mut self, java_frame_push: JavaFramePush, within_push: impl for<'l> FnOnce(&mut JavaInterpreterFrame<'gc, 'l>) -> Result<T, WasException<'gc>>) -> Result<T, WasException<'gc>> {
        self.java_stack.push_java_frame(self.frame_ptr, self.next_frame_pointer(), java_frame_push, |java_frame| within_push(java_frame))
    }

    fn push_frame_native<T>(&mut self, native_frame_push: NativeFramePush, within_push: impl for<'l> FnOnce(&mut NativeFrame<'gc, 'l>) -> Result<T, WasException<'gc>>) -> Result<T, WasException<'gc>> {
        // self.update_stack_depth()//todo sshould update stack depths
        self.java_stack.push_frame_native(self.frame_ptr, self.next_frame_pointer(), native_frame_push, |native_frame| within_push(native_frame))
    }
}

impl<'gc, 'k> JavaInterpreterFrame<'gc, 'k> {
    pub fn from_frame_pointer_interpreter<T>(java_stack_guard: &mut JavaStackGuard<'gc>, frame_pointer: FramePointer,
                                             within_interpreter: impl for<'k2> FnOnce(&mut JavaInterpreterFrame<'gc, 'k2>) -> Result<T, WasException<'gc>>) -> Result<T, WasException<'gc>> {
        let jvm = java_stack_guard.jvm();
        let mut res = JavaInterpreterFrame {
            java_stack: java_stack_guard,
            frame_ptr: frame_pointer,
            num_locals: 0,
            max_stack: 0,
            current_operand_stack_depth: 0,
        };
        let method_id = res.frame_ref().method_id().unwrap();
        let jvm = res.jvm();
        let (rc, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
        let view = rc.view();
        let method_view = view.method_view_i(method_i);
        let code = method_view.code_attribute().unwrap();
        res.num_locals = code.max_locals;
        res.max_stack = code.max_stack;
        within_interpreter(&mut res)
    }

    pub fn push_os(&mut self, njv: InterpreterJavaValue) {
        let current_depth = self.current_operand_stack_depth;
        self.os_set_from_start_raw(current_depth, njv.to_raw());
        self.current_operand_stack_depth += 1;
    }

    pub fn pop_all(&mut self) {
        self.current_operand_stack_depth = 0;
    }

    pub fn pop_os(&mut self, expected_type: RuntimeType) -> InterpreterJavaValue {
        if self.current_operand_stack_depth == 0 {
            panic!()
        }
        self.current_operand_stack_depth -= 1;
        let current_depth = self.current_operand_stack_depth;
        self.os_get_from_start(current_depth, expected_type).to_interpreter_jv()
    }

    pub fn local_get_interpreter(&mut self, i: u16, rtype: RuntimeType) -> InterpreterJavaValue {
        self.local_get_handle(i, rtype).to_interpreter_jv()
    }

    pub fn class_pointer(&self, jvm: &'gc JVMState<'gc>) -> Arc<RuntimeClass<'gc>> {
        let method_id = self.frame_ref().method_id().unwrap();
        let (rc, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
        rc
    }

    pub fn current_class_view(&self, jvm: &'gc JVMState<'gc>) -> Arc<dyn ClassView> {
        self.class_pointer(jvm).view()
    }

    pub fn current_method_i(&self, jvm: &'gc JVMState<'gc>) -> MethodI {
        let method_id = self.frame_ref().method_id().unwrap();
        let (rc, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
        method_i
    }

    pub fn current_loader(&self, jvm: &'gc JVMState<'gc>) -> LoaderName {
        LoaderName::BootstrapLoader //todo
    }
}
