use std::sync::Arc;

use another_jit_vm_ir::ir_stack::{IRFrameMut, IRFrameRef};
use rust_jvm_common::{MethodI, NativeJavaValue};
use rust_jvm_common::loading::LoaderName;
use rust_jvm_common::runtime_type::RuntimeType;

use crate::{ JavaValueCommon, JVMState, NewJavaValue, NewJavaValueHandle, StackEntryPush, WasException};
use crate::better_java_stack::frame_iter::JavaFrameIterRefNew;
use crate::better_java_stack::FramePointer;
use crate::better_java_stack::interpreter_frame::JavaInterpreterFrame;
use crate::better_java_stack::java_stack_guard::JavaStackGuard;
use crate::better_java_stack::native_frame::NativeFrame;
use crate::better_java_stack::opaque_frame::OpaqueFrame;
use crate::ir_to_java_layer::java_stack::OpaqueFrameIdOrMethodID;
use crate::java_values::native_to_new_java_value_rtype;
use crate::stack_entry::{JavaFramePush, NativeFramePush, OpaqueFramePush};
use crate::threading::java_thread::JavaThread;

#[derive(Debug)]
pub struct IsOpaque {}

pub trait HasFrame<'gc> {
    fn frame_ref(&self) -> IRFrameRef;
    fn frame_mut(&mut self) -> IRFrameMut;
    fn jvm(&self) -> &'gc JVMState<'gc>;
    //todo should just only implement this on some frames so no error needed
    fn num_locals(&self) -> Result<u16, IsOpaque>;
    fn max_stack(&self) -> u16;
    fn next_frame_pointer(&self) -> FramePointer;
    fn debug_assert(&self);
    fn frame_iter(&self) -> JavaFrameIterRefNew<'gc, '_>;
    fn local_get_handle(&self, i: u16, expected_type: RuntimeType) -> NewJavaValueHandle<'gc> {
        assert!(i < self.num_locals().unwrap());
        let jvm = self.jvm();
        let ir_frame_ref = self.frame_ref();
        let data = ir_frame_ref.data(i as usize);//todo replace this with a layout lookup thing again
        let native_jv = NativeJavaValue { as_u64: data };
        native_to_new_java_value_rtype(native_jv, expected_type, jvm)
    }

    fn local_set_njv(&mut self, i: u16, njv: NewJavaValue<'gc, '_>) {
        assert!(i < self.num_locals().unwrap());
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
        let num_locals = self.num_locals().unwrap() as usize;
        let ir_frame_mut = self.frame_mut();
        ir_frame_mut.write_data(num_locals + from_start as usize, raw);
    }

    fn os_get_from_start(&mut self, from_start: u16, expected_type: RuntimeType) -> NewJavaValueHandle<'gc> {
        assert!(from_start < self.max_stack());
        let ir_frame_ref = self.frame_ref();
        let num_locals = self.num_locals().unwrap() as usize;
        let data = ir_frame_ref.data(num_locals + from_start as usize);//todo replace this with a layout lookup thing again
        let native_jv = NativeJavaValue { as_u64: data };
        native_to_new_java_value_rtype(native_jv, expected_type, self.jvm())
    }

    fn is_native_method(&self) -> bool {
        match self.frame_ref().method_id() {
            None => false,
            Some(method_id) => {
                self.jvm().is_native_by_method_id(method_id)
            }
        }
    }

    fn is_opaque_method(&self) -> bool {
        let opaque_frame_or_method_id = OpaqueFrameIdOrMethodID::from_native(self.frame_ref().raw_method_id());
        opaque_frame_or_method_id.is_opaque()
    }

    fn method_i(&self) -> MethodI {
        let method_id = self.frame_ref().method_id().unwrap();
        self.jvm().method_table.read().unwrap().try_lookup(method_id).unwrap().1
    }
}

pub trait PushableFrame<'gc>: HasFrame<'gc> {
    //todo maybe specialize these based on what is being pushed
    fn push_frame<T>(&mut self, frame_to_write: StackEntryPush, within_push: impl FnOnce(&mut JavaStackGuard<'gc>) -> Result<T, WasException<'gc>>) -> Result<T, WasException<'gc>>;
    fn push_frame_opaque<T>(&mut self, opaque_frame: OpaqueFramePush, within_push: impl for<'k> FnOnce(&mut OpaqueFrame<'gc, 'k>) -> Result<T, WasException<'gc>>) -> Result<T, WasException<'gc>>;
    fn push_frame_java<T>(&mut self, java_frame: JavaFramePush, within_push: impl for<'k> FnOnce(&mut JavaInterpreterFrame<'gc, 'k>) -> Result<T, WasException<'gc>>) -> Result<T, WasException<'gc>>;
    fn push_frame_native<T>(&mut self, java_frame: NativeFramePush, within_push: impl for<'k> FnOnce(&mut NativeFrame<'gc, 'k>) -> Result<T, WasException<'gc>>) -> Result<T, WasException<'gc>>;
    fn current_loader(&self, jvm: &'gc JVMState<'gc>) -> LoaderName {
        LoaderName::BootstrapLoader //todo
    }
}

pub trait HasJavaStack<'gc> {
    fn java_stack_ref(&self) -> &JavaStackGuard<'gc>;
    fn java_stack_mut(&mut self) -> &mut JavaStackGuard<'gc>;

    fn java_thread(&self) -> Arc<JavaThread<'gc>> {
        self.java_stack_ref().java_thread.clone()
    }

    fn drop_guard(&mut self) {
        self.java_stack_mut().drop_guard();
    }
    // fn signal_safe_data(&self) -> &'k SignalAccessibleJavaStackData {
    //     self.java_stack().signal_safe_data()
    // }
}