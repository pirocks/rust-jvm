use another_jit_vm_ir::ir_stack::{IRFrameIterRef, IRFrameMut, IRFrameRef};
use another_jit_vm_ir::WasException;
use rust_jvm_common::loading::LoaderName;
use rust_jvm_common::{ByteCodeOffset, NativeJavaValue};
use rust_jvm_common::runtime_type::RuntimeType;

use crate::{JavaValueCommon, JVMState, NewJavaValue, NewJavaValueHandle, StackEntryPush};
use crate::better_java_stack::frame_iter::FrameIterFrameRef;
use crate::better_java_stack::FramePointer;
use crate::better_java_stack::interpreter_frame::JavaInterpreterFrame;
use crate::better_java_stack::java_stack_guard::JavaStackGuard;
use crate::better_java_stack::native_frame::NativeFrame;
use crate::better_java_stack::opaque_frame::OpaqueFrame;
use crate::java_values::native_to_new_java_value_rtype;
use crate::stack_entry::{JavaFramePush, NativeFramePush, OpaqueFramePush};

pub struct JavaFrameIterRefNew<'vm, 'l, 'h> {
    ir: IRFrameIterRef<'l, 'h, 'vm>,
    jvm: &'vm JVMState<'vm>,
    current_pc: Option<ByteCodeOffset>,
}

impl<'l, 'h, 'vm> Iterator for JavaFrameIterRefNew<'l, 'h, 'vm> {
    type Item = FrameIterFrameRef<'vm, 'l>;

    fn next(&mut self) -> Option<Self::Item> {
        self.ir.next().map(|ir_frame_ref| {
            let prev_rip = ir_frame_ref.prev_rip();
            let res = StackEntryRef {
                frame_view: RuntimeJavaStackFrameRef {
                    ir_ref: ir_frame_ref,
                    jvm: self.jvm,
                },
                pc: self.current_pc,
            };
            match self.jvm.java_vm_state.lookup_ip(prev_rip) {
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




pub trait HasFrame<'gc> {
    fn frame_ref(&self) -> IRFrameRef;
    fn frame_mut(&mut self) -> IRFrameMut;
    fn jvm(&self) -> &'gc JVMState<'gc>;
    fn num_locals(&self) -> u16;
    fn max_stack(&self) -> u16;
    fn next_frame_pointer(&self) -> FramePointer;
    fn debug_assert(&self);
    fn frame_iter(&self) -> JavaFrameIterRefNew;
    fn local_get_handle(&self, i: u16, expected_type: RuntimeType) -> NewJavaValueHandle<'gc> {
        assert!(i < self.num_locals());
        let jvm = self.jvm();
        let ir_frame_ref = self.frame_ref();
        let data = ir_frame_ref.data(i as usize);//todo replace this with a layout lookup thing again
        let native_jv = NativeJavaValue { as_u64: data };
        native_to_new_java_value_rtype(native_jv, expected_type, jvm)
    }

    fn local_set_njv(&mut self, i: u16, njv: NewJavaValue<'gc, '_>) {
        assert!(i < self.num_locals());
        let native_jv = njv.to_native();
        let ir_frame_mut = self.frame_mut();
        ir_frame_mut.write_data(i as usize, unsafe { native_jv.as_u64 });
    }

    fn os_set_from_start(&mut self, from_start: u16, njv: NewJavaValue<'gc, '_>) {
        let native_jv = njv.to_native();
        self.os_set_from_start_raw(from_start, unsafe { native_jv.as_u64 })
    }

    fn os_set_from_start_raw(&mut self, from_start: u16, raw: u64) {
        assert!(from_start < self.max_stack());
        let num_locals = self.num_locals() as usize;
        let ir_frame_mut = self.frame_mut();
        ir_frame_mut.write_data(num_locals + from_start as usize, raw);
    }

    fn os_get_from_start(&mut self, from_start: u16, expected_type: RuntimeType) -> NewJavaValueHandle<'gc> {
        assert!(from_start < self.max_stack());
        let ir_frame_ref = self.frame_ref();
        let num_locals = self.num_locals() as usize;
        let data = ir_frame_ref.data(num_locals + from_start as usize);//todo replace this with a layout lookup thing again
        let native_jv = NativeJavaValue { as_u64: data };
        native_to_new_java_value_rtype(native_jv, expected_type, self.jvm())
    }
}

pub trait PushableFrame<'gc>: HasFrame<'gc> {
    //todo maybe specialize these based on what is being pushed
    fn push_frame<T>(&mut self, frame_to_write: StackEntryPush, within_push: impl FnOnce(&mut JavaStackGuard<'gc>) -> Result<T, WasException>) -> Result<T, WasException>;
    fn push_frame_opaque<T>(&mut self, opaque_frame: OpaqueFramePush, within_push: impl for<'k> FnOnce(&mut OpaqueFrame<'gc, 'k>) -> Result<T, WasException>) -> Result<T, WasException>;
    fn push_frame_java<T>(&mut self, java_frame: JavaFramePush, within_push: impl for<'k> FnOnce(&mut JavaInterpreterFrame<'gc, 'k>) -> Result<T, WasException>) -> Result<T, WasException>;
    fn push_frame_native<T>(&mut self, java_frame: NativeFramePush, within_push: impl for<'k> FnOnce(&mut NativeFrame<'gc, 'k>) -> Result<T, WasException>) -> Result<T, WasException>;
    fn current_loader(&self, jvm: &'gc JVMState<'gc>) -> LoaderName {
        LoaderName::BootstrapLoader //todo
    }
}
