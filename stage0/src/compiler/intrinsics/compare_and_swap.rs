use another_jit_vm::{IRMethodID, Register};
use another_jit_vm_ir::compiler::{IRInstr, Size};
use gc_memory_layout_common::layout::NativeStackframeMemoryLayout;
use rust_jvm_common::compressed_classfile::CPDType;
use rust_jvm_common::MethodId;
use crate::compiler::fields::field_type_to_register_size;

use crate::compiler_common::MethodResolver;

pub fn intrinsic_compare_and_swap_long<'gc>(resolver: &impl MethodResolver<'gc>, layout: &NativeStackframeMemoryLayout, method_id: MethodId, ir_method_id: IRMethodID) -> Option<Vec<IRInstr>> {
    compare_and_swap_common(resolver, layout, method_id, ir_method_id, CPDType::LongType)
}

pub fn intrinsic_compare_and_swap_int<'gc>(resolver: &impl MethodResolver<'gc>, layout: &NativeStackframeMemoryLayout, method_id: MethodId, ir_method_id: IRMethodID) -> Option<Vec<IRInstr>> {
    compare_and_swap_common(resolver, layout, method_id, ir_method_id, CPDType::IntType)
}

fn compare_and_swap_common<'gc>(resolver: &impl MethodResolver<'gc>, layout: &NativeStackframeMemoryLayout, method_id: MethodId, ir_method_id: IRMethodID, cpdtype: CPDType) -> Option<Vec<IRInstr>> {
    let target_obj = Register(1);
    let offset = Register(2);
    let old = Register(3);
    let new = Register(4);

    let (old_offset, new_offset) = if cpdtype.is_double_or_long(){
        (4, 6)
    }else {
        (4,5)
    };
    let size = field_type_to_register_size(cpdtype);
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
            size,
        },
        IRInstr::LoadFPRelative {
            from: layout.local_var_entry(old_offset),
            to: old,
            size,
        },
        IRInstr::LoadFPRelative {
            from: layout.local_var_entry(new_offset),
            to: new,
            size,
        },
        IRInstr::Add {
            res: target_obj,
            a: offset,
            size: Size::pointer(),
        },
        IRInstr::CompareAndSwapAtomic {
            ptr: target_obj,
            old,
            new,
            res: Register(0),
            rax: Register(0),
            size,
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

