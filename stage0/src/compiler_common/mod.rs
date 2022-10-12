use std::collections::HashMap;
use std::ffi::c_void;
use std::mem::size_of;
use std::sync::Arc;
use std::sync::atomic::AtomicPtr;

use itertools::Itertools;
use wtf8::Wtf8Buf;

use another_jit_vm::{FramePointerOffset, IRMethodID};
use classfile_view::view::HasAccessFlags;
use classfile_view::view::method_view::MethodView;
use gc_memory_layout_common::layout::{FRAME_HEADER_END_OFFSET, NativeStackframeMemoryLayout};
use gc_memory_layout_common::memory_regions::{AllocatedTypeID, RegionHeader};
use inheritance_tree::ClassID;
use method_table::interface_table::InterfaceID;
use method_table::MethodTable;
use runtime_class_stuff::method_numbers::MethodNumber;
use runtime_class_stuff::RuntimeClass;
use rust_jvm_common::{ByteCodeIndex, ByteCodeOffset, FieldId, MethodId};
use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CompressedClassfileStringPool, CPDType};
use rust_jvm_common::compressed_classfile::code::{CompressedCode, CompressedInstruction};
use rust_jvm_common::compressed_classfile::field_names::FieldName;
use rust_jvm_common::compressed_classfile::method_names::MethodName;
use rust_jvm_common::cpdtype_table::CPDTypeID;
use rust_jvm_common::loading::LoaderName;
use rust_jvm_common::method_shape::{MethodShape, MethodShapeID};
use sketch_jvm_version_of_utf8::wtf8_pool::CompressedWtf8String;

use crate::compiler_common::frame_data::SunkVerifierFrames;

pub mod frame_data;


// all metadata needed to compile to ir, excluding resolver stuff
pub struct JavaCompilerMethodAndFrameData {
    pub should_trace_instructions: bool,
    pub(crate) layout: YetAnotherLayoutImpl,
    pub index_by_bytecode_offset: HashMap<ByteCodeOffset, ByteCodeIndex>,
    pub current_method_id: MethodId,
    pub local_vars: usize,
    pub should_synchronize: bool,
    pub is_static: bool,
}

impl JavaCompilerMethodAndFrameData {
    pub fn new<'vm>(should_trace_instructions: bool, method_table: &MethodTable<'vm>, frames_no_tops: &HashMap<ByteCodeOffset, SunkVerifierFrames>, method_id: MethodId) -> Self {
        let (rc, method_i) = method_table.try_lookup(method_id).unwrap();
        let view = rc.view();
        let method_view = view.method_view_i(method_i);
        let code = method_view.code_attribute().unwrap();
        Self {
            should_trace_instructions,
            layout: YetAnotherLayoutImpl::new(frames_no_tops, code),
            index_by_bytecode_offset: code.instructions.iter().sorted_by_key(|(byte_code_offset, _)| *byte_code_offset).enumerate().map(|(index, (bytecode_offset, _))| (*bytecode_offset, ByteCodeIndex(index as u16))).collect(),
            current_method_id: method_id,
            local_vars: code.max_locals as usize,
            should_synchronize: method_view.is_synchronized(),
            is_static: method_view.is_static(),
        }
    }

    pub fn operand_stack_entry(&self, index: ByteCodeIndex, from_end: u16) -> FramePointerOffset {
        self.layout.operand_stack_entry(index, from_end)
    }

    pub fn is_category_2(&self, index: ByteCodeIndex, from_end: u16) -> bool {
        self.layout.is_category_2(index, from_end)
    }

    pub fn local_var_entry(&self, index: ByteCodeIndex, local_var_index: u16) -> FramePointerOffset {
        self.layout.local_var_entry(index, local_var_index)
    }

    pub fn full_frame_size(&self) -> usize {
        self.layout.full_frame_size()
    }

    pub fn num_local_vars(&self) -> usize {
        self.local_vars
    }
}


pub struct YetAnotherLayoutImpl {
    pub(crate) max_locals: u16,
    max_stack: u16,
    stack_depth_by_index: Vec<u16>,
    is_type_2_computational_type: Vec<Vec<bool>>,
    pub(crate) code_by_index: Vec<CompressedInstruction>,
}

impl YetAnotherLayoutImpl {
    pub fn new(frames_no_top: &HashMap<ByteCodeOffset, SunkVerifierFrames>, code: &CompressedCode) -> Self {
        for (offset, _) in code.instructions.iter() {
            assert!(frames_no_top.contains_key(&offset));
        }
        let stack_depth = frames_no_top.iter().sorted_by_key(|(offset, _)| *offset).map(|(_offset, frame)| {
            // assert!(frame.stack_map.iter().all(|types| !matches!(types, VType::TopType)));
            frame.stack_depth_no_tops()/*stack_map.len()*/ as u16
        }).collect();
        let computational_type = frames_no_top.iter().sorted_by_key(|(offset, _)| *offset).map(|(_offset, frame)| {
            /*assert!(frame.stack_map.iter().all(|types| !matches!(types, VType::TopType)));
            frame.stack_map.iter().map(|vtype| Self::is_type_2_computational_type(vtype)).collect()*/
            frame.is_category_2_no_tops()
        }).collect();
        Self {
            max_locals: code.max_locals,
            max_stack: code.max_stack,
            stack_depth_by_index: stack_depth,
            is_type_2_computational_type: computational_type,
            code_by_index: code.instructions.iter().sorted_by_key(|(byte_code_offset, _)| *byte_code_offset).map(|(_, instr)| instr.clone()).collect(),
        }
    }

