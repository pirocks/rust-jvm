use another_jit_vm::{IRMethodID, Register};
use another_jit_vm_ir::compiler::{IRInstr};
use gc_memory_layout_common::frame_layout::NativeStackframeMemoryLayout;
use rust_jvm_common::compressed_classfile::compressed_descriptors::CompressedMethodDescriptor;
use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPDType};
use rust_jvm_common::compressed_classfile::method_names::MethodName;
use rust_jvm_common::global_consts::ADDRESS_SIZE;
use rust_jvm_common::MethodId;

use crate::compiler::intrinsics::sun_misc_unsafe::malloc_interface::{unsafe_allocate_memory, unsafe_free_memory};
use crate::compiler_common::MethodResolver;

pub mod compare_and_swap;
pub mod get_raw;
pub mod put_raw;
pub mod malloc_interface;

pub fn address_size<'gc>(resolver: &impl MethodResolver<'gc>, layout: &NativeStackframeMemoryLayout, method_id: MethodId, ir_method_id: IRMethodID) -> Option<Vec<IRInstr>> {
    let res = Register(0);
    return Some(vec![
        IRInstr::IRStart {
            temp_register: Register(2),
            ir_method_id,
            method_id,
            frame_size: layout.full_frame_size(),
            num_locals: resolver.num_locals(method_id) as usize,
        },
        IRInstr::Const32bit { to: res, const_: ADDRESS_SIZE as u32 },
        IRInstr::Return {
            return_val: Some(res),
            temp_register_1: Register(1),
            temp_register_2: Register(2),
            temp_register_3: Register(3),
            temp_register_4: Register(4),
            frame_size: layout.full_frame_size(),
        },
    ]);
}



pub fn sun_misc_unsafe<'gc>(resolver: &impl MethodResolver<'gc>, layout: &NativeStackframeMemoryLayout, method_id: MethodId, ir_method_id: IRMethodID, desc: &CMethodDescriptor, method_name: MethodName) -> Option<Vec<IRInstr>> {
    let address_size_desc = CompressedMethodDescriptor::empty_args(CPDType::IntType);
    if method_name == MethodName::method_addressSize() && desc == &address_size_desc {
        return address_size(resolver, layout, method_id, ir_method_id);
    }

    let allocate_memory_desc = CompressedMethodDescriptor { arg_types: vec![CPDType::LongType], return_type: CPDType::LongType };
    if method_name == MethodName::method_allocateMemory() && desc == &allocate_memory_desc {
        return unsafe_allocate_memory(resolver, layout, method_id, ir_method_id);
    }

    let free_memory_desc = CompressedMethodDescriptor::void_return(vec![CPDType::LongType]);
    if method_name == MethodName::method_freeMemory() && desc == &free_memory_desc {
        return unsafe_free_memory(resolver, layout, method_id, ir_method_id);
    }

    None
}