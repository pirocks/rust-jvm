use std::rc::Rc;
use std::sync::Arc;
use crate::java_values::JavaValue;
use crate::StackEntry;

pub fn goto_(current_frame: &Rc<StackEntry>, target: i16) {
    current_frame.pc_offset.replace(target as isize);
}

//todo why are these consts in branch?
pub fn iconst_5(current_frame: &Rc<StackEntry>) -> () {
    current_frame.push(JavaValue::Int(5))
}

pub fn iconst_4(current_frame: &Rc<StackEntry>) -> () {
    current_frame.push(JavaValue::Int(4))
}

pub fn iconst_3(current_frame: &Rc<StackEntry>) -> () {
    current_frame.push(JavaValue::Int(3))
}

pub fn iconst_2(current_frame: &Rc<StackEntry>) -> () {
    current_frame.push(JavaValue::Int(2))
}

pub fn iconst_1(current_frame: &Rc<StackEntry>) -> () {
    current_frame.push(JavaValue::Int(1))
}

pub fn iconst_0(current_frame: &Rc<StackEntry>) -> () {
    current_frame.push(JavaValue::Int(0))
}

pub fn dconst_1(current_frame: &Rc<StackEntry>) -> () {
    current_frame.push(JavaValue::Double(1.0))
}

pub fn dconst_0(current_frame: &Rc<StackEntry>) -> () {
    current_frame.push(JavaValue::Double(0.0))
}

pub fn iconst_m1(current_frame: &Rc<StackEntry>) -> () {
    current_frame.push(JavaValue::Int(-1))
}

pub fn lconst(current_frame: &Rc<StackEntry>, i: i64) -> () {
    current_frame.push(JavaValue::Long(i))
}

pub fn ifnull(current_frame: &Rc<StackEntry>, offset: i16) -> () {
    let val = current_frame.pop();
    let succeeds = match val {
        JavaValue::Object(o) => o.is_none(),
        _ => panic!()
    };
    if succeeds {
        current_frame.pc_offset.replace(offset as isize);
    }
}

pub fn ifnonnull(current_frame: &Rc<StackEntry>, offset: i16) -> () {
    let val = current_frame.pop();
    let succeeds = match val {
        JavaValue::Object(o) => o.is_some(),
        _ => panic!()
    };
    if succeeds {
        current_frame.pc_offset.replace(offset as isize);
    }
}

pub fn ifle(current_frame: &Rc<StackEntry>, offset: i16) -> () {
    //todo dup
    let val = current_frame.pop();
    let succeeds = match val {
        JavaValue::Int(i) => i <= 0,
        _ => panic!()
    };
    if succeeds {
        current_frame.pc_offset.replace(offset as isize);
    }
}

pub fn ifgt(current_frame: &Rc<StackEntry>, offset: i16) -> () {
    //todo dup
    let val = current_frame.pop();
    let succeeds = match val {
        JavaValue::Int(i) => i > 0,
        _ => panic!()
    };
    if succeeds {
        current_frame.pc_offset.replace(offset as isize);
    }
}

pub fn ifge(current_frame: &Rc<StackEntry>, offset: i16) -> () {
    let val = current_frame.pop();
    let succeeds = match val {
        JavaValue::Int(i) => i >= 0,
        _ => panic!()
    };
    if succeeds {
        current_frame.pc_offset.replace(offset as isize);
    }
}

pub fn iflt(current_frame: &Rc<StackEntry>, offset: i16) -> () {
    let val = current_frame.pop();
    let succeeds = match val {
        JavaValue::Int(i) => i < 0,
        _ => panic!()
    };
    if succeeds {
        current_frame.pc_offset.replace(offset as isize);
    }
}

pub fn ifne(current_frame: &Rc<StackEntry>, offset: i16) -> () {
    let val = current_frame.pop();
    let succeeds = match val {
        JavaValue::Int(i) => i != 0,
        JavaValue::Boolean(b) => b != false,
        _ => panic!()
    };
    if succeeds {
        current_frame.pc_offset.replace(offset as isize);
    }
}

pub fn ifeq(current_frame: &Rc<StackEntry>, offset: i16) -> () {
    //todo dup
    let val = current_frame.pop();
    let succeeds = match val {
        JavaValue::Int(i) => i == 0,
        JavaValue::Boolean(b) => b == false,//todo cover shorts etc. in every place where relevant
        _ => panic!()
    };
    if succeeds {
        current_frame.pc_offset.replace(offset as isize);
    }
}

pub fn if_icmpgt(current_frame: &Rc<StackEntry>, offset: i16) -> () {
    let value2 = current_frame.pop().unwrap_int();
    let value1 = current_frame.pop().unwrap_int();
    let succeeds = value1 > value2;
    if succeeds {
        current_frame.pc_offset.replace(offset as isize);
    }
}


pub fn if_icmplt(current_frame: &Rc<StackEntry>, offset: i16) -> () {
    let value2 = current_frame.pop().unwrap_int();
    let value1 = current_frame.pop().unwrap_int();
    let succeeds = value1 < value2;
    if succeeds {
        current_frame.pc_offset.replace(offset as isize);
    }
}


pub fn if_icmple(current_frame: &Rc<StackEntry>, offset: i16) -> () {
    let value2 = current_frame.pop().unwrap_int();
    let value1 = current_frame.pop().unwrap_int();
    let succeeds = value1 <= value2;
    if succeeds {
        current_frame.pc_offset.replace(offset as isize);
    }
}

pub fn if_icmpge(current_frame: &Rc<StackEntry>, offset: i16) -> () {
    let value2 = current_frame.pop().unwrap_int();
    let value1 = current_frame.pop().unwrap_int();
    let succeeds = value1 >= value2;
    if succeeds {
        current_frame.pc_offset.replace(offset as isize);
    }
}


pub fn if_icmpne(current_frame: &Rc<StackEntry>, offset: i16) -> () {
    let value2 = current_frame.pop().unwrap_int();
    let value1 = current_frame.pop().unwrap_int();
    let succeeds = value1 != value2;
    if succeeds {
        current_frame.pc_offset.replace(offset as isize);
    }
}


pub fn if_icmpeq(current_frame: &Rc<StackEntry>, offset: i16) -> () {
    let value2 = current_frame.pop().unwrap_int();
    let value1 = current_frame.pop().unwrap_int();
    let succeeds = value1 == value2;
    if succeeds {
        current_frame.pc_offset.replace(offset as isize);
    }
}


pub fn if_acmpne(current_frame: &Rc<StackEntry>, offset: i16) -> () {
    let value2 = current_frame.pop();
    let value1 = current_frame.pop();
    let succeeds = !equal_ref(value2, value1);
    if succeeds {
        current_frame.pc_offset.replace(offset as isize);
    }
}


pub fn if_acmpeq(current_frame: &Rc<StackEntry>, offset: i16) -> () {
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
                    None => true,
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
