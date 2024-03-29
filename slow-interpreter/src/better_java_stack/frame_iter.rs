use std::ffi::c_void;
use std::ptr::NonNull;
use std::sync::Arc;

use nonnull_const::NonNullConst;

use another_jit_vm_ir::ir_stack::{IRFrameMut, IRFrameRef, IsOpaque};
use runtime_class_stuff::RuntimeClass;
use rust_jvm_common::ByteCodeOffset;
use rust_jvm_common::loading::LoaderName;

use crate::better_java_stack::FramePointer;
use crate::better_java_stack::frames::{HasFrame};
use crate::better_java_stack::java_stack_guard::JavaStackGuard;
use crate::JVMState;

pub struct FrameIterFrameRef<'gc, 'k> {
    java_stack: &'k JavaStackGuard<'gc>,
    frame_pointer: FramePointer,
    is_interpreted: bool,
    pc: Option<ByteCodeOffset>,
}

impl<'gc, 'k> HasFrame<'gc> for FrameIterFrameRef<'gc, 'k> {
    fn frame_ref(&self) -> IRFrameRef {
        IRFrameRef {
            ptr: self.frame_pointer.as_const_nonnull(),
            _ir_stack: self.java_stack.ir_stack(),
        }
    }

    fn frame_mut(&mut self) -> IRFrameMut {
        todo!()
    }

    fn jvm(&self) -> &'gc JVMState<'gc> {
        self.java_stack.jvm()
    }

    fn num_locals(&self) -> Result<u16, IsOpaque> {
        let method_id = self.frame_ref().method_id()?;
        Ok(self.jvm().num_local_var_slots(method_id))
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

    fn frame_iter(&self) -> JavaFrameIterRefNew<'gc, '_> {
        todo!()
    }

    fn class_pointer(&self) -> Result<Arc<RuntimeClass<'gc>>, IsOpaque> {
        self.try_class_pointer(self.jvm())
    }

    fn try_current_frame_pc(&self) -> Option<ByteCodeOffset> {
        todo!()
    }

    fn java_stack_ref(&self) -> &JavaStackGuard<'gc> {
        todo!()
    }

    fn java_stack_mut(&mut self) -> &mut JavaStackGuard<'gc> {
        todo!()
    }
}

impl<'vm, 'k> FrameIterFrameRef<'vm, 'k> {
    pub fn try_class_pointer(&self, jvm: &'vm JVMState<'vm>) -> Result<Arc<RuntimeClass<'vm>>, IsOpaque> {
        let method_id = self.frame_ref().method_id()?;
        let (rc, _) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
        Ok(rc)
    }

    pub fn try_pc(&self) -> Option<ByteCodeOffset> {
        self.pc
    }

    pub fn is_interpreted(&self) -> bool {
        self.is_interpreted
    }

    pub fn loader(&self) -> LoaderName{
        LoaderName::BootstrapLoader//todo loader
    }
}

pub struct PreviousFramePointerIter<'vm, 'k> {
    java_stack_guard: &'k JavaStackGuard<'vm>,
    current_frame_pointer: Option<FramePointer>,
}

impl<'vm, 'k> Iterator for PreviousFramePointerIter<'vm, 'k> {
    type Item = IRFrameRef<'k>;

    fn next(&mut self) -> Option<Self::Item> {
        let ir_stack = self.java_stack_guard.ir_stack();
        if self.current_frame_pointer? == FramePointer(ir_stack.native.mmaped_top) {
            self.current_frame_pointer = None;
            None
        } else {
            let res = unsafe { ir_stack.frame_at(self.current_frame_pointer?.as_const_nonnull()) };
            self.current_frame_pointer = Some(FramePointer(res.prev_rbp().unwrap()));
            Some(res)
        }
    }
}

pub struct JavaFrameIterRefNew<'vm, 'k> {
    helper: PreviousFramePointerIter<'vm, 'k>,
    java_stack_guard: &'k JavaStackGuard<'vm>,
    current_rip: Option<NonNullConst<c_void>>,
    current_pc: Option<ByteCodeOffset>,
}

impl<'vm, 'k> JavaFrameIterRefNew<'vm, 'k> {
    pub fn new(java_stack_guard: &'k JavaStackGuard<'vm>, current_frame_pointer: FramePointer, current_pc: Option<ByteCodeOffset>) -> Self {
        Self {
            helper: PreviousFramePointerIter { java_stack_guard, current_frame_pointer: Some(current_frame_pointer) },
            java_stack_guard,
            current_rip: None,
            current_pc,
        }
    }
}

impl<'vm, 'k> Iterator for JavaFrameIterRefNew<'vm, 'k> {
    type Item = FrameIterFrameRef<'vm, 'k>;

    fn next(&mut self) -> Option<Self::Item> {
        self.helper.next().map(|ir_frame_ref| {
            let prev_rip = NonNullConst::new(ir_frame_ref.prev_rip()).unwrap();
            let current_frame_pointer = FramePointer(NonNull::new(ir_frame_ref.ptr.as_ptr() as *mut c_void).unwrap());
            let mut is_interpreted = false;
            if self.current_pc.is_none() {
                self.current_pc = self.java_stack_guard.lookup_interpreter_pc_offset_with_frame_pointer(current_frame_pointer);
                is_interpreted = true;
            }
            let res = FrameIterFrameRef {
                java_stack: self.java_stack_guard,
                frame_pointer: current_frame_pointer,
                is_interpreted,
                pc: self.current_pc,
            };
            let jvm= self.java_stack_guard.jvm();
            // if let Ok(rc) = res.class_pointer() {
            //     let view = rc.view();
            //     let method_view = view.method_view_i(res.method_i());
            //     // dbg!((method_view.name().0.to_str(&jvm.string_pool), self.current_pc));
            // }
            self.current_rip = Some(prev_rip);
            match self.java_stack_guard.jvm().java_vm_state.lookup_ip(prev_rip.as_ptr()) {
                Some((_, new_pc)) => {
                    self.current_pc = Some(new_pc);
                }
                None => {
                    self.current_pc = None
                }
            };
            res
        })
    }
}
