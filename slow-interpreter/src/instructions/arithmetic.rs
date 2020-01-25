use runtime_common::CallStackEntry;
use std::rc::Rc;
use runtime_common::java_values::JavaValue;

pub fn fmul(current_frame: Rc<CallStackEntry>) -> () {
    let value2 = current_frame.operand_stack.borrow_mut().pop().unwrap().unwrap_float();
    let value1 = current_frame.operand_stack.borrow_mut().pop().unwrap().unwrap_float();
    current_frame.operand_stack.borrow_mut().push(JavaValue::Float(value2 * value1));
}


pub fn land(current_frame: Rc<CallStackEntry>) -> () {
    let first = current_frame.operand_stack.borrow_mut().pop().unwrap().unwrap_long();
    let second = current_frame.operand_stack.borrow_mut().pop().unwrap().unwrap_long();
    current_frame.operand_stack.borrow_mut().push(JavaValue::Long(first & second))
}

pub fn iand(current_frame: &Rc<CallStackEntry>) -> () {
    let first = current_frame.operand_stack.borrow_mut().pop().unwrap().unwrap_int();
    let second = current_frame.operand_stack.borrow_mut().pop().unwrap().unwrap_int();
    current_frame.operand_stack.borrow_mut().push(JavaValue::Int(first & second))
}

pub fn ladd(current_frame: Rc<CallStackEntry>) -> () {
    let first = current_frame.operand_stack.borrow_mut().pop().unwrap().unwrap_long();
    let second = current_frame.operand_stack.borrow_mut().pop().unwrap().unwrap_long();
    current_frame.operand_stack.borrow_mut().push(JavaValue::Long(first + second));
}


pub fn lshl(current_frame: Rc<CallStackEntry>) -> () {
    let value2 = current_frame.operand_stack.borrow_mut().pop().unwrap().unwrap_int();
    let value1 = current_frame.operand_stack.borrow_mut().pop().unwrap().unwrap_long();
    current_frame.operand_stack.borrow_mut().push(JavaValue::Long(value1 << ((value2 & 0x7F) as i64)));
}
