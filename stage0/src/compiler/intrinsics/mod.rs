use another_jit_vm::IRMethodID;
use another_jit_vm_ir::compiler::IRInstr;
use classfile_view::view::ClassView;
use gc_memory_layout_common::layout::NativeStackframeMemoryLayout;
use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CompressedMethodDescriptor, CompressedParsedDescriptorType, CPDType};
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::method_names::MethodName;
use rust_jvm_common::MethodId;

use crate::compiler::CompilerLabeler;
use crate::compiler::intrinsics::array_copy::intrinsic_array_copy;
use crate::compiler::intrinsics::get_class::intrinsic_get_class;
use crate::compiler::intrinsics::get_component_type::get_component_type_intrinsic;
use crate::compiler::intrinsics::hashcode::intrinsic_hashcode;
use crate::compiler::intrinsics::reflect_new_array::reflect_new_array;
use crate::compiler::intrinsics::sun_misc_unsafe::{address_size, get_int_volatile};
use crate::compiler::intrinsics::sun_misc_unsafe::compare_and_swap::{intrinsic_compare_and_swap_int, intrinsic_compare_and_swap_long, intrinsic_compare_and_swap_object};
use crate::compiler::intrinsics::sun_misc_unsafe::get_raw::{unsafe_get_byte_raw, unsafe_get_long_raw};
use crate::compiler::intrinsics::sun_misc_unsafe::malloc_interface::{unsafe_allocate_memory, unsafe_free_memory};
use crate::compiler::intrinsics::sun_misc_unsafe::put_raw::unsafe_put_long;
use crate::compiler::intrinsics::system_identity_hashcode::system_identity_hashcode;
use crate::compiler_common::MethodResolver;

pub mod sun_misc_unsafe;
pub mod reflect_new_array;
pub mod get_component_type;
pub mod system_identity_hashcode;
pub mod array_copy;
pub mod get_class;
pub mod hashcode;

pub fn gen_intrinsic_ir<'vm>(
    resolver: &impl MethodResolver<'vm>,
    layout: &NativeStackframeMemoryLayout,
    method_id: MethodId,
    ir_method_id: IRMethodID,
    labeler: &mut CompilerLabeler,
) -> Option<Vec<IRInstr>> {
    let (desc, method_name, class_name) = resolver.using_method_view_impl(method_id, |method_view| {
        Some((method_view.desc().clone(), method_view.name(), method_view.classview().name().try_unwrap_name()?))//todo handle intrinsics on arrays
    })?;

    if class_name == CClassName::unsafe_() {
        return sun_misc_unsafe(resolver, layout, labeler, method_id, ir_method_id, desc, method_name);
    }

    if method_name == MethodName::method_hashCode() && desc == CompressedMethodDescriptor::empty_args(CPDType::IntType) && class_name == CClassName::object() {
        return intrinsic_hashcode(resolver, layout, method_id, ir_method_id);
    }
    if method_name == MethodName::method_getClass() && desc == CompressedMethodDescriptor::empty_args(CClassName::class().into()) && class_name == CClassName::object() {
        return intrinsic_get_class(resolver, layout, method_id, ir_method_id);
    }
    let identity_hash_code = CompressedMethodDescriptor { arg_types: vec![CClassName::object().into()], return_type: CompressedParsedDescriptorType::IntType };
    if method_name == MethodName::method_identityHashCode() && desc == identity_hash_code && class_name == CClassName::system() {
        return system_identity_hashcode(resolver, layout, method_id, ir_method_id);
    }
    let array_copy_hashcode = CompressedMethodDescriptor::void_return(vec![CPDType::object(), CPDType::IntType, CPDType::object(), CPDType::IntType, CPDType::IntType]);
    if method_name == MethodName::method_arraycopy() &&
        desc == array_copy_hashcode &&
        class_name == CClassName::system() {
        return intrinsic_array_copy(resolver, layout, method_id, ir_method_id, labeler);
    }
    let get_component_type_desc = CompressedMethodDescriptor::empty_args(CPDType::class());
    if method_name == MethodName::method_getComponentType() && desc == get_component_type_desc && class_name == CClassName::class() {
        return get_component_type_intrinsic(resolver, layout, method_id, ir_method_id);
    }
    let new_array_desc = CompressedMethodDescriptor { arg_types: vec![CPDType::class(), CPDType::IntType], return_type: CPDType::object() };
    if method_name == MethodName::method_newArray() && desc == new_array_desc && class_name == CClassName::array() {
        return reflect_new_array(resolver, layout, method_id, ir_method_id);
    }
    None
}


