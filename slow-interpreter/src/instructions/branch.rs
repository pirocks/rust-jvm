use runtime_common::CallStackEntry;
use std::rc::Rc;
use runtime_common::java_values::JavaValue;

pub fn goto_(current_frame: &Rc<CallStackEntry>, target: i16) {
    current_frame.pc_offset.replace(target as isize);
}



pub fn iconst_5(current_frame: &Rc<CallStackEntry>) -> () {
    current_frame.operand_stack.borrow_mut().push(JavaValue::Int(5))
}

pub fn iconst_4(current_frame: &Rc<CallStackEntry>) -> () {
    current_frame.operand_stack.borrow_mut().push(JavaValue::Int(4))
}

pub fn iconst_3(current_frame: &Rc<CallStackEntry>) -> () {
    current_frame.operand_stack.borrow_mut().push(JavaValue::Int(3))
}

pub fn iconst_2(current_frame: &Rc<CallStackEntry>) -> () {
    current_frame.operand_stack.borrow_mut().push(JavaValue::Int(2))
}

pub fn iconst_1(current_frame: &Rc<CallStackEntry>) -> () {
    current_frame.operand_stack.borrow_mut().push(JavaValue::Int(1))
}

pub fn iconst_0(current_frame: &Rc<CallStackEntry>) -> () {
    current_frame.operand_stack.borrow_mut().push(JavaValue::Int(0))
}

pub fn ifnull(current_frame: &Rc<CallStackEntry>, offset: i16) -> () {
    let val = current_frame.operand_stack.borrow_mut().pop().unwrap();
    let succeeds = match val {
        JavaValue::Object(o) => o.is_none(),
        _ => panic!()
    };
    if succeeds {
        current_frame.pc_offset.replace(offset as isize);
    }
}

pub fn ifnonnull(current_frame: &Rc<CallStackEntry>, offset: i16) -> () {
    let val = current_frame.operand_stack.borrow_mut().pop().unwrap();
    let succeeds = match val {
        JavaValue::Object(o) => o.is_some(),
        _ => panic!()
    };
    if succeeds {
        current_frame.pc_offset.replace(offset as isize);
    }
}

pub fn ifle(current_frame: &Rc<CallStackEntry>, offset: i16) -> () {
    //todo dup
    let val = current_frame.operand_stack.borrow_mut().pop().unwrap();
    let succeeds = match val {
        JavaValue::Int(i) => i <= 0,
        _ => panic!()
    };
    if succeeds {
        current_frame.pc_offset.replace(offset as isize);
    }
}

pub fn ifgt(current_frame: &Rc<CallStackEntry>, offset: i16) -> () {
    //todo dup
    let val = current_frame.operand_stack.borrow_mut().pop().unwrap();
    let succeeds = match val {
        JavaValue::Int(i) => i > 0,
        _ => panic!()
    };
    if succeeds {
        current_frame.pc_offset.replace(offset as isize);
    }
}

pub fn ifge(current_frame: &Rc<CallStackEntry>, offset: i16) -> () {
    let val = current_frame.operand_stack.borrow_mut().pop().unwrap();
    let succeeds = match val {
        JavaValue::Int(i) => i >= 0,
        _ => panic!()
    };
    if succeeds {
        current_frame.pc_offset.replace(offset as isize);
    }
}

pub fn iflt(current_frame: &Rc<CallStackEntry>, offset: i16) -> () {
    let val = current_frame.operand_stack.borrow_mut().pop().unwrap();
    let succeeds = match val {
        JavaValue::Int(i) => i < 0,
        _ => panic!()
    };
    if succeeds {
        current_frame.pc_offset.replace(offset as isize);
    }
}

pub fn ifne(current_frame: &Rc<CallStackEntry>, offset: i16) -> () {
    let val = current_frame.operand_stack.borrow_mut().pop().unwrap();
    let succeeds = match val {
        JavaValue::Int(i) => i != 0,
        JavaValue::Boolean(b) => b != false,
        _ => panic!()
    };
    if succeeds {
        current_frame.pc_offset.replace(offset as isize);
    }
}

pub fn ifeq(current_frame: &Rc<CallStackEntry>, offset: i16) -> () {
    //todo dup
    let val = current_frame.operand_stack.borrow_mut().pop().unwrap();
    let succeeds = match val {
        JavaValue::Int(i) => i == 0,
        JavaValue::Boolean(b) => b != false,
        _ => panic!()
    };
    if succeeds {
        current_frame.pc_offset.replace(offset as isize);
    }
}

pub fn if_cmpgt(current_frame: &Rc<CallStackEntry>, offset: i16) -> () {
    let value2 = current_frame.operand_stack.borrow_mut().pop().unwrap();
    let value1 = current_frame.operand_stack.borrow_mut().pop().unwrap();
    let succeeds = match value1 {
        JavaValue::Int(i1) => match value2 {
            JavaValue::Int(i2) => i1 > i2,
            _ => panic!()
        },
        _ => panic!()
    };
    if succeeds {
        current_frame.pc_offset.replace(offset as isize);
    }
}
