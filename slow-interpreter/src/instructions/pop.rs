use rust_jvm_common::runtime_type::RuntimeType;
use verification::OperandStack;
use verification::verifier::Frame;

use crate::java_values::JavaValue;
use crate::jvm_state::JVMState;
use crate::method_table::MethodId;
use crate::stack_entry::StackEntryMut;

pub fn pop2(jvm: &'gc_life JVMState<'gc_life>, method_id: MethodId, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let current_pc = current_frame.to_ref().pc(jvm);
    let stack_frames = &jvm.function_frame_type_data.read().unwrap()[&method_id];
    let Frame { stack_map: OperandStack { data }, .. } = &stack_frames[&current_pc];
    let value1_vtype = data[0].clone();
    let value1 = current_frame.pop(Some(RuntimeType::LongType));
    match value1.to_type() {
        RuntimeType::LongType | RuntimeType::DoubleType => {}
        _ => {
            if let JavaValue::Long(_) | JavaValue::Double(_) = current_frame.pop(Some(RuntimeType::IntType)) {
                panic!()
            };
        }
    };
}

pub fn pop(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) { current_frame.pop(Some(RuntimeType::LongType)); }
