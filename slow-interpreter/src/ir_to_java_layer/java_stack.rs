use std::collections::HashSet;
use std::ffi::c_void;
use std::mem::size_of;
use std::ptr::{NonNull, null_mut};

use jvmti_jni_bindings::jobject;
use rust_jvm_common::loading::LoaderName;
use rust_jvm_common::runtime_type::RuntimeType;

use crate::{JavaValue, JVMState};
use crate::gc_memory_layout_common::FramePointerOffset;
use crate::ir_to_java_layer::JavaVMStateWrapper;
use crate::java_values::GcManagedObject;
use crate::method_table::MethodId;
use crate::native_to_ir_layer::{IRFrameMut, IRFrameRef, IRMethodID, OwnedIRStack};

pub struct OwnedJavaStack<'vm_life> {
    jvm: &'vm_life JVMState<'vm_life>,
    java_vm_state: &'vm_life JavaVMStateWrapper<'vm_life>,
    pub(crate) inner: OwnedIRStack,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum OpaqueFrameIdOrMethodID {
    Opaque {
        opaque_id: u64,
    },
    Method {
        method_id: u64
    },
}

impl OpaqueFrameIdOrMethodID {
    pub fn try_unwrap_method_id(&self) -> Option<MethodId> {
        match self {
            OpaqueFrameIdOrMethodID::Opaque { .. } => None,
            OpaqueFrameIdOrMethodID::Method { method_id } => Some(*method_id as MethodId)
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
            let ir_method_id_2 = self.java_vm_state.inner.read().unwrap().method_id_to_ir_method_id.get_by_left(&method_id).cloned();
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
            frame_ptr: java_stack_position.get_frame_pointer(),
            ir_ref: ir_frame,
            jvm,
            max_locals,
        }
    }

    pub fn mut_frame_at(&'_ mut self, java_stack_position: JavaStackPosition, jvm: &'vm_life JVMState<'vm_life>) -> RuntimeJavaStackFrameMut<'_, 'vm_life> {
        let ir_frame = unsafe { self.inner.frame_at(java_stack_position.get_frame_pointer()) };
        let ir_method_id = ir_frame.ir_method_id();
        let max_locals = if let Some(method_id) = ir_frame.method_id() {
            let ir_method_id_2 = self.java_vm_state.inner.read().unwrap().method_id_to_ir_method_id.get_by_left(&method_id).cloned();
            // assert_eq!(ir_method_id_2, ir_method_id);
            jvm.max_locals_by_method_id(method_id)
        } else {
            todo!("should have seperate thing for opaque frames")
        };
        let ir_frame_mut = unsafe { self.inner.frame_at_mut(java_stack_position.get_frame_pointer() as *mut c_void) };
        RuntimeJavaStackFrameMut {
            frame_ptr: java_stack_position.get_frame_pointer(),
            ir_mut: ir_frame_mut,
            jvm,
            max_locals,
        }
    }

    pub fn write_frame(&mut self, at_position: JavaStackPosition, method_id: OpaqueFrameIdOrMethodID, locals: Vec<JavaValue<'vm_life>>, operand_stack: Vec<JavaValue<'vm_life>>, prev_rip: *const c_void, prev_rbp: *mut c_void) {
        //todo need to write magic etc

        match self.java_vm_state.try_lookup_ir_method_id(method_id) {
            None => {
                let to_write_to_data = box NativeFrameInfo {
                    method_id: method_id.try_unwrap_method_id().unwrap(),
                    loader: LoaderName::BootstrapLoader,//todo fix this
                    native_local_refs: vec![HashSet::new()],
                    local_vars: locals,
                    operand_stack,
                };
                let data: [u64; 1] = [Box::into_raw(to_write_to_data) as usize as u64];
                unsafe { self.inner.write_frame(at_position.get_frame_pointer() as *mut c_void, prev_rip, prev_rbp as *mut c_void, None, Some(method_id.try_unwrap_method_id().unwrap()), data.as_slice()); }
            }
            Some(ir_method_id) => {
                let mut data = vec![];

                for local in locals {
                    data.push(unsafe { local.to_native().as_u64 });
                }

                for stack_elem in operand_stack {
                    data.push(unsafe { stack_elem.to_native().as_u64 });
                }

                unsafe { self.inner.write_frame(at_position.get_frame_pointer() as *mut c_void, prev_rip, prev_rbp as *mut c_void, Some(ir_method_id), method_id.try_unwrap_method_id(), data.as_slice()) }
            }
        }
    }

