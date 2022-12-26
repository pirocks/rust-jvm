use std::mem::size_of;
use another_jit_vm::{IRMethodID, Register};
use another_jit_vm_ir::compiler::{IRInstr, Size};
use gc_memory_layout_common::frame_layout::NativeStackframeMemoryLayout;
use runtime_class_stuff::hidden_fields::HiddenJVMField;

use rust_jvm_common::{MethodId};
use rust_jvm_common::compressed_classfile::compressed_types::CPDType;
use compiler_common::MethodResolver;

pub fn get_component_type_intrinsic<'gc>(resolver: &impl MethodResolver<'gc>, layout: &NativeStackframeMemoryLayout, method_id: MethodId, ir_method_id: IRMethodID) -> Option<Vec<IRInstr>> {
    match resolver.lookup_type_inited_initing(&CPDType::class()) {
        None => {
            panic!("Was expecting class class to be laoded")
        }
        Some((class_class, _)) => {
            let object_layout = &class_class.unwrap_class_class().object_layout;
            //todo should really be using a function for this:
            //todo should have object layout to get offset for hidden fields
            let component_type_offset = object_layout.hidden_field_numbers.get(&HiddenJVMField::class_component_type()).unwrap().number.0 * (size_of::<u64>() as u32);
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
                    to: Register(1),
                    size: Size::pointer()
                },
                IRInstr::AddConst { res: Register(1), a: component_type_offset as i32 },
                IRInstr::Load {
                    to: Register(0),
                    from_address: Register(1),
                    size: Size::pointer()
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
    }
}
