use rust_jvm_common::compressed_classfile::names::CClassName;
use rust_jvm_common::runtime_type::RuntimeType;

use crate::java_values::{GcManagedObject, JavaValue};
use crate::jvm_state::JVMState;
use crate::stack_entry::StackEntryMut;

pub fn goto_(_jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, target: i32) {
    *current_frame.pc_offset_mut() = target;
}

pub fn ifnull(_jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, offset: i16) {
    let val = current_frame.pop(Some(CClassName::object().into()));
    let succeeds = match val {
        JavaValue::Object(o) => o.is_none(),
        _ => panic!()
    };
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}

pub fn ifnonnull(_jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, offset: i16) {
    let val = current_frame.pop(Some(CClassName::object().into()));
    let succeeds = match val {
        JavaValue::Object(o) => o.is_some(),
        _ => panic!()
    };
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}

pub fn ifle(_jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, offset: i16) {
    let val = current_frame.pop(Some(RuntimeType::IntType));
    let succeeds = val.unwrap_int() <= 0;
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}

pub fn ifgt(_jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, offset: i16) {
    let val = current_frame.pop(Some(RuntimeType::IntType));
    let succeeds = val.unwrap_int() > 0;
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}

pub fn ifge(_jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, offset: i16) {
    let val = current_frame.pop(Some(RuntimeType::IntType));
    let succeeds = val.unwrap_int() >= 0;
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}

pub fn iflt(_jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, offset: i16) {
    let val = current_frame.pop(Some(RuntimeType::IntType));
    let succeeds = val.unwrap_int() < 0;
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}

pub fn ifne(_jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, offset: i16) {
    let val = current_frame.pop(Some(RuntimeType::IntType));
    let succeeds = val.unwrap_int() != 0;
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}

pub fn ifeq(_jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, offset: i16) {
    //todo dup
    let val = current_frame.pop(Some(RuntimeType::IntType));
    let succeeds = val.unwrap_int() == 0;
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}

pub fn if_icmpgt(_jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, offset: i16) {
    let value2 = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let value1 = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let succeeds = value1 > value2;
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}


pub fn if_icmplt(_jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, offset: i16) {
    let value2 = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let value1 = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let succeeds = value1 < value2;
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}


pub fn if_icmple(_jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, offset: i16) {
    let value2 = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let value1 = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let succeeds = value1 <= value2;
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}

pub fn if_icmpge(_jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, offset: i16) {
    let value2 = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let value1 = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let succeeds = value1 >= value2;
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}


pub fn if_icmpne(_jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, offset: i16) {
    let value2 = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let value1 = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let succeeds = value1 != value2;
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}


pub fn if_icmpeq(_jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, offset: i16) {
    let value2 = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let value1 = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let succeeds = value1 == value2;
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}


pub fn if_acmpne(_jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, offset: i16) {
    let value2 = current_frame.pop(Some(RuntimeType::object()));
    let value1 = current_frame.pop(Some(RuntimeType::object()));
    let succeeds = !equal_ref(value2, value1);
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}


pub fn if_acmpeq(_jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, offset: i16) {
    let value2 = current_frame.pop(Some(RuntimeType::object()));
    let value1 = current_frame.pop(Some(RuntimeType::object()));
    let succeeds = equal_ref(value2, value1);
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}

fn equal_ref<'gc_life>(value2: JavaValue<'gc_life>, value1: JavaValue<'gc_life>) -> bool {
    match value1 {
        JavaValue::Object(o1) => match value2 {
            JavaValue::Object(o2) => match o1 {
                None => o2.is_none(),
                Some(o1_arc) => match o2 {
                    None => false,
                    Some(o2_arc) => {
                        GcManagedObject::ptr_eq(&o1_arc, &o2_arc)
                    }
                },
            },
            _ => panic!()
        },
        _ => panic!()
    }
}
