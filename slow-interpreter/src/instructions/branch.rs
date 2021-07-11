use classfile_view::view::ptype_view::PTypeView;
use rust_jvm_common::classnames::ClassName;

use crate::java_values::{GcManagedObject, JavaValue};
use crate::jvm_state::JVMState;
use crate::runtime_type::RuntimeType;
use crate::stack_entry::StackEntryMut;

pub fn goto_(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, target: i32) {
    *current_frame.pc_offset_mut() = target;
}

pub fn ifnull(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, offset: i16) {
    let val = current_frame.pop(Some(ClassName::object().into()));
    let succeeds = match val {
        JavaValue::Object(o) => o.is_none(),
        _ => panic!()
    };
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}

pub fn ifnonnull(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, offset: i16) {
    let val = current_frame.pop(Some(ClassName::object().into()));
    let succeeds = match val {
        JavaValue::Object(o) => o.is_some(),
        _ => panic!()
    };
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}

pub fn ifle(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, offset: i16) {
    let val = current_frame.pop(Some(RuntimeType::IntType));
    let succeeds = val.unwrap_int() <= 0;
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}

pub fn ifgt(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, offset: i16) {
    let val = current_frame.pop(Some(RuntimeType::IntType));
    let succeeds = val.unwrap_int() > 0;
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}

pub fn ifge(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, offset: i16) {
    let val = current_frame.pop(Some(RuntimeType::IntType));
    let succeeds = val.unwrap_int() >= 0;
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}

pub fn iflt(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, offset: i16) {
    let val = current_frame.pop(Some(RuntimeType::IntType));
    let succeeds = val.unwrap_int() < 0;
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}

pub fn ifne(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, offset: i16) {
    let val = current_frame.pop(Some(RuntimeType::IntType));
    let succeeds = val.unwrap_int() != 0;
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}

pub fn ifeq(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, offset: i16) {
    //todo dup
    let val = current_frame.pop(Some(RuntimeType::IntType));
    let succeeds = val.unwrap_int() == 0;
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}

pub fn if_icmpgt(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, offset: i16) {
    let value2 = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let value1 = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let succeeds = value1 > value2;
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}


pub fn if_icmplt(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, offset: i16) {
    let value2 = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let value1 = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let succeeds = value1 < value2;
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}


pub fn if_icmple(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, offset: i16) {
    let value2 = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let value1 = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let succeeds = value1 <= value2;
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}

pub fn if_icmpge(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, offset: i16) {
    let value2 = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let value1 = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let succeeds = value1 >= value2;
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}


pub fn if_icmpne(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, offset: i16) {
    let value2 = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let value1 = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let succeeds = value1 != value2;
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}


pub fn if_icmpeq(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, offset: i16) {
    let value2 = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let value1 = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let succeeds = value1 == value2;
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}


pub fn if_acmpne(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, offset: i16) {
    let value2 = current_frame.pop(Some(RuntimeType::object()));
    let value1 = current_frame.pop(Some(RuntimeType::object()));
    let succeeds = !equal_ref(value2, value1);
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}


pub fn if_acmpeq(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, offset: i16) {
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
