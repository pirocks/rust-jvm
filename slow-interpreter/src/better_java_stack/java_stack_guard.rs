use std::sync::{Mutex, MutexGuard};
use another_jit_vm_ir::ir_stack::OwnedIRStack;
use another_jit_vm_ir::WasException;
use crate::better_java_stack::{FramePointer, JavaStack, StackDepth};
use crate::better_java_stack::exit_frame::JavaExitFrame;
use crate::better_java_stack::opaque_frame::OpaqueFrame;
use crate::JVMState;

pub struct JavaStackGuard<'vm> {
    stack: &'vm Mutex<JavaStack<'vm>>,
    guard: Option<MutexGuard<'vm, JavaStack<'vm>>>,
    jvm: &'vm JVMState<'vm>,
    current_frame_pointer: FramePointer
}

impl <'vm> JavaStackGuard<'vm> {
    pub(crate) fn has_been_used(&self) -> bool{
        self.guard.as_ref().unwrap().has_been_used
    }

    pub(crate) fn ir_stack(&self) -> &OwnedIRStack{
        &self.guard.as_ref().unwrap().owned_ir_stack
    }

    //todo I really need an init function which just creates the mutex and everything in one place
    pub fn new_from_empty_stack(jvm: &'vm JVMState<'vm>, stack: &'vm Mutex<JavaStack<'vm>>, with_initial_opaque_frame: impl FnOnce(&mut OpaqueFrame) -> Result<(), WasException>) -> Result<(), WasException>{
        let mut guard = stack.lock().unwrap();
        if guard.has_been_used{
            panic!()
        }
        let mut res = Self{
            stack,
            guard: Some(guard),
            jvm,
            current_frame_pointer: FramePointer(guard.owned_ir_stack.native.mmaped_top)
        };
        let mut opaque_frame = OpaqueFrame::new_from_empty_stack(&mut res);
        guard.has_been_used = true;
        with_initial_opaque_frame(&mut opaque_frame)
    }

    pub fn new_from_prev_with_new_frame_pointer(old: Self, new_frame_pointer: FramePointer) -> Self{
        let Self { stack, guard, jvm, current_frame_pointer } = old;
        Self{
            stack,
            guard,
            jvm,
            current_frame_pointer: new_frame_pointer
        }
    }

    pub fn debug_assert(&self) {
        self.assert_interpreter_frame_operand_stack_depths_sorted();
    }

    fn assert_interpreter_frame_operand_stack_depths_sorted(&self) {
        self.guard.as_ref().unwrap().assert_interpreter_frame_operand_stack_depths_sorted();
    }

    pub fn exit_frame<'k>(&'k mut self, frame_pointer: FramePointer, stack_depth: Option<StackDepth>) -> JavaExitFrame<'vm, 'k> {
        JavaExitFrame { java_stack: self, frame_pointer, num_locals: todo!(), max_stack: todo!(), stack_depth }
    }

    // pub fn push_frame<T>(&mut self, frame_to_write: StackEntryPush, within_push: impl FnOnce(&mut JavaStackGuard<'vm>) -> Result<T, WasException>) -> Result<T, WasException> {
    //     push_interpreter(self, self.current_frame_pointer, )
    // }

    fn enter_guest(&mut self) {
        todo!()
    }

    fn exit_guest(&mut self) {
        todo!()
    }

    // within guerst java
    pub fn within_guest<T>(&mut self, within_native: impl FnOnce(&mut JavaStackGuard<'vm>) -> Result<T, WasException>) {
        self.enter_guest();
        todo!();
        self.exit_guest();
    }

    fn current_frame_ptr(&self) -> FramePointer{
        self.current_frame_pointer
    }

    pub fn jvm(&self) -> &'vm JVMState<'vm> {
        self.jvm
    }

}
