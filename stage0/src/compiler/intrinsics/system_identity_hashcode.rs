use another_jit_vm::{IRMethodID, Register};
use another_jit_vm_ir::compiler::{IRInstr, Size};
use gc_memory_layout_common::layout::NativeStackframeMemoryLayout;
use rust_jvm_common::MethodId;

use crate::compiler_common::MethodResolver;

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
