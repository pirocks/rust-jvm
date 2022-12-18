use std::ffi::c_void;
use nonnull_const::NonNullConst;
use another_jit_vm::{IRMethodID, Register};
use another_jit_vm_ir::compiler::{IRInstr, IRLabel, Size};
use another_jit_vm_ir::vm_exit_abi::IRVMExitType;
use another_jit_vm_ir::vm_exit_abi::runtime_input::TodoCase;
use gc_memory_layout_common::array_copy_no_validate;
use gc_memory_layout_common::frame_layout::NativeStackframeMemoryLayout;
use rust_jvm_common::{ByteCodeOffset, MethodId};
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_descriptors::CompressedMethodDescriptor;
use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CompressedParsedDescriptorType, CPDType};
use rust_jvm_common::compressed_classfile::method_names::MethodName;

use crate::compiler::CompilerLabeler;
use crate::compiler_common::MethodResolver;

pub fn java_lang_system<'gc>(resolver: &impl MethodResolver<'gc>, layout: &NativeStackframeMemoryLayout, method_id: MethodId, ir_method_id: IRMethodID, labeler: &mut CompilerLabeler, desc: &CMethodDescriptor, method_name: MethodName, class_name: CClassName) -> Option<Vec<IRInstr>> {
    let identity_hash_code = CompressedMethodDescriptor { arg_types: vec![CClassName::object().into()], return_type: CompressedParsedDescriptorType::IntType };
    if method_name == MethodName::method_identityHashCode() && desc == &identity_hash_code && class_name == CClassName::system() {
        return system_identity_hashcode(resolver, layout, method_id, ir_method_id);
    }
    let array_copy_hashcode = CompressedMethodDescriptor::void_return(vec![CPDType::object(), CPDType::IntType, CPDType::object(), CPDType::IntType, CPDType::IntType]);
    if method_name == MethodName::method_arraycopy() &&
        desc == &array_copy_hashcode &&
        class_name == CClassName::system() {
        return intrinsic_array_copy(resolver, layout, method_id, ir_method_id, labeler);
    }
    None
}

pub fn intrinsic_array_copy<'gc>(
    resolver: &impl MethodResolver<'gc>,
    layout: &NativeStackframeMemoryLayout,
    method_id: MethodId,
    ir_method_id: IRMethodID,
    labeler: &mut CompilerLabeler,
) -> Option<Vec<IRInstr>> {
    let temp = Register(1);
    let src = Register(2);
    let src_pos = Register(3);
    let dst = Register(4);
    let dst_pos = Register(5);
    let length = Register(6);
    let todo_label = labeler.local_label();
    let mut res = vec![];
    //todo need to validate array store exception again
    res.push(IRInstr::IRStart {
        temp_register: temp,
        ir_method_id,
        method_id,
        frame_size: layout.full_frame_size(),
        num_locals: resolver.num_locals(method_id) as usize,
    });
    // res.push(IRInstr::DebuggerBreakpoint);
    let zero = Register(7);
    res.push(IRInstr::BinaryBitXor {
        res: zero,
        a: zero,
        size: Size::pointer(),
    });
    let src_offset = layout.local_var_entry(0);
    res.push(IRInstr::LoadFPRelative {
        from: src_offset,
        to: src,
        size: Size::pointer(),
    });
    let src_pos_offset = layout.local_var_entry(1);
    res.push(IRInstr::LoadFPRelative {
        from: src_pos_offset,
        to: src_pos,
        size: Size::int(),
    });
    let dst_offset = layout.local_var_entry(2);
    res.push(IRInstr::LoadFPRelative {
        from: dst_offset,
        to: dst,
        size: Size::pointer(),
    });
    let dst_pos_offset = layout.local_var_entry(3);
    res.push(IRInstr::LoadFPRelative {
        from: dst_pos_offset,
        to: dst_pos,
        size: Size::int(),
    });
    let length_offset = layout.local_var_entry(4);
    res.push(IRInstr::LoadFPRelative {
        from: length_offset,
        to: length,
        size: Size::int(),
    });
    res.push(IRInstr::BranchEqual {
        a: src,
        b: zero,
        label: todo_label,
        size: Size::pointer(),
    });
    res.push(IRInstr::BranchEqual {
        a: dst,
        b: zero,
        label: todo_label,
        size: Size::pointer(),
    });
    res.push(IRInstr::BranchAGreaterB {
        a: zero,
        b: src_pos,
        label: todo_label,
        size: Size::int(),
    });
    res.push(IRInstr::BranchAGreaterB {
        a: zero,
        b: dst_pos,
        label: todo_label,
        size: Size::int(),
    });
    res.push(IRInstr::BranchAGreaterB {
        a: zero,
        b: length,
        label: todo_label,
        size: Size::int(),
    });
    let src_length = Register(7);
    // assert_eq!(array_layout.len_entry_offset(), 0);
    res.push(IRInstr::Load {
        to: src_length,
        from_address: src,// + len offset
        size: Size::int(),
    });
    let sum = Register(8);
    res.push(IRInstr::CopyRegister { from: src_pos, to: sum });
    //todo do I need to extend to 64 bit here and on all bounds check
    res.push(IRInstr::Add {
        res: sum,
        a: length,
        size: Size::int(),
    });
    res.push(IRInstr::BranchAGreaterB {
        a: sum,
        b: src_length,
        label: todo_label,
        size: Size::int(),
    });
    let dst_length = Register(7);
    // assert_eq!(array_layout.len_entry_offset(), 0);
    res.push(IRInstr::Load {
        to: dst_length,
        from_address: dst,// + len offset
        size: Size::int(),
    });
    res.push(IRInstr::CopyRegister {
        from: dst_pos,
        to: sum,
    });
    res.push(IRInstr::Add {
        res: sum,
        a: length,
        size: Size::int(),
    });
    res.push(IRInstr::BranchAGreaterB {
        a: sum,
        b: dst_length,
        label: todo_label,
        size: Size::int(),
    });

    let copy_label = labeler.local_label();
    res.push(IRInstr::BranchToLabel { label: copy_label });
    res.push(IRInstr::Label(IRLabel { name: todo_label }));
    // res.push(IRInstr::DebuggerBreakpoint);
    res.push(IRInstr::VMExit2 { exit_type: IRVMExitType::Todo { java_pc: ByteCodeOffset(0), todo_case: TodoCase::ArrayCopyFailure } });
    res.push(IRInstr::Label(IRLabel { name: copy_label }));
    res.push(IRInstr::CallNativeHelper {
        to_call: NonNullConst::new(array_copy_no_validate as *const c_void).unwrap(),
        integer_args: vec![src_offset, src_pos_offset, dst_offset, dst_pos_offset, length_offset],
        integer_res: None,
        float_res: None,
        double_res: None,
        float_double_args: vec![],
    });
    res.push(IRInstr::Return {
        return_val: None,
        temp_register_1: Register(1),
        temp_register_2: Register(2),
        temp_register_3: Register(3),
        temp_register_4: Register(4),
        frame_size: layout.full_frame_size(),
    });
    Some(res)
}


pub fn system_identity_hashcode<'gc>(resolver: &impl MethodResolver<'gc>, layout: &NativeStackframeMemoryLayout, method_id: MethodId, ir_method_id: IRMethodID) -> Option<Vec<IRInstr>> {
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


