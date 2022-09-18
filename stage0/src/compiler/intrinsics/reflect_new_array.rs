use std::mem::size_of;
use another_jit_vm::{IRMethodID, Register};
use another_jit_vm_ir::compiler::{IRInstr, Size};
use another_jit_vm_ir::vm_exit_abi::IRVMExitType;
use gc_memory_layout_common::layout::NativeStackframeMemoryLayout;
use runtime_class_stuff::hidden_fields::HiddenJVMField;
use rust_jvm_common::compressed_classfile::CPDType;
use rust_jvm_common::{MethodId, NativeJavaValue};
use crate::compiler_common::MethodResolver;

pub fn reflect_new_array<'gc>(resolver: &impl MethodResolver<'gc>, layout: &NativeStackframeMemoryLayout, method_id: MethodId, ir_method_id: IRMethodID) -> Option<Vec<IRInstr>> {
    let component_type = Register(1);
    let length = Register(2);
    let cpdtype_id = Register(3);
    match resolver.lookup_type_inited_initing(&CPDType::class()) {
        None => {
            panic!("Was expecting class class to be laoded")
        }
        Some((class_class, _)) => {
            let object_layout = &class_class.unwrap_class_class().object_layout;
            //todo should really be using a function for this:
            let cpdtype_id_offset = object_layout.hidden_field_numbers.get(&HiddenJVMField::class_cpdtype_id_of_wrapped_in_array()).unwrap().number.0 * (size_of::<NativeJavaValue>() as u32);
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
                    to: component_type,
                    size: Size::pointer(),
                },
                IRInstr::LoadFPRelative {
                    from: layout.local_var_entry(1),
                    to: length,
                    size: Size::int(),
                },
                IRInstr::AddConst {
                    res: component_type,
                    a: cpdtype_id_offset as i32,
                },
                IRInstr::Load {
                    to: cpdtype_id,
                    from_address: component_type,
                    size: Size::int()
                },
                IRInstr::StoreFPRelative {
                    from: cpdtype_id,
                    to: layout.local_var_entry(0),
                    size: Size::int()
                },
                IRInstr::VMExit2 {
                    exit_type: IRVMExitType::AllocateObjectArrayIntrinsic {
                        array_type: layout.local_var_entry(0),
                        arr_len: layout.local_var_entry(1),
                        arr_res: layout.local_var_entry(0),
                    }
                },
                IRInstr::LoadFPRelative {
                    from: layout.local_var_entry(0),
                    to: Register(0),
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
            ]);
        }
    }
}
