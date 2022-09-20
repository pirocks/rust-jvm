use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::NonNull;
use std::sync::Arc;

use libc::c_void;
use nonnull_const::NonNullConst;

use another_jit_vm_ir::ir_stack::OwnedIRStack;
use rust_jvm_common::ByteCodeOffset;
use thread_signal_handler::SignalAccessibleJavaStackData;

use crate::better_java_stack::exit_frame::JavaExitFrame;
use crate::better_java_stack::java_stack_guard::JavaStackGuard;
use crate::rust_jni::jni_interface::PerStackInterfaces;

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
pub mod frame_iter;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Ord, PartialOrd)]
pub struct FramePointer(pub NonNull<c_void>);

impl FramePointer {
    pub fn as_ptr(&self) -> *mut c_void {
        self.0.as_ptr()
    }

    pub fn as_const_ptr(&self) -> *const c_void {
        self.0.as_ptr() as *const c_void
    }

    pub fn as_const_nonnull(&self) -> NonNullConst<c_void> {
        self.0.into()
    }

    pub fn as_nonnull(&self) -> NonNull<c_void> {
        self.0
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct StackDepth(pub u16);

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
    phantom: PhantomData<&'gc ()>,
    owned_ir_stack: OwnedIRStack,
    //todo is a sorted vec by frame pointer need some better data structure
    interpreter_frame_operand_stack_depths: Vec<(FramePointer, InterpreterFrameState)>,
    //todo this should probably be in some kind of thread state thing
    thread_stack_data: Arc<SignalAccessibleJavaStackData>,
    has_been_used: bool,
    per_stack_interface: PerStackInterfaces,
}

#[derive(Copy, Clone, Debug)]
pub struct InterpreterFrameState {
    stack_depth: StackDepth,
    current_pc: ByteCodeOffset,
}

impl<'gc> JavaStack<'gc> {
    pub fn new(owned_ir_stack: OwnedIRStack, thread_stack_data: Arc<SignalAccessibleJavaStackData>) -> Self {
        Self {
            phantom: Default::default(),
            owned_ir_stack,
            interpreter_frame_operand_stack_depths: vec![],
            thread_stack_data,
            has_been_used: false,
            per_stack_interface: PerStackInterfaces::new()
        }
    }

    pub fn assert_interpreter_frame_operand_stack_depths_sorted(&self) {
        assert!(self.interpreter_frame_operand_stack_depths.iter().rev().map(|(frame_ptr, _)| *frame_ptr).is_sorted());
    }

    pub fn signal_safe_data(&self) -> &SignalAccessibleJavaStackData {
        self.thread_stack_data.deref()
    }
}


//need enter and exit native functions, enter taking an operand stack depth?


impl<'gc, 'k> JavaExitFrame<'gc, 'k> {}
