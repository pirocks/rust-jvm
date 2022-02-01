use std::collections::HashSet;
use std::ffi::c_void;
use std::mem::size_of;
use std::ptr::{NonNull, null_mut};

use iced_x86::CC_be::na;

use another_jit_vm_ir::ir_stack::{IRFrameMut, IRFrameRef, OwnedIRStack};
use another_jit_vm_ir::IRMethodID;
use gc_memory_layout_common::FramePointerOffset;
use jvmti_jni_bindings::jobject;
use rust_jvm_common::loading::LoaderName;
use rust_jvm_common::MethodId;
use rust_jvm_common::opaque_id_table::OpaqueID;
use rust_jvm_common::runtime_type::RuntimeType;

use crate::{JavaValue, JVMState};
use crate::ir_to_java_layer::JavaVMStateWrapper;
use crate::java_values::GcManagedObject;

pub struct OwnedJavaStack<'vm_life> {
    jvm: &'vm_life JVMState<'vm_life>,
    java_vm_state: &'vm_life JavaVMStateWrapper<'vm_life>,
    pub(crate) inner: OwnedIRStack,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum OpaqueFrameIdOrMethodID {
    Opaque {
        opaque_id: OpaqueID,
    },
    Method {
        method_id: u64
    },
}

impl OpaqueFrameIdOrMethodID {
    pub fn try_unwrap_method_id(&self) -> Option<MethodId> {
        match self {
            OpaqueFrameIdOrMethodID::Opaque { .. } => None,
            OpaqueFrameIdOrMethodID::Method { method_id } => {
                assert_ne!(*method_id, u64::MAX);
                Some(*method_id as MethodId)
            }
        }
    }

    pub fn to_native(&self) -> i64 {
        match self {
            OpaqueFrameIdOrMethodID::Opaque { opaque_id } => {
                -((opaque_id.0 + 1) as i64)
            }
            OpaqueFrameIdOrMethodID::Method { method_id } => {
                (*method_id as i64)
            }
        }
    }

    pub fn from_native(native: i64) -> Self {
        if native < 0 {
            Self::Opaque { opaque_id: OpaqueID(((-native) as u64) - 1) }
        } else {
            Self::Method { method_id: native as u64 }
        }
    }

    pub fn is_opaque(&self) -> bool{
        match self {
            OpaqueFrameIdOrMethodID::Opaque { .. } => true,
            OpaqueFrameIdOrMethodID::Method { .. } => false
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum JavaStackPosition {
    Frame {
        frame_pointer: *const c_void
    },
    Top,
}

impl JavaStackPosition {
    pub fn get_frame_pointer(&self) -> *const c_void {
        match self {
            JavaStackPosition::Frame { frame_pointer } => *frame_pointer,
            JavaStackPosition::Top => panic!()
        }
    }
}

impl<'vm_life> OwnedJavaStack<'vm_life> {
    pub fn new(java_vm_state: &'vm_life JavaVMStateWrapper<'vm_life>, jvm: &'vm_life JVMState<'vm_life>) -> Self {
        Self {
            jvm,
            java_vm_state,
            inner: OwnedIRStack::new(),
        }
    }
    pub fn frame_at(&self, java_stack_position: JavaStackPosition, jvm: &'vm_life JVMState<'vm_life>) -> RuntimeJavaStackFrameRef<'_, 'vm_life> {
        let ir_frame = unsafe { self.inner.frame_at(java_stack_position.get_frame_pointer()) };
        let ir_method_id = ir_frame.ir_method_id();
        let max_locals = if let Some(method_id) = ir_frame.method_id() {
            let ir_method_id_2 = self.java_vm_state.inner.read().unwrap().most_up_to_date_ir_method_id_for_method_id.get(&method_id).cloned();
            // assert_eq!(ir_method_id_2, ir_method_id);
            if jvm.is_native_by_method_id(method_id) {
                Some(jvm.num_args_by_method_id(method_id))
            } else {
                Some(jvm.max_locals_by_method_id(method_id))
            }
        } else {
            None
        };
        RuntimeJavaStackFrameRef {
            ir_ref: ir_frame,
            jvm,
        }
    }
}



pub struct RuntimeJavaStackFrameRef<'l, 'vm_life> {
    pub(crate) ir_ref: IRFrameRef<'l>,
    pub(crate) jvm: &'vm_life JVMState<'vm_life>,
}

impl<'vm_life> RuntimeJavaStackFrameRef<'_, 'vm_life> {
    fn read_target(&self, offset: FramePointerOffset, rtype: RuntimeType) -> JavaValue<'vm_life> {
        let res = self.ir_ref.read_at_offset(offset);
        match rtype {
            RuntimeType::IntType => JavaValue::Int(res as i32),
            RuntimeType::FloatType => JavaValue::Float(f32::from_le_bytes((res as u32).to_le_bytes())),
            RuntimeType::DoubleType => JavaValue::Double(f64::from_le_bytes((res as f64).to_le_bytes())),
            RuntimeType::LongType => JavaValue::Long(res as i64),
            RuntimeType::Ref(ref_) => {
                let ptr = res as *mut c_void;
                JavaValue::Object(NonNull::new(ptr).map(|nonnull| GcManagedObject::from_native(nonnull, self.jvm)))
            }
            RuntimeType::TopType => {
                panic!()
            }
        }
    }

    pub fn nth_operand_stack_member(&self, n: usize, rtype: RuntimeType) -> JavaValue<'vm_life> {
        todo!()
        /*let offset = FramePointerOffset(self.max_locals.unwrap() as usize * size_of::<u64>() + n * size_of::<u64>());
        self.read_target(offset, rtype)*/
    }

