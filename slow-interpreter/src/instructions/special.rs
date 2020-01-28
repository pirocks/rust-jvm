use runtime_common::CallStackEntry;
use std::rc::Rc;
use runtime_common::java_values::JavaValue;

pub fn arraylength(current_frame: &Rc<CallStackEntry>) -> () {
    let array = current_frame.operand_stack.borrow_mut().pop().unwrap();
    match array {
        JavaValue::Array(a) => {
            current_frame.operand_stack.borrow_mut().push(JavaValue::Int(a.unwrap().borrow().len() as i32));
        }
        _ => panic!()
    }
}