use another_jit_vm::{IRMethodID, Register};
use another_jit_vm::intrinsic_helpers::IntrinsicHelperType;
use another_jit_vm_ir::compiler::{BitwiseLogicType, IRInstr, IRLabel, Size};
use another_jit_vm_ir::vm_exit_abi::IRVMExitType;
use compiler_common::MethodResolver;
use gc_memory_layout_common::frame_layout::NativeStackframeMemoryLayout;
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_descriptors::CompressedMethodDescriptor;
use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPDType};
use rust_jvm_common::compressed_classfile::method_names::MethodName;
use rust_jvm_common::MethodId;

use crate::CompilerLabeler;

pub fn java_lang_object<'gc>(resolver: &impl MethodResolver<'gc>, labeler: &mut CompilerLabeler, layout: &NativeStackframeMemoryLayout, method_id: MethodId, ir_method_id: IRMethodID, desc: &CMethodDescriptor, method_name: MethodName) -> Option<Vec<IRInstr>> {
    if method_name == MethodName::method_hashCode() && desc == &CompressedMethodDescriptor::empty_args(CPDType::IntType) {
        return intrinsic_hashcode(resolver, layout, method_id, ir_method_id);
    }
    if method_name == MethodName::method_getClass() && desc == &CompressedMethodDescriptor::empty_args(CClassName::class().into()) {
        return intrinsic_get_class(resolver, layout, method_id, ir_method_id);
    }
    if method_name == MethodName::method_clone() && desc == &(CompressedMethodDescriptor { arg_types: vec![], return_type: CPDType::object() }) {
        return intrinsic_clone(resolver, labeler, layout, method_id, ir_method_id);
    }
    None
}

pub fn intrinsic_hashcode<'gc>(resolver: &impl MethodResolver<'gc>, layout: &NativeStackframeMemoryLayout, method_id: MethodId, ir_method_id: IRMethodID) -> Option<Vec<IRInstr>> {
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
        IRInstr::Const16bit {
            to: shift_amount,
            const_: 32,
        },
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

pub fn intrinsic_get_class<'gc>(resolver: &impl MethodResolver<'gc>, layout: &NativeStackframeMemoryLayout, method_id: MethodId, ir_method_id: IRMethodID) -> Option<Vec<IRInstr>> {
    return Some(vec![
        IRInstr::IRStart {
            temp_register: Register(2),
            ir_method_id,
            method_id,
            frame_size: layout.full_frame_size(),
            num_locals: resolver.num_locals(method_id) as usize,
        },
        IRInstr::GetClassOrExit {
            object_ref: layout.local_var_entry(0),
            res: Register(0),
            get_class_exit: IRVMExitType::RunSpecialNativeNew { method_id, resolved_fn_ptr: resolver.resolve_native_method_pointer(method_id) },
        },
        IRInstr::Return {
            return_val: Some(Register(0)),
            temp_register_1: Register(1),
            temp_register_2: Register(2),
            temp_register_3: Register(3),
            temp_register_4: Register(4),
            frame_size: layout.full_frame_size(),
        },
    ]);
}


pub fn intrinsic_clone<'gc>(resolver: &impl MethodResolver<'gc>, labeler: &mut CompilerLabeler, layout: &NativeStackframeMemoryLayout, method_id: MethodId, ir_method_id: IRMethodID) -> Option<Vec<IRInstr>> {
    let after_slow_label = labeler.local_label();
    let jvm_clone_resolved = resolver.resolve_native_method_pointer(method_id);
    return Some(vec![
        IRInstr::IRStart {
            temp_register: Register(2),
            ir_method_id,
            method_id,
            frame_size: layout.full_frame_size(),
            num_locals: resolver.num_locals(method_id) as usize,
        },
        IRInstr::LoadFPRelative {
            from: layout.local_var_entry(0),
            to: Register(0),
            size: Size::pointer(),
        },
        IRInstr::CallIntrinsicHelper {
            intrinsic_helper_type: IntrinsicHelperType::FastClone,
            integer_args: vec![Register(0)],
            integer_res: Some(Register(0)),
            float_args: vec![],
            float_res: None,
            double_args: vec![],
            double_res: None,
        },
        IRInstr::Const64bit { to: Register(1), const_: 0 },
        IRInstr::BranchNotEqual {
            a: Register(0),
            b: Register(1),
            label: after_slow_label,
            size: Size::pointer(),
        },
        IRInstr::VMExit2 { exit_type: IRVMExitType::RunSpecialNativeNew { method_id, resolved_fn_ptr: jvm_clone_resolved } },
        IRInstr::Label(IRLabel { name: after_slow_label }),
        IRInstr::Return {
            return_val: Some(Register(0)),
            temp_register_1: Register(1),
            temp_register_2: Register(2),
            temp_register_3: Register(3),
            temp_register_4: Register(4),
            frame_size: layout.full_frame_size(),
        },
    ]);
}