    pub fn nth_local(&self, n: usize, rtype: RuntimeType) -> JavaValue<'vm_life> {
        let offset = FramePointerOffset(n * size_of::<u64>());
        self.read_target(offset, rtype)
    }
}

pub struct RuntimeJavaStackFrameMut<'l, 'vm_life> {
    pub ir_mut: IRFrameMut<'l>,
    pub(crate) jvm: &'vm_life JVMState<'vm_life>,
}

impl<'k, 'l, 'vm_life, 'ir_vm_life, 'native_vm_life> RuntimeJavaStackFrameMut<'l, 'vm_life> {
    pub fn downgrade(self) -> RuntimeJavaStackFrameRef<'l, 'vm_life> {
        RuntimeJavaStackFrameRef {
            ir_ref: self.ir_mut.downgrade_owned(),
            jvm: self.jvm,
        }
    }

    fn write_target(&mut self, offset: FramePointerOffset, jv: JavaValue<'vm_life>) {
        let to_write = match jv {
            JavaValue::Long(long) => { long as u64 }
            JavaValue::Int(int) => { int as u64 }
            JavaValue::Short(short) => { short as u64 }
            JavaValue::Byte(byte) => { byte as u64 }
            JavaValue::Boolean(boolean) => { boolean as u64 }
            JavaValue::Char(char) => { char as u64 }
            JavaValue::Float(float) => { u32::from_le_bytes(float.to_le_bytes()) as u64 }
            JavaValue::Double(double) => { u64::from_le_bytes(double.to_le_bytes()) }
            JavaValue::Object(obj) => {
                match obj {
                    None => 0u64,
                    Some(obj) => {
                        obj.raw_ptr_usize() as u64
                    }
                }
            }
            JavaValue::Top => {
                panic!()
            }
        };
        self.ir_mut.write_at_offset(offset, to_write);
    }

    pub fn set_nth_local(&mut self, n: usize, jv: JavaValue<'vm_life>) {
        let offset = FramePointerOffset(n * size_of::<u64>());
        todo!()
    }

    pub fn set_nth_stack_pointer(&mut self, n: usize, jv: JavaValue<'vm_life>) {
        todo!()
    }

    pub fn assert_prev_rip(&mut self, ir_method_ref: IRMethodID, jvm: &'gc_life JVMState<'gc_life>) {
        let method_pointer = jvm.java_vm_state.ir.lookup_ir_method_id_pointer(ir_method_ref);
        self.ir_mut.assert_prev_rip(method_pointer);
    }
}
