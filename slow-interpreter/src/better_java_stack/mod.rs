use std::ptr::NonNull;
use std::sync::{Arc};

use libc::c_void;

use another_jit_vm_ir::ir_stack::{OwnedIRStack};
use rust_jvm_common::{ByteCodeOffset};

use crate::{AllocatedHandle, JVMState};
use crate::better_java_stack::exit_frame::JavaExitFrame;
use crate::better_java_stack::java_stack_guard::JavaStackGuard;
use crate::better_java_stack::thread_remote_read_mechanism::SignalAccessibleJavaStackData;

#[cfg(test)]
pub mod test;
pub mod thread_remote_read_mechanism;
pub mod frames;
pub mod interpreter_frame;
pub mod exit_frame;
pub mod remote_frame;
pub mod java_stack_guard;
pub mod opaque_frame;
pub mod native_frame;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Ord, PartialOrd)]
pub struct FramePointer(pub NonNull<c_void>);

impl FramePointer {
    pub fn as_ptr(&self) -> *mut c_void {
        self.0.as_ptr()
    }

    pub fn as_const_ptr(&self) -> *const c_void {
        self.0.as_ptr() as *const c_void
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct StackDepth(u16);

//needs to keep track of operand stack for interpreter
//      needs to be viewable by other threads
//needs to have same underlying for interpreter and not-interpreter
//      follows that there needs to be a mechanism for non-interpreter frames in exits to know
//      operand stack depth
//needs to be fast
// one per java thread, needs to be
// maybe built on top of ir stack
//todo needs to be interruptable and viewable once interrupted
// todo if in guest then can send stack pointer.
// need a in guest/not in guest atomic, per thread atomic.
pub struct JavaStack<'gc> {
    jvm: &'gc JVMState<'gc>,
    owned_ir_stack: OwnedIRStack,
    interpreter_frame_operand_stack_depths: Vec<(FramePointer, InterpreterFrameState)>,
    throw: Option<AllocatedHandle<'gc>>,
    //todo this should probably be in some kind of thread state thing
    thread_stack_data: Arc<SignalAccessibleJavaStackData>,
    has_been_used: bool
}

#[derive(Copy, Clone, Debug)]
pub struct InterpreterFrameState {
    stack_depth: StackDepth,
    current_pc: ByteCodeOffset,
}

impl<'gc> JavaStack<'gc> {
    pub fn new(jvm: &'gc JVMState<'gc>, owned_ir_stack: OwnedIRStack, thread_stack_data: Arc<SignalAccessibleJavaStackData>) -> Self {
        Self {
            jvm,
            owned_ir_stack,
            interpreter_frame_operand_stack_depths: vec![],
            throw: None,
            thread_stack_data,
            has_been_used: false
        }
    }

    pub fn assert_interpreter_frame_operand_stack_depths_sorted(&self) {
        assert!(self.interpreter_frame_operand_stack_depths.iter().map(|(frame_ptr, _)| *frame_ptr).is_sorted());
    }
}


//need enter and exit native functions, enter taking an operand stack depth?




// fn push_interpreter<'gc, 'k>(
//     java_stack_guard: &'k mut JavaStackGuard<'gc>,
//     current_frame_pointer: FramePointer,
//     next_frame_pointer: FramePointer,
//     stack_entry: StackEntryPush
// ) -> JavaInterpreterFrame<'gc, 'k> {
//     let jvm = java_stack_guard.jvm();
//     let top_level_exit_ptr = get_top_level_exit_ptr(jvm);
//     match stack_entry {
//         StackEntryPush::Java { operand_stack, local_vars, method_id } => {
//             todo!()
//         }
//         StackEntryPush::Native { method_id, native_local_refs, local_vars, operand_stack } => {
//
//         }
//         StackEntryPush::Opaque { opaque_id, native_local_refs } => {
//
//         }
//     }
// }

impl<'gc, 'k> JavaExitFrame<'gc, 'k> {}
