use std::collections::{HashMap, HashSet};
use std::ffi::c_void;
use std::mem::size_of;
use std::panic;
use std::panic::catch_unwind;
use std::process::exit;
use std::sync::Arc;

use iced_x86::{BlockEncoder, InstructionBlock};
use iced_x86::BlockEncoderOptions;
use iced_x86::code_asm::{CodeAssembler, dword_bcst, dword_ptr, qword_ptr, r15, rax, rbp, rdx, rsp};
use itertools::Itertools;
use memoffset::offset_of;
use wtf8::{Wtf8, Wtf8Buf};

use another_jit_vm_ir::compiler::{IRInstr, IRLabel, LabelName};
use another_jit_vm_ir::IRMethodID;
use another_jit_vm_ir::vm_exit_abi::VMExitTypeWithArgs;
use classfile_view::view::HasAccessFlags;
use classfile_view::view::ptype_view::PTypeView;
use rust_jvm_common::{ByteCodeOffset, FieldId, InheritanceMethodID, MethodI, MethodId};
use rust_jvm_common::classfile::InstructionInfo::jsr;
use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CompressedParsedDescriptorType, CPDType, CPRefType};
use rust_jvm_common::compressed_classfile::code::{CInstruction, CompressedCode, CompressedInstructionInfo};
use rust_jvm_common::compressed_classfile::names::{FieldName, MethodName};
use rust_jvm_common::cpdtype_table::CPDTypeID;
use rust_jvm_common::loading::LoaderName;
use rust_jvm_common::method_shape::{MethodShape, MethodShapeID};
use sketch_jvm_version_of_utf8::wtf8_pool::CompressedWtf8String;
use crate::ir_to_java_layer::compiler::YetAnotherLayoutImpl;

use crate::ir_to_java_layer::java_stack::OpaqueFrameIdOrMethodID;
use crate::java::lang::reflect::method::Method;
use crate::jit::state::{Labeler, NaiveStackframeLayout};
use crate::jit::state::birangemap::BiRangeMap;
use crate::jit_common::java_stack::JavaStack;
use crate::jvm_state::JVMState;
use crate::runtime_class::RuntimeClass;
use crate::rust_jni::interface::array::release_boolean_array_elements;

pub mod ir;
pub mod state;


/*#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum VMExitType {
    ResolveInvokeStatic { method_name: MethodName, desc: CMethodDescriptor, target_class: CPDType },
    RunNativeStatic { method_name: MethodName, desc: CMethodDescriptor, target_class: CPDType },
    ResolveInvokeSpecial { method_name: MethodName, desc: CMethodDescriptor, target_class: CPDType },
    InvokeSpecialNative { method_name: MethodName, desc: CMethodDescriptor, target_class: CPDType },
    InitClass { target_class: CPDType },
    NeedNewRegion { target_class: AllocatedObjectType },
    PutStatic { target_class: CPDType, target_type: CPDType, name: FieldName, frame_pointer_offset_of_to_put: FramePointerOffset },
    Allocate { ptypeview: CPDType, loader: LoaderName, res: FramePointerOffset, bytecode_size: u16 },
    LoadString { string: Wtf8Buf, res: FramePointerOffset },
    LoadClass { class_type: CPDType, res: FramePointerOffset, bytecode_size: u16 },
    Throw { res: FramePointerOffset },
    MonitorEnter { ref_offset: FramePointerOffset },
    MonitorExit { ref_offset: FramePointerOffset },
    Trace { values: Vec<(String, FramePointerOffset)> },
    TopLevelReturn {},
    Todo {},
    NPE {},
    AllocateVariableSizeArrayANewArray { target_type_sub_type: CPDType, len_offset: FramePointerOffset, res_write_offset: FramePointerOffset },
}*/

#[derive(PartialEq, Eq, Copy, Clone, Debug, Hash)]
pub struct CompiledCodeID(pub u32);

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct IRInstructionIndex(u32);


#[derive(Clone, Debug)]
pub struct NotSupported;

#[derive(Clone, Copy)]
pub struct MethodResolver<'gc_life> {
    pub(crate) jvm: &'gc_life JVMState<'gc_life>,
    pub(crate) loader: LoaderName,
}

impl<'gc_life> MethodResolver<'gc_life> {
    pub fn lookup_static(&self, on: CPDType, name: MethodName, desc: CMethodDescriptor) -> Option<(MethodId, bool)> {
        let classes_guard = self.jvm.classes.read().unwrap();
        let (loader_name, rc) = classes_guard.get_loader_and_runtime_class(&on)?;
        assert_eq!(loader_name, self.loader);
        let view = rc.view();
        let method_view = view.lookup_method(name, &desc).unwrap();
        assert!(method_view.is_static());
        let method_id = self.jvm.method_table.write().unwrap().get_method_id(rc.clone(), method_view.method_i());
        Some((method_id, method_view.is_native()))
    }

