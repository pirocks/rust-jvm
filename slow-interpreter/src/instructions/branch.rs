use std::sync::Arc;

use classfile_view::view::ptype_view::PTypeView;
use rust_jvm_common::classnames::ClassName;

use crate::java_values::{GcManagedObject, JavaValue};
use crate::jvm_state::JVMState;
use crate::stack_entry::StackEntryMut;

pub fn goto_(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>, target: i32) {
    *current_frame.pc_offset_mut() = target;
}

pub fn ifnull(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>, offset: i16) {
    let val = current_frame.pop(jvm, ClassName::object().into());
    let succeeds = match val {
        JavaValue::Object(o) => o.is_none(),
        _ => panic!()
    };
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}

pub fn ifnonnull(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>, offset: i16) {
    let val = current_frame.pop(jvm, ClassName::object().into());
    let succeeds = match val {
        JavaValue::Object(o) => o.is_some(),
        _ => panic!()
    };
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}

pub fn ifle(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>, offset: i16) {
    let val = current_frame.pop(jvm, PTypeView::IntType);
    let succeeds = val.unwrap_int() <= 0;
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}

pub fn ifgt(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>, offset: i16) {
    let val = current_frame.pop(jvm, PTypeView::IntType);
    let succeeds = val.unwrap_int() > 0;
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}

pub fn ifge(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>, offset: i16) {
    let val = current_frame.pop(jvm, PTypeView::IntType);
    let succeeds = val.unwrap_int() >= 0;
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}

pub fn iflt(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>, offset: i16) {
    let val = current_frame.pop(jvm, PTypeView::IntType);
    let succeeds = val.unwrap_int() < 0;
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}

pub fn ifne(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>, offset: i16) {
    let val = current_frame.pop(jvm, PTypeView::IntType);
    let succeeds = val.unwrap_int() != 0;
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}

pub fn ifeq(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>, offset: i16) {
    //todo dup
    let val = current_frame.pop(jvm, PTypeView::IntType);
    let succeeds = val.unwrap_int() == 0;
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}

pub fn if_icmpgt(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>, offset: i16) {
    let value2 = current_frame.pop(jvm, PTypeView::IntType).unwrap_int();
    let value1 = current_frame.pop(jvm, PTypeView::IntType).unwrap_int();
    let succeeds = value1 > value2;
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}


pub fn if_icmplt(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>, offset: i16) {
    let value2 = current_frame.pop(jvm, PTypeView::IntType).unwrap_int();
    let value1 = current_frame.pop(jvm, PTypeView::IntType).unwrap_int();
    let succeeds = value1 < value2;
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}


pub fn if_icmple(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>, offset: i16) {
    let value2 = current_frame.pop(jvm, PTypeView::IntType).unwrap_int();
    let value1 = current_frame.pop(jvm, PTypeView::IntType).unwrap_int();
    let succeeds = value1 <= value2;
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}

pub fn if_icmpge(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>, offset: i16) {
    let value2 = current_frame.pop(jvm, PTypeView::IntType).unwrap_int();
    let value1 = current_frame.pop(jvm, PTypeView::IntType).unwrap_int();
    let succeeds = value1 >= value2;
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}


pub fn if_icmpne(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>, offset: i16) {
    let value2 = current_frame.pop(jvm, PTypeView::IntType).unwrap_int();
    let value1 = current_frame.pop(jvm, PTypeView::IntType).unwrap_int();
    let succeeds = value1 != value2;
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}


pub fn if_icmpeq(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>, offset: i16) {
    let value2 = current_frame.pop(jvm, PTypeView::IntType).unwrap_int();
    let value1 = current_frame.pop(jvm, PTypeView::IntType).unwrap_int();
    let succeeds = value1 == value2;
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}


pub fn if_acmpne(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>, offset: i16) {
    let value2 = current_frame.pop(jvm, PTypeView::object());
    let value1 = current_frame.pop(jvm, PTypeView::object());
    let succeeds = !equal_ref(value2, value1);
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}


pub fn if_acmpeq(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>, offset: i16) {
    let value2 = current_frame.pop(jvm, PTypeView::object());
    let value1 = current_frame.pop(jvm, PTypeView::object());
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