    pub fn push_frame(&mut self,
                      java_stack_position: JavaStackPosition,
                      method_id: OpaqueFrameIdOrMethodID,
                      locals: Vec<JavaValue<'vm_life>>,
                      operand_stack: Vec<JavaValue<'vm_life>>,
    ) -> JavaStackPosition {
        let postion_to_write = match java_stack_position {
            JavaStackPosition::Frame { frame_pointer } => {
                let current_frame = self.frame_at(java_stack_position, self.jvm);
                let frame_size = current_frame.ir_ref.frame_size(&self.java_vm_state.ir);
                let new_frame_pointer = unsafe { frame_pointer.offset(-(frame_size as isize)) };
                JavaStackPosition::Frame { frame_pointer: new_frame_pointer }
            }
            JavaStackPosition::Top => JavaStackPosition::Frame { frame_pointer: self.inner.mmaped_top }
        };
        let ir_method_ref = self.jvm.java_vm_state.ir.get_top_level_return_ir_method_id();
        let method_pointer = self.jvm.java_vm_state.ir.lookup_ir_method_id_pointer(ir_method_ref);
        self.write_frame(postion_to_write, method_id, locals, operand_stack, method_pointer, match java_stack_position{
            JavaStackPosition::Frame { frame_pointer } => {
                frame_pointer
            }
            JavaStackPosition::Top => self.inner.mmaped_top
        } as *mut c_void);
        postion_to_write
    }
}

#[derive(Debug, Clone)]
pub struct NativeFrameInfo<'gc_life> {
    pub method_id: usize,
    pub loader: LoaderName,
    pub native_local_refs: Vec<HashSet<jobject>>,
    pub local_vars: Vec<JavaValue<'gc_life>>,
    pub operand_stack: Vec<JavaValue<'gc_life>>,
}

pub struct RuntimeJavaStackFrameRef<'l, 'vm_life> {
    frame_ptr: *const c_void,
    pub(crate) ir_ref: IRFrameRef<'l>,
    pub(crate) jvm: &'vm_life JVMState<'vm_life>,
    max_locals: Option<u16>,//todo opaque frame ref
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
        let offset = FramePointerOffset(self.max_locals.unwrap() as usize * size_of::<u64>() + n * size_of::<u64>());
        self.read_target(offset, rtype)
    }

    pub fn nth_local(&self, n: usize, rtype: RuntimeType) -> JavaValue<'vm_life> {
        let offset = FramePointerOffset(n * size_of::<u64>());
        self.read_target(offset, rtype)
    }

    pub fn position(&self) -> JavaStackPosition {
        JavaStackPosition::Frame { frame_pointer: self.frame_ptr }
    }
}

pub struct RuntimeJavaStackFrameMut<'l, 'vm_life> {
    frame_ptr: *const c_void,
    pub ir_mut: IRFrameMut<'l>,
    jvm: &'vm_life JVMState<'vm_life>,
    max_locals: u16,
}

impl<'k, 'l, 'vm_life, 'ir_vm_life, 'native_vm_life> RuntimeJavaStackFrameMut<'l, 'vm_life> {
    pub fn downgrade(self) -> RuntimeJavaStackFrameRef<'l, 'vm_life> {
        RuntimeJavaStackFrameRef {
            frame_ptr: self.frame_ptr,
            ir_ref: self.ir_mut.downgrade(),
            jvm: self.jvm,
            max_locals: self.max_locals.into(),
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

    pub fn set_prev_rip(&mut self, ir_method_ref: IRMethodID, jvm: &'gc_life JVMState<'gc_life>) {
        let java_stack_position = JavaStackPosition::Frame { frame_pointer: self.frame_ptr };
        let method_pointer = jvm.java_vm_state.ir.lookup_ir_method_id_pointer(ir_method_ref);
        self.ir_mut.set_prev_rip(method_pointer);
    }
}
