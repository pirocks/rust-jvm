use another_jit_vm::{IRMethodID};
use another_jit_vm_ir::compiler::{IRInstr};
use classfile_view::view::ClassView;
use gc_memory_layout_common::layout::NativeStackframeMemoryLayout;
use rust_jvm_common::compressed_classfile::{CompressedMethodDescriptor, CompressedParsedDescriptorType, CPDType};
use rust_jvm_common::compressed_classfile::names::{CClassName, MethodName};
use rust_jvm_common::MethodId;

use crate::compiler::CompilerLabeler;
use crate::compiler::intrinsics::array_copy::intrinsic_array_copy;
use crate::compiler::intrinsics::compare_and_swap::intrinsic_compare_and_swap_long;
use crate::compiler::intrinsics::get_class::intrinsic_get_class;
use crate::compiler::intrinsics::hashcode::intrinsic_hashcode;
use crate::compiler::intrinsics::system_identity_hashcode::system_identity_hashcode;
use crate::compiler_common::MethodResolver;

pub fn gen_intrinsic_ir<'vm>(
    resolver: &impl MethodResolver<'vm>,
    layout: &NativeStackframeMemoryLayout,
    method_id: MethodId,
    ir_method_id: IRMethodID,
    _labeler: &mut CompilerLabeler,
) -> Option<Vec<IRInstr>> {
    let (desc, method_name, class_name) = resolver.using_method_view_impl(method_id, |method_view| {
        Some((method_view.desc().clone(), method_view.name(), method_view.classview().name().try_unwrap_name()?))//todo handle intrinsics on arrays
    })?;

    if method_name == MethodName::method_hashCode() && desc == CompressedMethodDescriptor::empty_args(CPDType::IntType) && class_name == CClassName::object() {
        return intrinsic_hashcode(resolver, layout, method_id, ir_method_id);
    }
    if method_name == MethodName::method_getClass() && desc == CompressedMethodDescriptor::empty_args(CClassName::class().into()) && class_name == CClassName::object() {
        return intrinsic_get_class(resolver, layout, method_id, ir_method_id);
    }
    let compare_and_swap_long = CompressedMethodDescriptor {
        arg_types: vec![CClassName::object().into(), CPDType::LongType, CPDType::LongType, CPDType::LongType],
        return_type: CPDType::BooleanType,
    };
    if method_name == MethodName::method_compareAndSwapLong() && desc == compare_and_swap_long && class_name == CClassName::unsafe_() {
        return intrinsic_compare_and_swap_long(resolver, layout, method_id, ir_method_id);
    }
    let identity_hash_code = CompressedMethodDescriptor { arg_types: vec![CClassName::object().into()], return_type: CompressedParsedDescriptorType::IntType };
    if method_name == MethodName::method_identityHashCode() && desc == identity_hash_code && class_name == CClassName::system() {
        return system_identity_hashcode(resolver, layout, method_id, ir_method_id);
    }
    let array_copy_hashcode = CompressedMethodDescriptor::void_return(vec![CPDType::object(), CPDType::IntType, CPDType::object(), CPDType::IntType, CPDType::IntType]);
    if method_name == MethodName::method_arraycopy() &&
        desc == array_copy_hashcode &&
        class_name == CClassName::system() {
        return intrinsic_array_copy(resolver, layout, method_id, ir_method_id, _labeler);
    }
    None
}


//Java_sun_misc_Unsafe_compareAndSwapLong
pub mod system_identity_hashcode;
pub mod array_copy;
pub mod get_class;
pub mod hashcode;
pub mod compare_and_swap;