    pub fn lookup_virtual(&self, on: CPDType, name: MethodName, desc: CMethodDescriptor) -> Option<(MethodId, bool)> {
        let classes_guard = self.jvm.classes.read().unwrap();
        let (loader_name, rc) = classes_guard.get_loader_and_runtime_class(&on)?;
        assert_eq!(loader_name, self.loader);
        let view = rc.view();
        let method_view = view.lookup_method(name, &desc).unwrap();
        assert!(!method_view.is_static());
        let method_id = self.jvm.method_table.write().unwrap().get_method_id(rc.clone(), method_view.method_i());
        Some((method_id, method_view.is_native()))
    }

    pub fn lookup_special(&self, on: CPDType, name: MethodName, desc: CMethodDescriptor) -> Option<(MethodId, bool)> {
        let classes_guard = self.jvm.classes.read().unwrap();
        let (loader_name, rc) = classes_guard.get_loader_and_runtime_class(&on)?;
        assert_eq!(loader_name, self.loader);
        let view = rc.view();
        let method_view = view.lookup_method(name, &desc).unwrap();
        let method_id = self.jvm.method_table.write().unwrap().get_method_id(rc.clone(), method_view.method_i());
        Some((method_id, method_view.is_native()))
    }

    pub fn lookup_type_loaded(&self, cpdtype: &CPDType) -> Option<(Arc<RuntimeClass<'gc_life>>, LoaderName)> {
        let read_guard = self.jvm.classes.read().unwrap();
        let rc = read_guard.is_loaded(cpdtype)?;
        let loader = read_guard.get_initiating_loader(&rc);
        Some((rc, loader))
    }

    pub fn lookup_method_layout(&self, methodid: usize) -> YetAnotherLayoutImpl {
        let (rc, method_i) = self.jvm.method_table.read().unwrap().try_lookup(methodid).unwrap();
        let view = rc.view();
        let method_view = view.method_view_i(method_i);
        let function_frame_type = self.jvm.function_frame_type_data_no_tops.read().unwrap();
        let frames = function_frame_type.get(&methodid).unwrap();
        let code = method_view.code_attribute().unwrap();
        YetAnotherLayoutImpl::new(frames,code)
    }

    pub fn get_compressed_code(&self, method_id: MethodId) -> CompressedCode {
        let (rc, method_i) = self.jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
        let view = rc.view();
        let method_view = view.method_view_i(method_i);
        method_view.code_attribute().unwrap().clone()
    }

    pub fn num_args(&self, method_id: MethodId) -> u16 {
        let (rc, method_i) = self.jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
        let view = rc.view();
        let method_view = view.method_view_i(method_i);
        method_view.num_args() as u16
    }

    pub fn lookup_ir_method_id_and_address(&self, method_id: MethodId) -> Option<(IRMethodID, *const c_void)> {
        let ir_method_id = self.jvm.java_vm_state.try_lookup_ir_method_id(OpaqueFrameIdOrMethodID::Method { method_id: method_id as u64 })?;
        let ptr = self.jvm.java_vm_state.ir.lookup_ir_method_id_pointer(ir_method_id);
        Some((ir_method_id,ptr))
    }

    pub fn get_field_id(&self, runtime_class: Arc<RuntimeClass<'gc_life>>, field_name: FieldName) -> FieldId {
        let view = runtime_class.view();
        let field_view = view.lookup_field(field_name).unwrap();
        self.jvm.field_table.write().unwrap().get_field_id(runtime_class, field_view.field_i())
    }

    pub fn get_cpdtype_id(&self, cpdtype: &CPDType) -> CPDTypeID {
        self.jvm.cpdtype_table.write().unwrap().get_cpdtype_id(cpdtype)
    }

    pub fn get_commpressed_version_of_wtf8(&self, wtf8: &Wtf8Buf) -> CompressedWtf8String{
        self.jvm.wtf8_pool.add_entry(wtf8.clone())
    }

    // pub fn lookup_inheritance_method_id(&self, method_id: MethodId) -> InheritanceMethodID{
    //     self.jvm.inheritance_ids.read().unwrap().lookup(self.jvm,method_id)
    // }

    pub fn lookup_method_shape(&self, method_shape: MethodShape) -> MethodShapeID{
        self.jvm.method_shapes.lookup_method_shape_id(method_shape)
    }
}

pub struct ToIR {
    labels: Vec<IRLabel>,
    ir: Vec<(ByteCodeOffset, IRInstr, CInstruction)>,
    pub function_start_label: LabelName,
}

pub struct ToNative {
    code: Vec<u8>,
    new_labels: HashMap<LabelName, *mut c_void>,
    bytecode_offset_to_address: BiRangeMap<*mut c_void, (u16, ByteCodeOffset, CInstruction)>,
    exits: HashMap<LabelName, VMExitTypeWithArgs>,
    function_start_label: LabelName,
}

pub enum TransitionType {
    ResolveCalls,
}

pub fn transition_stack_frame(transition_type: TransitionType, frame_to_fix: &mut JavaStack) {
    match transition_type {
        TransitionType::ResolveCalls => {}
    }
}