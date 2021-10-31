use rust_jvm_common::runtime_type::RuntimeType;
use verification::OperandStack;
use verification::verifier::Frame;

use crate::jvm_state::JVMState;
use crate::method_table::MethodId;
use crate::stack_entry::StackEntryMut;

pub fn dup(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let val = current_frame.pop(None);//type doesn't currently matter so do whatever(well it has to be 64 bit).//todo fix for when type does matter
    current_frame.push(val.clone());
    current_frame.push(val);
}

pub fn dup_x1(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let value1 = current_frame.pop(None);//type doesn't matter
    let value2 = current_frame.pop(None);//type doesn't matter
    current_frame.push(value1.clone());
    current_frame.push(value2);
    current_frame.push(value1);
}

pub fn dup_x2(jvm: &'gc_life JVMState<'gc_life>, method_id: MethodId, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let current_pc = current_frame.to_ref().pc(jvm);
    let stack_frames = &jvm.function_frame_type_data.read().unwrap()[&method_id];
    let Frame { stack_map: OperandStack { data }, .. } = &stack_frames[&current_pc];
    /*let value2_vtype = data[1].clone();*/
    let value1 = current_frame.pop(None);//in principle type doesn't matter
    let value2 = current_frame.pop(None);
    match value1.to_type() {
        RuntimeType::LongType | RuntimeType::DoubleType => {
            current_frame.push(value1.clone());
            current_frame.push(value2);
            current_frame.push(value1);
        }
        _ => {
            let value3 = current_frame.pop(None);//in principle type doesn't matter todo pass it anyway
            current_frame.push(value1.clone());
            current_frame.push(value3);
            current_frame.push(value2);
            current_frame.push(value1);
        }
    }
}


pub fn dup2(jvm: &'gc_life JVMState<'gc_life>, method_id: MethodId, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let current_pc = current_frame.to_ref().pc(jvm);
    let stack_frames = &jvm.function_frame_type_data.read().unwrap()[&method_id];
    let Frame { stack_map: OperandStack { data }, .. } = &stack_frames[&current_pc];
    /*let value1_vtype = if matches!(data[0].clone(),VType::TopType) {
        data[1].clone()
    } else {
        data[0].clone()
    };*/
    let value1 = current_frame.pop(None);//in principle type doesn't matter todo pass it anyway
    match value1.to_type() {
        RuntimeType::LongType | RuntimeType::DoubleType => {
            current_frame.push(value1.clone());
            current_frame.push(value1);
        }
        _ => {
            let value2 = current_frame.pop(None);
            // match value2 {
            //     JavaValue::Long(_) | JavaValue::Double(_) => panic!(),
            //     _ => {}
            // };
            current_frame.push(value2.clone());
            current_frame.push(value1.clone());
            current_frame.push(value2);
            current_frame.push(value1);
        }
    }
}


pub fn dup2_x1(jvm: &'gc_life JVMState<'gc_life>, method_id: MethodId, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let (rc, count) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
    let current_pc = current_frame.to_ref().pc(jvm);
    let stack_frames = &jvm.function_frame_type_data.read().unwrap()[&method_id];
    let Frame { stack_map: OperandStack { data }, .. } = &stack_frames[&current_pc];
    let value1_vtype = data[0].clone();
    dbg!(&value1_vtype);
    let value1 = current_frame.pop(None);//in principle type doesn't matter todo pass it anyway
    match value1.to_type() {
        RuntimeType::LongType | RuntimeType::DoubleType => {
            let value2 = current_frame.pop(None);
            current_frame.push(value1.clone());
            current_frame.push(value2);
            current_frame.push(value1);
        }
        _ => {
            let value2 = current_frame.pop(None);
            let value3 = current_frame.pop(None);
            // match value2 {
            //     JavaValue::Long(_) | JavaValue::Double(_) => panic!(),
            //     _ => {}
            // };
            // match value3 {
            //     JavaValue::Long(_) | JavaValue::Double(_) => panic!(),
            //     _ => {}
            // };
            current_frame.push(value2.clone());
            current_frame.push(value1.clone());
            current_frame.push(value3);
            current_frame.push(value2);
            current_frame.push(value1);
        }
    }
}
