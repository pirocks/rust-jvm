use std::sync::Arc;
use crate::java_values::JavaValue;
use crate::StackEntry;

pub fn goto_(current_frame: & StackEntry, target: i16) {
    current_frame.pc_offset.replace(target as isize);
}

pub fn ifnull(current_frame: & StackEntry, offset: i16) -> () {
    let val = current_frame.pop();
    let succeeds = match val {
        JavaValue::Object(o) => o.is_none(),
        _ => panic!()
    };
    if succeeds {
        current_frame.pc_offset.replace(offset as isize);
    }
}

pub fn ifnonnull(current_frame: & StackEntry, offset: i16) -> () {
    let val = current_frame.pop();
    let succeeds = match val {
        JavaValue::Object(o) => o.is_some(),
        _ => panic!()
    };
    if succeeds {
        current_frame.pc_offset.replace(offset as isize);
    }
}

pub fn ifle(current_frame: & StackEntry, offset: i16) -> () {
    let val = current_frame.pop();
    let succeeds = val.unwrap_int() <= 0;
    if succeeds {
        current_frame.pc_offset.replace(offset as isize);
    }
}

pub fn ifgt(current_frame: & StackEntry, offset: i16) -> () {
    let val = current_frame.pop();
    let succeeds = val.unwrap_int() > 0;
    if succeeds {
        current_frame.pc_offset.replace(offset as isize);
    }
}

pub fn ifge(current_frame: & StackEntry, offset: i16) -> () {
    let val = current_frame.pop();
    let succeeds = val.unwrap_int() >= 0;
    if succeeds {
        current_frame.pc_offset.replace(offset as isize);
    }
}

pub fn iflt(current_frame: & StackEntry, offset: i16) -> () {
    let val = current_frame.pop();
    let succeeds = val.unwrap_int() < 0;
    if succeeds {
        current_frame.pc_offset.replace(offset as isize);
    }
}

pub fn ifne(current_frame: & StackEntry, offset: i16) -> () {
    let val = current_frame.pop();
    let succeeds = val.unwrap_int() != 0;
    if succeeds {
        current_frame.pc_offset.replace(offset as isize);
    }
}

pub fn ifeq(current_frame: & StackEntry, offset: i16) -> () {
    //todo dup
    let val = current_frame.pop();
    let succeeds = val.unwrap_int() == 0;
    if succeeds {
        current_frame.pc_offset.replace(offset as isize);
    }
}

pub fn if_icmpgt(current_frame: & StackEntry, offset: i16) -> () {
    let value2 = current_frame.pop().unwrap_int();
    let value1 = current_frame.pop().unwrap_int();
    let succeeds = value1 > value2;
    if succeeds {
        current_frame.pc_offset.replace(offset as isize);
    }
}


pub fn if_icmplt(current_frame: & StackEntry, offset: i16) -> () {
    let value2 = current_frame.pop().unwrap_int();
    let value1 = current_frame.pop().unwrap_int();
    let succeeds = value1 < value2;
    if succeeds {
        current_frame.pc_offset.replace(offset as isize);
    }
}


pub fn if_icmple(current_frame: & StackEntry, offset: i16) -> () {
    let value2 = current_frame.pop().unwrap_int();
    let value1 = current_frame.pop().unwrap_int();
    let succeeds = value1 <= value2;
    if succeeds {
        current_frame.pc_offset.replace(offset as isize);
    }
}

pub fn if_icmpge(current_frame: & StackEntry, offset: i16) -> () {
    let value2 = current_frame.pop().unwrap_int();
    let value1 = current_frame.pop().unwrap_int();
    let succeeds = value1 >= value2;
    if succeeds {
        current_frame.pc_offset.replace(offset as isize);
    }
}


pub fn if_icmpne(current_frame: & StackEntry, offset: i16) -> () {
    let value2 = current_frame.pop().unwrap_int();
    let value1 = current_frame.pop().unwrap_int();
    let succeeds = value1 != value2;
    if succeeds {
        current_frame.pc_offset.replace(offset as isize);
    }
}


pub fn if_icmpeq(current_frame: & StackEntry, offset: i16) -> () {
    let value2 = current_frame.pop().unwrap_int();
    let value1 = current_frame.pop().unwrap_int();
    let succeeds = value1 == value2;
    if succeeds {
        current_frame.pc_offset.replace(offset as isize);
    }
}


pub fn if_acmpne(current_frame: & StackEntry, offset: i16) -> () {
    let value2 = current_frame.pop();
    let value1 = current_frame.pop();
    let succeeds = !equal_ref(value2, value1);
    if succeeds {
        current_frame.pc_offset.replace(offset as isize);
    }
}


pub fn if_acmpeq(current_frame: & StackEntry, offset: i16) -> () {
    let value2 = current_frame.pop();
    let value1 = current_frame.pop();
    let succeeds = equal_ref(value2, value1);
    if succeeds {
        current_frame.pc_offset.replace(offset as isize);
    }
}

fn equal_ref(value2: JavaValue, value1: JavaValue) -> bool {
    match value1 {
        JavaValue::Object(o1) => match value2 {
            JavaValue::Object(o2) => match o1 {
                None => match o2 {
                    None => true,
                    Some(_) => false,
                },
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
