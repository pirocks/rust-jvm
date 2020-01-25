use runtime_common::CallStackEntry;
use std::rc::Rc;
use runtime_common::java_values::JavaValue;

pub fn fcmpl(current_frame: &Rc<CallStackEntry>) -> () {
    //todo dup
    let value2 = current_frame.operand_stack.borrow_mut().pop().unwrap().unwrap_float();
    let value1 = current_frame.operand_stack.borrow_mut().pop().unwrap().unwrap_float();
    if value1 == value2 {
        current_frame.operand_stack.borrow_mut().push(JavaValue::Int(0))
    } else if value1 > value2 {
        current_frame.operand_stack.borrow_mut().push(JavaValue::Int(1))
    } else if value1 < value2 {
        current_frame.operand_stack.borrow_mut().push(JavaValue::Int(-1))
    } else {
        current_frame.operand_stack.borrow_mut().push(JavaValue::Int(-1))
    }
}

pub fn fcmpg(current_frame: &Rc<CallStackEntry>) -> () {
    let value2 = current_frame.operand_stack.borrow_mut().pop().unwrap().unwrap_float();
    let value1 = current_frame.operand_stack.borrow_mut().pop().unwrap().unwrap_float();
    if value1 == value2 {
        current_frame.operand_stack.borrow_mut().push(JavaValue::Int(0))
    } else if value1 > value2 {
        current_frame.operand_stack.borrow_mut().push(JavaValue::Int(1))
    } else if value1 < value2 {
        current_frame.operand_stack.borrow_mut().push(JavaValue::Int(-1))
    } else {
        current_frame.operand_stack.borrow_mut().push(JavaValue::Int(1))
    }
}