    pub fn operand_stack_start(&self) -> FramePointerOffset {
        FramePointerOffset(FRAME_HEADER_END_OFFSET + (self.max_locals) as usize * size_of::<u64>())
    }

    pub fn operand_stack_entry(&self, index: ByteCodeIndex, from_end: u16) -> FramePointerOffset {
        if index.0 as usize >= self.stack_depth_by_index.len() {
            dbg!(&self.code_by_index[index.0 as usize]);
        }
        FramePointerOffset(FRAME_HEADER_END_OFFSET + (self.max_locals + self.stack_depth_by_index[index.0 as usize] - from_end - 1) as usize * size_of::<u64>())//-1 b/c stack depth is a len
    }

    pub fn is_category_2(&self, index: ByteCodeIndex, from_end: u16) -> bool {
        let category_2_array = &self.is_type_2_computational_type[index.0 as usize];
        *category_2_array.iter().nth(from_end as usize).unwrap()
    }

    pub fn local_var_entry(&self, _index: ByteCodeIndex, local_var_index: u16) -> FramePointerOffset {
        assert!(local_var_index <= self.max_locals);
        FramePointerOffset(FRAME_HEADER_END_OFFSET + local_var_index as usize * size_of::<u64>())
    }

    pub fn full_frame_size(&self) -> usize {
        let max_locals = self.max_locals;
        let max_stack = self.max_stack;
        full_frame_size_impl(max_locals, max_stack)
    }
}

fn full_frame_size_impl(max_locals: u16, max_stack: u16) -> usize {
    FRAME_HEADER_END_OFFSET + (max_locals + max_stack) as usize * size_of::<u64>()
}


pub struct PartialYetAnotherLayoutImpl {
    max_locals: u16,
    max_stack: u16,
}

impl PartialYetAnotherLayoutImpl {
    pub fn new(code: &CompressedCode) -> Self {
        Self {
            max_locals: code.max_locals,
            max_stack: code.max_stack,
        }
    }

    pub fn full_frame_size(&self) -> usize {
        full_frame_size_impl(self.max_locals, self.max_stack)
    }
}


pub trait MethodResolver<'gc> {
    fn lookup_static(&self, on: CPDType, name: MethodName, desc: CMethodDescriptor) -> Option<(MethodId, bool)>;
    fn lookup_virtual(&self, on: CPDType, name: MethodName, desc: CMethodDescriptor) -> MethodShapeID;
    fn lookup_native_virtual(&self, on: CPDType, name: MethodName, desc: CMethodDescriptor) -> Option<MethodId>;
    //todo unnify both of thes
    fn lookup_interface_id(&self, interface: CPDType) -> Option<InterfaceID>;
    fn lookup_interface_class_id(&self, interface: CPDType) -> ClassID;
    fn lookup_interface_method_number(&self, interface: CPDType, method_shape: MethodShape) -> Option<MethodNumber>;
    fn lookup_special(&self, on: &CPDType, name: MethodName, desc: CMethodDescriptor) -> Option<(MethodId, bool)>;
    fn lookup_type_inited_initing(&self, cpdtype: &CPDType) -> Option<(Arc<RuntimeClass<'gc>>, LoaderName)>;
    fn allocated_object_type_id(&self, rc: Arc<RuntimeClass<'gc>>, loader: LoaderName, arr_len: Option<usize>) -> AllocatedTypeID;
    fn allocated_object_region_header_pointer(&self, id: AllocatedTypeID) -> *const AtomicPtr<RegionHeader>;
    fn lookup_method_layout(&self, method_id: usize) -> YetAnotherLayoutImpl;
    fn lookup_native_method_layout(&self, method_id: usize) -> NativeStackframeMemoryLayout;
    fn lookup_partial_method_layout(&self, method_id: usize) -> PartialYetAnotherLayoutImpl;
    fn using_method_view_impl<T>(&self, method_id: MethodId, using: impl FnOnce(&MethodView) -> T) -> T;
    fn is_synchronized(&self, method_id: MethodId) -> bool;
    fn is_static(&self, method_id: MethodId) -> bool;
    fn is_native(&self, method_id: MethodId) -> bool;
    fn get_compressed_code(&self, method_id: MethodId) -> CompressedCode;
    fn num_args(&self, method_id: MethodId) -> u16;
    fn num_locals(&self, method_id: MethodId) -> u16;
    fn lookup_method_desc(&self, method_id: MethodId) -> CMethodDescriptor;
    fn lookup_ir_method_id_and_address(&self, method_id: MethodId) -> Option<(IRMethodID, *const c_void)>;
    fn get_field_id(&self, runtime_class: Arc<RuntimeClass<'gc>>, field_name: FieldName) -> FieldId;
    fn get_cpdtype_id(&self, cpdtype: CPDType) -> CPDTypeID;
    fn get_commpressed_version_of_wtf8(&self, wtf8: &Wtf8Buf) -> CompressedWtf8String;
    fn lookup_method_shape(&self, method_shape: MethodShape) -> MethodShapeID;
    fn lookup_method_number(&self, rc: Arc<RuntimeClass<'gc>>, method_shape: MethodShape) -> MethodNumber;
    fn debug_checkcast_assertions(&self) -> bool;
    // fn invocation_compilation_threshold(&self) -> u64;
    // fn invocation_count(&self, method_id: MethodId) -> u64;
    fn compile_interpreted(&self, method_id: MethodId) -> bool;
    fn string_pool(&self) -> &CompressedClassfileStringPool;
}