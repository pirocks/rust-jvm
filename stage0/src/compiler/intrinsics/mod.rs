use another_jit_vm::{IRMethodID, Register};
use another_jit_vm_ir::compiler::{BitwiseLogicType, IRInstr, Size};
use classfile_view::view::ClassView;
use gc_memory_layout_common::layout::NativeStackframeMemoryLayout;
use rust_jvm_common::compressed_classfile::{CompressedMethodDescriptor, CPDType};
use rust_jvm_common::compressed_classfile::names::{CClassName, MethodName};
use rust_jvm_common::MethodId;
use crate::compiler_common::MethodResolver;

pub fn gen_intrinsic_ir<'vm>(resolver: &impl MethodResolver<'vm>, layout: &NativeStackframeMemoryLayout, method_id: MethodId, ir_method_id: IRMethodID) -> Option<Vec<IRInstr>> {
    let (desc, method_name, class_name) = resolver.using_method_view_impl(method_id, |method_view| {
        Some((method_view.desc().clone(), method_view.name(), method_view.classview().name().try_unwrap_name()?))//todo handle intrinsics on arrays
    })?;

    if method_name == MethodName::method_hashCode() && desc == CompressedMethodDescriptor::empty_args(CPDType::IntType) && class_name == CClassName::object(){
        let temp = Register(1);
        let res = Register(0);
        let shift_amount = Register(2);
        let arg_frame_pointer_offset = layout.local_var_entry(0);
        return Some(vec![
            IRInstr::IRStart {
                temp_register: Register(2),
                ir_method_id,
                method_id,
                frame_size: layout.full_frame_size(),
                num_locals: resolver.num_locals(method_id) as usize,
            },
            IRInstr::LoadFPRelative {
                from: arg_frame_pointer_offset,
                to: temp,
                size: Size::pointer(),
            },
            IRInstr::LoadFPRelative {
                from: arg_frame_pointer_offset,
                to: res,
                size: Size::pointer(),
            },
            IRInstr::Const16bit { to: shift_amount, const_: 32 },
            IRInstr::ShiftRight {
                res,
                a: shift_amount,
                cl_aka_register_2: shift_amount,
                size: Size::pointer(),
                signed: BitwiseLogicType::Logical,
            },
            IRInstr::BinaryBitXor {
                res,
                a: temp,
                size: Size::int(),
            },
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
    None
}