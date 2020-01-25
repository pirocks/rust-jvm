use runtime_common::CallStackEntry;
use std::rc::Rc;
use runtime_common::java_values::JavaValue;

pub fn fconst_0(current_frame: &Rc<CallStackEntry>) {
    current_frame.operand_stack.borrow_mut().push(JavaValue::Float(0.0));
}

pub fn bipush(current_frame: &Rc<CallStackEntry>, b: u8) -> () {
    current_frame.operand_stack.borrow_mut().push(JavaValue::Int(b as i32))
}
pub fn sipush(current_frame: &Rc<CallStackEntry>, val: u16) {
    current_frame.operand_stack.borrow_mut().push(JavaValue::Int(val as i32));
}



pub fn aconst_null(current_frame: &Rc<CallStackEntry>) -> () {
    current_frame.operand_stack.borrow_mut().push(JavaValue::Object(None))
}