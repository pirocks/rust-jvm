use another_jit_vm::{IRMethodID, Register};
use another_jit_vm_ir::compiler::{IRInstr, Size};
use gc_memory_layout_common::layout::NativeStackframeMemoryLayout;
use rust_jvm_common::compressed_classfile::CPDType;
use rust_jvm_common::MethodId;
use crate::compiler::fields::field_type_to_register_size;
use crate::compiler_common::MethodResolver;

pub fn unsafe_get_long_raw<'gc>(resolver: &impl MethodResolver<'gc>, layout: &NativeStackframeMemoryLayout, method_id: MethodId, ir_method_id: IRMethodID) -> Option<Vec<IRInstr>> {
    get_raw_common(resolver, layout, method_id, ir_method_id, CPDType::LongType)
}

pub fn unsafe_get_byte_raw<'gc>(resolver: &impl MethodResolver<'gc>, layout: &NativeStackframeMemoryLayout, method_id: MethodId, ir_method_id: IRMethodID) -> Option<Vec<IRInstr>> {
    get_raw_common(resolver, layout, method_id, ir_method_id, CPDType::ByteType)
}

fn get_raw_common<'gc>(resolver: &impl MethodResolver<'gc>, layout: &NativeStackframeMemoryLayout, method_id: MethodId, ir_method_id: IRMethodID, cpdtype: CPDType) -> Option<Vec<IRInstr>> {
    let res = Register(0);
    let ptr = Register(1);
    let size = field_type_to_register_size(cpdtype);
    return Some(vec![
        IRInstr::IRStart {
            temp_register: Register(2),
            ir_method_id,
            method_id,
            frame_size: layout.full_frame_size(),
            num_locals: resolver.num_locals(method_id) as usize,
        },
        IRInstr::LoadFPRelative {
            from: layout.local_var_entry(1),
            to: ptr,
            size: Size::long(),
        },
        IRInstr::Load {
            to: res,
            from_address: ptr,
            size: size,
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
