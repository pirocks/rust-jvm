use runtime_common::CallStackEntry;
use std::rc::Rc;
use runtime_common::java_values::JavaValue;

pub fn i2l(current_frame: &Rc<CallStackEntry>) -> () {
    let int = current_frame.operand_stack.borrow_mut().pop().unwrap().unwrap_int();
    current_frame.operand_stack.borrow_mut().push(JavaValue::Long(int as i64));
}

pub fn i2f(current_frame: &Rc<CallStackEntry>) -> () {
    let int = current_frame.operand_stack.borrow_mut().pop().unwrap().unwrap_int();
    current_frame.operand_stack.borrow_mut().push(JavaValue::Float(int as f32));
}


pub fn f2i(current_frame: &Rc<CallStackEntry>) -> () {
    let f = current_frame.operand_stack.borrow_mut().pop().unwrap().unwrap_float();
    current_frame.operand_stack.borrow_mut().push(JavaValue::Int(f as i32))
}