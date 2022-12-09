use another_jit_vm::IRMethodID;
use another_jit_vm_ir::compiler::IRInstr;
use classfile_view::view::ClassView;
use gc_memory_layout_common::frame_layout::NativeStackframeMemoryLayout;
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_descriptors::CompressedMethodDescriptor;
use rust_jvm_common::compressed_classfile::compressed_types::{CPDType};
use rust_jvm_common::compressed_classfile::method_names::MethodName;


use rust_jvm_common::MethodId;

use crate::compiler::CompilerLabeler;
use crate::compiler::intrinsics::get_component_type::get_component_type_intrinsic;
use crate::compiler::intrinsics::java_lang_object::java_lang_object;
use crate::compiler::intrinsics::java_lang_system::java_lang_system;
use crate::compiler::intrinsics::reflect_new_array::reflect_new_array;
use crate::compiler::intrinsics::sun_misc_unsafe::{sun_misc_unsafe};
use crate::compiler_common::MethodResolver;

pub mod sun_misc_unsafe;
pub mod reflect_new_array;
pub mod get_component_type;
pub mod java_lang_system;
pub mod java_lang_object;

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

    if class_name == CClassName::object() {
        return java_lang_object(resolver, layout, method_id, ir_method_id, &desc, method_name);
    }

    if class_name == CClassName::system() {
        return java_lang_system(resolver, layout, method_id, ir_method_id, labeler, &desc, method_name, class_name);
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