pub fn sun_misc_unsafe<'gc>(resolver: &impl MethodResolver<'gc>, layout: &NativeStackframeMemoryLayout, labeler: &mut CompilerLabeler, method_id: MethodId, ir_method_id: IRMethodID, desc: CMethodDescriptor, method_name: MethodName) -> Option<Vec<IRInstr>> {
    let compare_and_swap_long = CompressedMethodDescriptor {
        arg_types: vec![CClassName::object().into(), CPDType::LongType, CPDType::LongType, CPDType::LongType],
        return_type: CPDType::BooleanType,
    };
    if method_name == MethodName::method_compareAndSwapLong() && desc == compare_and_swap_long {
        return intrinsic_compare_and_swap_long(resolver, layout, labeler, method_id, ir_method_id);
    }
    let compare_and_swap_int = CompressedMethodDescriptor {
        arg_types: vec![CPDType::object(), CPDType::LongType, CPDType::IntType, CPDType::IntType],
        return_type: CPDType::BooleanType,
    };
    if method_name == MethodName::method_compareAndSwapInt() && desc == compare_and_swap_int {
        return intrinsic_compare_and_swap_int(resolver, layout, labeler, method_id, ir_method_id);
    }

    let compare_and_swap_obj = CompressedMethodDescriptor {
        arg_types: vec![CPDType::object(), CPDType::LongType, CPDType::object(), CPDType::object()],
        return_type: CPDType::BooleanType,
    };
    if method_name == MethodName::method_compareAndSwapObject() && desc == compare_and_swap_obj {
        return intrinsic_compare_and_swap_object(resolver, layout, labeler, method_id, ir_method_id);
    }

    let address_size_desc = CompressedMethodDescriptor::empty_args(CPDType::IntType);
    if method_name == MethodName::method_addressSize() && desc == address_size_desc {
        return address_size(resolver, layout, method_id, ir_method_id);
    }

    let get_long = CompressedMethodDescriptor {
        arg_types: vec![CPDType::LongType],
        return_type: CompressedParsedDescriptorType::LongType,
    };
    if method_name == MethodName::method_getLong() && desc == get_long {
        return unsafe_get_long_raw(resolver, layout, method_id, ir_method_id);
    }

    let get_byte_desc = CompressedMethodDescriptor {
        arg_types: vec![CPDType::LongType],
        return_type: CompressedParsedDescriptorType::ByteType,
    };
    if method_name == MethodName::method_getByte() && desc == get_byte_desc {
        return unsafe_get_byte_raw(resolver, layout, method_id, ir_method_id);
    }

    let get_int_volatile_desc = CompressedMethodDescriptor { arg_types: vec![CPDType::object(), CPDType::LongType], return_type: CPDType::IntType };
    if method_name == MethodName::method_getIntVolatile() && desc == get_int_volatile_desc {
        return get_int_volatile(resolver, layout, labeler, method_id, ir_method_id);
    }

    let allocate_memory_desc = CompressedMethodDescriptor { arg_types: vec![CPDType::LongType], return_type: CPDType::LongType };
    if method_name == MethodName::method_allocateMemory() && desc == allocate_memory_desc {
        return unsafe_allocate_memory(resolver, layout, method_id, ir_method_id);
    }

    let free_memory_desc = CompressedMethodDescriptor::void_return(vec![CPDType::LongType]);
    if method_name == MethodName::method_freeMemory() && desc == free_memory_desc {
        return unsafe_free_memory(resolver, layout, method_id, ir_method_id);
    }

    let put_long_desc = CompressedMethodDescriptor::void_return(vec![CPDType::LongType, CPDType::LongType]);
    if method_name == MethodName::method_putLong() && desc == put_long_desc {
        return unsafe_put_long(resolver, layout, labeler, method_id, ir_method_id);
    }



    if method_name != MethodName::method_registerNatives() &&
        method_name.0.to_str(resolver.string_pool()) != "arrayBaseOffset" &&
        method_name.0.to_str(resolver.string_pool()) != "objectFieldOffset" &&
        method_name.0.to_str(resolver.string_pool()) != "arrayIndexScale" {
        dbg!(method_name.0.to_str(resolver.string_pool()));
        dbg!(desc.jvm_representation(resolver.string_pool()));
        todo!()
    }
    None
}