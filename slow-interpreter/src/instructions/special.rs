use runtime_common::StackEntry;
use std::rc::Rc;
use runtime_common::java_values::JavaValue;

pub fn arraylength(current_frame: &Rc<StackEntry>) -> () {
    let array = current_frame.pop();
    match array {
        JavaValue::Array(a) => {
            current_frame.push(JavaValue::Int(a.unwrap().borrow().len() as i32));
        }
        _ => panic!()
    }
}