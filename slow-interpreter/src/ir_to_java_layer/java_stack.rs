use std::ffi::c_void;
use std::mem::size_of;
use std::ptr::NonNull;
use nonnull_const::NonNullConst;
use another_jit_vm::{FramePointerOffset, IRMethodID};
use another_jit_vm::stack::CannotAllocateStack;

use another_jit_vm_ir::ir_stack::{IRFrameMut, IRFrameRef, OwnedIRStack};
use rust_jvm_common::{MethodId, NativeJavaValue};
use rust_jvm_common::opaque_id_table::OpaqueID;
use rust_jvm_common::runtime_type::RuntimeType;

use crate::{JavaValue, JVMState};
use crate::ir_to_java_layer::java_vm_state::JavaVMStateWrapper;

pub struct OwnedJavaStack<'vm> {
    java_vm_state: &'vm JavaVMStateWrapper<'vm>,
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
                *method_id as i64
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

    pub fn unwrap_opaque(&self) -> Option<OpaqueID>{
        match self {
            OpaqueFrameIdOrMethodID::Opaque { opaque_id } => {
                return Some(*opaque_id)
            },
            OpaqueFrameIdOrMethodID::Method { .. } => panic!()
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum JavaStackPosition {
    Frame {
        frame_pointer: NonNull<c_void>
    },
    Top,
}

impl JavaStackPosition {
    pub fn get_frame_pointer(&self) -> NonNullConst<c_void> {
        match self {
            JavaStackPosition::Frame { frame_pointer } => (*frame_pointer).into(),
            JavaStackPosition::Top => panic!()
        }
    }
}

impl<'vm> OwnedJavaStack<'vm> {
    pub fn new(java_vm_state: &'vm JavaVMStateWrapper<'vm>) -> Result<Self,CannotAllocateStack> {
        let inner = OwnedIRStack::new()?;
        Ok(Self {
            java_vm_state,
            inner,
        })
    }
    pub fn frame_at(&self, java_stack_position: JavaStackPosition, jvm: &'vm JVMState<'vm>) -> RuntimeJavaStackFrameRef<'_, 'vm> {
        let ir_frame = unsafe { self.inner.frame_at(java_stack_position.get_frame_pointer()) };
        // let ir_method_id = ir_frame.ir_method_id();
        // let max_locals = if let Some(method_id) = ir_frame.method_id() {
        //     let ir_method_id_2 = self.java_vm_state.inner.read().unwrap().most_up_to_date_ir_method_id_for_method_id.get(&method_id).cloned();
        //     // assert_eq!(ir_method_id_2, ir_method_id);
        //     if jvm.is_native_by_method_id(method_id) {
        //         Some(jvm.num_args_by_method_id(method_id))
        //     } else {
        //         Some(jvm.max_locals_by_method_id(method_id))
        //     }
        // } else {
        //     None
        // };
        RuntimeJavaStackFrameRef {
            ir_ref: ir_frame,
            jvm,
        }
    }
}



pub struct RuntimeJavaStackFrameRef<'l, 'vm> {
    pub(crate) ir_ref: IRFrameRef<'l>,
    pub(crate) jvm: &'vm JVMState<'vm>,
}

impl<'vm> RuntimeJavaStackFrameRef<'_, 'vm> {
    pub fn read_target(&self, offset: FramePointerOffset) -> NativeJavaValue<'vm> {
        let res = self.ir_ref.read_at_offset(offset);
        NativeJavaValue{as_u64:res}
        /*match rtype {
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
        }*/
    }

    pub fn nth_operand_stack_member(&self, n: usize, rtype: RuntimeType) -> JavaValue<'vm> {
        todo!()
        /*let offset = FramePointerOffset(self.max_locals.unwrap() as usize * size_of::<u64>() + n * size_of::<u64>());
        self.read_target(offset, rtype)*/
    }

    pub fn nth_local(&self, n: usize) -> NativeJavaValue<'vm> {
        let offset = FramePointerOffset(n * size_of::<u64>());
        self.read_target(offset)
    }
}

pub struct RuntimeJavaStackFrameMut<'l, 'vm> {
    pub ir_mut: IRFrameMut<'l>,
    pub(crate) jvm: &'vm JVMState<'vm>,
}

impl<'k, 'l, 'vm, 'ir_vm_life, 'native_vm_life> RuntimeJavaStackFrameMut<'l, 'vm> {
    pub fn downgrade_owned(self) -> RuntimeJavaStackFrameRef<'l, 'vm> {
        RuntimeJavaStackFrameRef {
            ir_ref: self.ir_mut.downgrade_owned(),
            jvm: self.jvm,
        }
    }

    pub fn downgrade<'new_l>(&'new_l self) -> RuntimeJavaStackFrameRef<'new_l, 'vm> {
        RuntimeJavaStackFrameRef {
            ir_ref: self.ir_mut.downgrade(),
            jvm: self.jvm,
        }
    }


    pub fn assert_prev_rip<'gc>(&mut self, ir_method_ref: IRMethodID, jvm: &'gc JVMState<'gc>) {
        let method_pointer = jvm.java_vm_state.ir.lookup_ir_method_id_pointer(ir_method_ref);
        self.ir_mut.assert_prev_rip(method_pointer.as_ptr());
    }
}
