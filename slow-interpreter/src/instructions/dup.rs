use std::rc::Rc;
use runtime_common::CallStackEntry;

pub fn dup(current_frame: &Rc<CallStackEntry>) -> () {
    let val = current_frame.operand_stack.borrow_mut().pop().unwrap();
    current_frame.operand_stack.borrow_mut().push(val.clone());
    current_frame.operand_stack.borrow_mut().push(val.clone());
}
