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
use wtf8::Wtf8Buf;

use classfile_view::view::HasAccessFlags;
use classfile_view::view::ptype_view::PTypeView;
use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType, CPRefType};
use rust_jvm_common::compressed_classfile::code::{CInstruction, CompressedCode, CompressedInstructionInfo};
use rust_jvm_common::compressed_classfile::names::{FieldName, MethodName};
use rust_jvm_common::loading::LoaderName;

use crate::gc_memory_layout_common::{AllocatedObjectType, FramePointerOffset};
use crate::jit::ir::{IRInstr, IRLabel, Register};
use crate::jit::state::{Labeler, NaiveStackframeLayout};
use crate::jit::state::birangemap::BiRangeMap;
use crate::jit_common::java_stack::JavaStack;
use crate::jvm_state::JVMState;
use crate::method_table::MethodId;
use crate::runtime_class::RuntimeClass;
use crate::rust_jni::interface::array::release_boolean_array_elements;

pub mod ir;
pub mod state;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct LabelName(u32);

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
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
    LoadClass { class_type: CPDType, res: FramePointerOffset },
    Throw { res: FramePointerOffset },
    MonitorEnter { ref_offset: FramePointerOffset },
    MonitorExit { ref_offset: FramePointerOffset },
    TopLevelReturn {},
    Todo {},
    AllocateVariableSizeArrayANewArray { target_type_sub_type: CPDType, len_offset: FramePointerOffset, res_write_offset: FramePointerOffset },
}


#[derive(PartialEq, Eq, Copy, Clone, Debug, Hash)]
pub struct CompiledCodeID(pub u32);

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct IRInstructionIndex(u32);

pub struct NativeInstructionLocation(*mut c_void);

#[derive(Clone, Debug)]
pub struct NotSupported;


#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ByteCodeOffset(u16);

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
        let mut method_table_guard = self.jvm.method_table.write().unwrap();
        let method_id = method_table_guard.get_method_id(rc.clone(), method_view.method_i());
        Some((method_id, method_view.is_native()))
    }

    pub fn lookup_special(&self, on: CPDType, name: MethodName, desc: CMethodDescriptor) -> Option<(MethodId, bool)> {
        let classes_guard = self.jvm.classes.read().unwrap();
        let (loader_name, rc) = classes_guard.get_loader_and_runtime_class(&on)?;
        assert_eq!(loader_name, self.loader);
        let view = rc.view();
        let method_view = view.lookup_method(name, &desc).unwrap();
        let mut method_table_guard = self.jvm.method_table.write().unwrap();
        let method_id = method_table_guard.get_method_id(rc.clone(), method_view.method_i());
        Some((method_id, method_view.is_native()))
    }

    pub fn lookup_type_loaded(&self, cpdtype: &CPDType) -> Option<(Arc<RuntimeClass<'gc_life>>, LoaderName)> {
        let rc = self.jvm.classes.read().unwrap().is_loaded(cpdtype)?;
        let loader = self.jvm.classes.read().unwrap().get_initiating_loader(&rc);
        Some((rc, loader))
    }

    pub fn lookup_method_layout(&self, methodid: usize) -> NaiveStackframeLayout {
        let (rc, method_i) = self.jvm.method_table.read().unwrap().try_lookup(methodid).unwrap();
        let view = rc.view();
        let method_view = view.method_view_i(method_i);
        let function_frame_type = self.jvm.function_frame_type_data.read().unwrap();
        let frames = function_frame_type.get(&methodid).unwrap();
        let stack_depth = frames.iter()
            .sorted_by_key(|(offset, _)| *offset)
            .enumerate()
            .map(|(i, (_offset, frame))| (i as u16, frame.stack_map.len() as u16))
            .collect();
        let code = method_view.code_attribute().unwrap();
        NaiveStackframeLayout::from_stack_depth(stack_depth, code.max_locals, code.max_stack)
    }

    pub fn get_compressed_code(&self, method_id: MethodId) -> CompressedCode {
        let (rc, method_i) = self.jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
        let view = rc.view();
        let method_view = view.method_view_i(method_i);
        method_view.code_attribute().unwrap().clone()
    }
}

pub struct ToIR {
    labels: Vec<IRLabel>,
    ir: Vec<(ByteCodeOffset, IRInstr)>,
    pub function_start_label: LabelName,
}


pub struct ToNative {
    code: Vec<u8>,
    new_labels: HashMap<LabelName, *mut c_void>,
    bytecode_offset_to_address: BiRangeMap<*mut c_void, ByteCodeOffset>,
    exits: HashMap<LabelName, VMExitType>,
    function_start_label: LabelName,
}


pub enum TransitionType {
    ResolveCalls
}

pub fn transition_stack_frame(transition_type: TransitionType, frame_to_fix: &mut JavaStack) {
    match transition_type {
        TransitionType::ResolveCalls => {}
    }
}




