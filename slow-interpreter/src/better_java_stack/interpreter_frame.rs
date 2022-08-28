use std::mem::size_of;
use std::ptr::NonNull;
use std::sync::Mutex;
use another_jit_vm_ir::ir_stack::{IRFrameMut, IRFrameRef};
use another_jit_vm_ir::WasException;
use gc_memory_layout_common::layout::FRAME_HEADER_END_OFFSET;
use jvmti_jni_bindings::JavaPrimitiveType;
use rust_jvm_common::NativeJavaValue;
use rust_jvm_common::runtime_type::RuntimeType;
use crate::better_java_stack::{FramePointer, JavaStack, JavaStackGuard};
use crate::better_java_stack::frames::HasFrame;
use crate::interpreter::real_interpreter_state::InterpreterJavaValue;
use crate::JVMState;

pub struct JavaInterpreterFrame<'gc, 'k> {
    java_stack: &'k mut JavaStackGuard<'gc>,
    frame_ptr: FramePointer,
    num_locals: u16,
    max_stack: u16,
    current_operand_stack_depth: u16,
    //push, pop etc
}

impl<'gc, 'k> HasFrame<'gc> for JavaInterpreterFrame<'gc, 'k> {
    fn frame_ref(&self) -> IRFrameRef {
        IRFrameRef {
            ptr: self.frame_ptr.0.into(),
            _ir_stack: todo!()/*&self.java_stack.owned_ir_stack*/,
        }
    }

    fn frame_mut(&mut self) -> IRFrameMut {
        IRFrameMut {
            ptr: self.frame_ptr.0,
            ir_stack: todo!()/*&mut self.java_stack.owned_ir_stack*/,
        }
    }

    fn jvm(&self) -> &'gc JVMState<'gc> {
        self.java_stack.jvm()
    }

    fn num_locals(&self) -> u16 {
        self.num_locals
    }

    fn max_stack(&self) -> u16 {
        self.max_stack
    }

    fn next_frame_pointer(&self) -> FramePointer {
        unsafe {
            FramePointer(NonNull::new(self.frame_ptr.0.as_ptr()
                .sub(FRAME_HEADER_END_OFFSET)
                .sub((self.num_locals as usize * size_of::<NativeJavaValue<'gc>>()) as usize)
                .sub((self.max_stack as usize * size_of::<NativeJavaValue<'gc>>()) as usize)).unwrap())
        }
    }

    fn debug_assert(&self) {
        self.java_stack.debug_assert();
    }
}

impl<'gc, 'k> JavaInterpreterFrame<'gc, 'k> {

    pub fn from_frame_pointer_interpreter<T: JavaPrimitiveType>(jvm: &'gc JVMState<'gc>, java_stack: &'gc Mutex<JavaStack<'gc>>, frame_pointer: FramePointer,
                                                                within_interpreter: impl for<'k2> FnOnce(&mut JavaInterpreterFrame<'gc,'k2>) -> Result<T, WasException>) -> Result<T, WasException> {
        let mut java_stack_guard :JavaStackGuard = todo!();
        let mut res = JavaInterpreterFrame {
            java_stack: &mut java_stack_guard,
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

    pub fn pop_os(&mut self, expected_type: RuntimeType) -> InterpreterJavaValue {
        if self.current_operand_stack_depth == 0 {
            panic!()
        }
        self.current_operand_stack_depth -= 1;
        let current_depth = self.current_operand_stack_depth;
        self.os_get_from_start(current_depth, expected_type).to_interpreter_jv()
    }
}


