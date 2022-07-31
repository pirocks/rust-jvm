use another_jit_vm::{IRMethodID, Register};
use another_jit_vm_ir::compiler::{IRInstr, Size};
use gc_memory_layout_common::layout::NativeStackframeMemoryLayout;
use rust_jvm_common::MethodId;
use crate::compiler_common::MethodResolver;

pub fn intrinsic_compare_and_swap_long<'gc>(resolver: &impl MethodResolver<'gc>, layout: &NativeStackframeMemoryLayout, method_id: MethodId, ir_method_id: IRMethodID) -> Option<Vec<IRInstr>> {
    let target_obj = Register(1);
    let offset = Register(2);
    let old = Register(3);
    let new = Register(4);
    Some(vec![
        IRInstr::IRStart {
            temp_register: Register(2),
            ir_method_id,
            method_id,
            frame_size: layout.full_frame_size(),
            num_locals: resolver.num_locals(method_id) as usize,
        },
        IRInstr::LoadFPRelative {
            from: layout.local_var_entry(1),
            to: target_obj,
            size: Size::pointer(),
        },
        IRInstr::LoadFPRelative {
            from: layout.local_var_entry(2),
            to: offset,
            size: Size::long(),
        },
        IRInstr::LoadFPRelative {
            from: layout.local_var_entry(4),
            to: old,
            size: Size::long(),
        },
        IRInstr::LoadFPRelative {
            from: layout.local_var_entry(6),
            to: new,
            size: Size::long(),
        },
        IRInstr::Add {
            res: target_obj,
            a: offset,
            size: Size::pointer()
        },
        IRInstr::CompareAndSwapAtomic {
            ptr: target_obj,
            old,
            new,
            res: Register(0),
            rax: Register(0),
            size: Size::long()
        },
        IRInstr::Return {
            return_val: Some(Register(0)),
            temp_register_1: Register(1),
            temp_register_2: Register(2),
            temp_register_3: Register(3),
            temp_register_4: Register(4),
            frame_size: layout.full_frame_size(),
        },
    ])
}

