use std::os::raw::c_void;
use std::vec;

use nonnull_const::NonNullConst;

use another_jit_vm::{FramePointerOffset, IRMethodID, Register};
use another_jit_vm_ir::compiler::{IRInstr, Size};
use classfile_view::view::{ClassView, HasAccessFlags};
use gc_memory_layout_common::frame_layout::NativeStackframeMemoryLayout;
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_descriptors::CompressedMethodDescriptor;
use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CompressedParsedDescriptorType, CPDType};
use rust_jvm_common::compressed_classfile::method_names::MethodName;
use rust_jvm_common::MethodId;

use crate::compiler::CompilerLabeler;
use crate::compiler::intrinsics::get_component_type::get_component_type_intrinsic;
use crate::compiler::intrinsics::java_lang_object::java_lang_object;
use crate::compiler::intrinsics::java_lang_system::java_lang_system;
use crate::compiler::intrinsics::reflect_new_array::reflect_new_array;
use crate::compiler::intrinsics::sun_misc_unsafe::sun_misc_unsafe;
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
        if let Some(res) = sun_misc_unsafe(resolver, layout, method_id, ir_method_id, &desc, method_name) {
            return Some(res);
        }
    }

    if class_name == CClassName::object() {
        if let Some(res) = java_lang_object(resolver, layout, method_id, ir_method_id, &desc, method_name) {
            return Some(res);
        }
    }

    if class_name == CClassName::system() {
        if let Some(res) = java_lang_system(resolver, layout, method_id, ir_method_id, labeler, &desc, method_name, class_name) {
            return Some(res);
        }
    }

    let get_component_type_desc = CompressedMethodDescriptor::empty_args(CPDType::class());
    if method_name == MethodName::method_getComponentType() && desc == get_component_type_desc && class_name == CClassName::class() {
        if let Some(res) = get_component_type_intrinsic(resolver, layout, method_id, ir_method_id) {
            return Some(res);
        }
    }
    let new_array_desc = CompressedMethodDescriptor { arg_types: vec![CPDType::class(), CPDType::IntType], return_type: CPDType::object() };
    if method_name == MethodName::method_newArray() && desc == new_array_desc && class_name == CClassName::array() {
        if let Some(res) = reflect_new_array(resolver, layout, method_id, ir_method_id) {
            return Some(res);
        }
    }

    return direct_invoke_check(resolver, layout, desc, method_name, class_name, method_id, ir_method_id);
}

fn direct_invoke_check<'gc>(resolver: &impl MethodResolver<'gc>, layout: &NativeStackframeMemoryLayout, desc: CMethodDescriptor, method_name: MethodName, class_name: CClassName, method_id: MethodId, ir_method_id: IRMethodID) -> Option<Vec<IRInstr>> {
    match resolver.lookup_type_inited_initing(&class_name.into()) {
        None => {
            None
        }
        Some((class, _loader)) => {
            if let Some(direct_invoke) = resolver.is_direct_invoke(class, method_name, desc.clone()) {
                let (byte_res, bool_res, short_res, char_res, integer_res, float_res, double_res, res_size) = match desc.return_type {
                    CompressedParsedDescriptorType::ByteType => {
                        (Some(layout.local_var_entry(0)), None, None, None, None, None, None, Size::int())
                    }
                    CompressedParsedDescriptorType::BooleanType => {
                        (None, Some(layout.local_var_entry(0)), None, None, None, None, None, Size::int())
                    }
                    CompressedParsedDescriptorType::ShortType => {
                        (None, None, Some(layout.local_var_entry(0)), None, None, None, None, Size::int())
                    }
                    CompressedParsedDescriptorType::CharType => {
                        (None, None, None, Some(layout.local_var_entry(0)), None, None, None, Size::int())
                    }
                    CompressedParsedDescriptorType::IntType => {
                        (None, None, None, None, Some(layout.local_var_entry(0)), None, None, Size::int())
                    }
                    CompressedParsedDescriptorType::Class(_) |
                    CompressedParsedDescriptorType::Array { .. } => {
                        (None, None, None, None, Some(layout.local_var_entry(0)), None, None, Size::pointer())
                    }
                    CompressedParsedDescriptorType::LongType => {
                        (None, None, None, None, Some(layout.local_var_entry(0)), None, None, Size::long())
                    }
                    CompressedParsedDescriptorType::FloatType => {
                        (None, None, None, None, None, Some(layout.local_var_entry(0)), None, Size::float())
                    }
                    CompressedParsedDescriptorType::DoubleType => {
                        (None, None, None, None, None, None, Some(layout.local_var_entry(0)), Size::double())
                    }
                    CompressedParsedDescriptorType::VoidType => {
                        (None, None, None, None, None, None, None, Size::pointer())
                    }
                };
                let mut integer_args = vec![];
                let mut arg_index = 0;
                resolver.using_method_view_impl(method_id, |method_view| {
                    integer_args.push(FramePointerOffset(0));//the env pointer
                    if !method_view.is_static() {
                        integer_args.push(layout.local_var_entry(arg_index));
                        arg_index += 1;
                    }
                });
                let mut float_double_args = vec![];
                for arg_type in desc.arg_types {
                    match arg_type {
                        CompressedParsedDescriptorType::BooleanType |
                        CompressedParsedDescriptorType::ByteType |
                        CompressedParsedDescriptorType::ShortType |
                        CompressedParsedDescriptorType::CharType |
                        CompressedParsedDescriptorType::IntType |
                        CompressedParsedDescriptorType::Class(_) |
                        // CompressedParsedDescriptorType::LongType|
                        CompressedParsedDescriptorType::Array { .. } => {
                            integer_args.push(layout.local_var_entry(arg_index));
                        }
                        CompressedParsedDescriptorType::LongType => {
                            integer_args.push(layout.local_var_entry(arg_index));
                            arg_index += 1;
                        }
                        CompressedParsedDescriptorType::FloatType => {
                            float_double_args.push((layout.local_var_entry(arg_index), Size::float()));
                        }
                        CompressedParsedDescriptorType::DoubleType => {
                            float_double_args.push((layout.local_var_entry(arg_index), Size::double()));
                            arg_index += 1;
                        }
                        CompressedParsedDescriptorType::VoidType => {
                            todo!()
                        }
                    }
                    arg_index += 1;
                }
                Some(vec![
                    IRInstr::IRStart {
                        temp_register: Register(2),
                        ir_method_id,
                        method_id,
                        frame_size: layout.full_frame_size(),
                        num_locals: resolver.num_locals(method_id) as usize,
                    },
                    IRInstr::CallNativeHelper {
                        to_call: NonNullConst::new(direct_invoke as *const c_void).unwrap(),
                        integer_args,
                        byte_res,
                        bool_res,
                        char_res,
                        short_res,
                        integer_res,
                        float_double_args,
                        float_res,
                        double_res,
                    },
                    IRInstr::LoadFPRelative {
                        from: layout.local_var_entry(0),
                        to: Register(0),
                        size: res_size,
                    },
                    IRInstr::Return {
                        return_val: Some(Register(0)),
                        temp_register_1: Register(1),
                        temp_register_2: Register(2),
                        temp_register_3: Register(3),
                        temp_register_4: Register(4),
                        frame_size: layout.full_frame_size(),
                    }])
            } else {
                None
            }
        }
    }
}


