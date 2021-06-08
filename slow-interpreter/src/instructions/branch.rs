use std::sync::Arc;

use classfile_view::view::ptype_view::PTypeView;
use rust_jvm_common::classnames::ClassName;

use crate::java_values::JavaValue;
use crate::stack_entry::StackEntryMut;

pub fn goto_(mut current_frame: StackEntryMut, target: i32) {
    *current_frame.pc_offset_mut() = target;
}

pub fn ifnull(mut current_frame: StackEntryMut, offset: i16) {
    let val = current_frame.pop(ClassName::object().into());
    let succeeds = match val {
        JavaValue::Object(o) => o.is_none(),
        _ => panic!()
    };
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}

pub fn ifnonnull(mut current_frame: StackEntryMut, offset: i16) {
    let val = current_frame.pop(ClassName::object().into());
    let succeeds = match val {
        JavaValue::Object(o) => o.is_some(),
        _ => panic!()
    };
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}

pub fn ifle(mut current_frame: StackEntryMut, offset: i16) {
    let val = current_frame.pop(PTypeView::IntType);
    let succeeds = val.unwrap_int() <= 0;
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}

pub fn ifgt(mut current_frame: StackEntryMut, offset: i16) {
    let val = current_frame.pop(PTypeView::IntType);
    let succeeds = val.unwrap_int() > 0;
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}

pub fn ifge(mut current_frame: StackEntryMut, offset: i16) {
    let val = current_frame.pop(PTypeView::IntType);
    let succeeds = val.unwrap_int() >= 0;
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}

pub fn iflt(mut current_frame: StackEntryMut, offset: i16) {
    let val = current_frame.pop(PTypeView::IntType);
    let succeeds = val.unwrap_int() < 0;
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}

pub fn ifne(mut current_frame: StackEntryMut, offset: i16) {
    let val = current_frame.pop(PTypeView::IntType);
    let succeeds = val.unwrap_int() != 0;
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}

pub fn ifeq(mut current_frame: StackEntryMut, offset: i16) {
    //todo dup
    let val = current_frame.pop(PTypeView::IntType);
    let succeeds = val.unwrap_int() == 0;
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}

pub fn if_icmpgt(mut current_frame: StackEntryMut, offset: i16) {
    let value2 = current_frame.pop(PTypeView::IntType).unwrap_int();
    let value1 = current_frame.pop(PTypeView::IntType).unwrap_int();
    let succeeds = value1 > value2;
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}


pub fn if_icmplt(mut current_frame: StackEntryMut, offset: i16) {
    let value2 = current_frame.pop(PTypeView::IntType).unwrap_int();
    let value1 = current_frame.pop(PTypeView::IntType).unwrap_int();
    let succeeds = value1 < value2;
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}


pub fn if_icmple(mut current_frame: StackEntryMut, offset: i16) {
    let value2 = current_frame.pop(PTypeView::IntType).unwrap_int();
    let value1 = current_frame.pop(PTypeView::IntType).unwrap_int();
    let succeeds = value1 <= value2;
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}

pub fn if_icmpge(mut current_frame: StackEntryMut, offset: i16) {
    let value2 = current_frame.pop(PTypeView::IntType).unwrap_int();
    let value1 = current_frame.pop(PTypeView::IntType).unwrap_int();
    let succeeds = value1 >= value2;
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}


pub fn if_icmpne(mut current_frame: StackEntryMut, offset: i16) {
    let value2 = current_frame.pop(PTypeView::IntType).unwrap_int();
    let value1 = current_frame.pop(PTypeView::IntType).unwrap_int();
    let succeeds = value1 != value2;
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}


pub fn if_icmpeq(mut current_frame: StackEntryMut, offset: i16) {
    let value2 = current_frame.pop(PTypeView::IntType).unwrap_int();
    let value1 = current_frame.pop(PTypeView::IntType).unwrap_int();
    let succeeds = value1 == value2;
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}


pub fn if_acmpne(mut current_frame: StackEntryMut, offset: i16) {
    let value2 = current_frame.pop(PTypeView::IntType);
    let value1 = current_frame.pop(PTypeView::IntType);
    let succeeds = !equal_ref(value2, value1);
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}


pub fn if_acmpeq(mut current_frame: StackEntryMut, offset: i16) {
    let value2 = current_frame.pop(PTypeView::IntType);
    let value1 = current_frame.pop(PTypeView::IntType);
    let succeeds = equal_ref(value2, value1);
    if succeeds {
        *current_frame.pc_offset_mut() = offset as i32;
    }
}

fn equal_ref(value2: JavaValue, value1: JavaValue) -> bool {
    match value1 {
        JavaValue::Object(o1) => match value2 {
            JavaValue::Object(o2) => match o1 {
                None => o2.is_none(),
                Some(o1_arc) => match o2 {
                    None => false,
                    Some(o2_arc) => {
                        Arc::ptr_eq(&o1_arc, &o2_arc)
                    }
                },
            },
            _ => panic!()
        },
        _ => panic!()
    }
}
