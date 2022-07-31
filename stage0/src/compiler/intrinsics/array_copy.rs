use std::mem::size_of;

use another_jit_vm::{IRMethodID, Register};
use another_jit_vm::intrinsic_helpers::IntrinsicHelperType;
use another_jit_vm_ir::compiler::{IRInstr, IRLabel, Signed, Size};
use another_jit_vm_ir::vm_exit_abi::IRVMExitType;
use gc_memory_layout_common::layout::{ArrayMemoryLayout, NativeStackframeMemoryLayout};
use rust_jvm_common::{ByteCodeOffset, MethodId, NativeJavaValue};

use crate::compiler::CompilerLabeler;
use crate::compiler_common::MethodResolver;

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
    res.push(IRInstr::LoadFPRelative {
        from: layout.local_var_entry(0),
        to: src,
        size: Size::pointer(),
    });
    res.push(IRInstr::LoadFPRelative {
        from: layout.local_var_entry(1),
        to: src_pos,
        size: Size::int(),
    });
    res.push(IRInstr::LoadFPRelative {
        from: layout.local_var_entry(2),
        to: dst,
        size: Size::pointer(),
    });
    res.push(IRInstr::LoadFPRelative {
        from: layout.local_var_entry(3),
        to: dst_pos,
        size: Size::int(),
    });
    res.push(IRInstr::LoadFPRelative {
        from: layout.local_var_entry(4),
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
    let array_layout = ArrayMemoryLayout::from_unknown_cpdtype();
    assert_eq!(array_layout.len_entry_offset(), 0);
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
    let array_layout = ArrayMemoryLayout::from_unknown_cpdtype();
    assert_eq!(array_layout.len_entry_offset(), 0);
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

    let array_layout = ArrayMemoryLayout::from_unknown_cpdtype();
    assert_eq!(array_layout.elem_0_entry_offset(), 8);
    assert_eq!(array_layout.elem_size(), size_of::<NativeJavaValue>());
    let src_address_register = Register(7);
    res.push(IRInstr::CopyRegister {
        from: src,
        to: src_address_register,
    });
    res.push(IRInstr::AddConst { res: src_address_register, a: 8 });
    res.push(IRInstr::MulConst {
        res: src_pos,
        a: array_layout.elem_size() as i32,
        size: Size::pointer(),
        signed: Signed::Signed
    });
    res.push(IRInstr::Add {
        res: src_address_register,
        a: src_pos,
        size: Size::pointer(),
    });

    let array_layout = ArrayMemoryLayout::from_unknown_cpdtype();
    assert_eq!(array_layout.elem_0_entry_offset(), 8);
    assert_eq!(array_layout.elem_size(), size_of::<NativeJavaValue>());
    let dst_address_register = Register(8);
    res.push(IRInstr::CopyRegister {
        from: dst,
        to: dst_address_register,
    });
    res.push(IRInstr::AddConst { res: dst_address_register, a: 8 });
    res.push(IRInstr::MulConst {
        res: dst_pos,
        a: array_layout.elem_size() as i32,
        size: Size::pointer(),
        signed: Signed::Signed
    });
    res.push(IRInstr::Add {
        res: dst_address_register,
        a: dst_pos,
        size: Size::pointer(),
    });

    res.push(IRInstr::MulConst {
        res: length,
        a: array_layout.elem_size() as i32,
        size: Size::pointer(),
        signed: Signed::Signed //todo this should probe be not this
    });

    let copy_label = labeler.local_label();
    res.push(IRInstr::BranchToLabel { label: copy_label });
    res.push(IRInstr::Label(IRLabel { name: todo_label }));
    // res.push(IRInstr::DebuggerBreakpoint);
    res.push(IRInstr::VMExit2 { exit_type: IRVMExitType::Todo { java_pc: ByteCodeOffset(0) } });
    res.push(IRInstr::Label(IRLabel { name: copy_label }));
    // pub fn memmove(dest: *mut c_void, src: *const c_void, n: size_t) -> *mut c_void;
    res.push(IRInstr::CallIntrinsicHelper {
        intrinsic_helper_type: IntrinsicHelperType::Memmove,
        integer_args: vec![dst_address_register, src_address_register, length],
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
