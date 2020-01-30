use std::rc::Rc;
use runtime_common::StackEntry;

pub fn dup(current_frame: &Rc<StackEntry>) -> () {
    let val = current_frame.pop();
    current_frame.push(val.clone());
    current_frame.push(val.clone());
}

pub fn dup_x1(current_frame: &Rc<StackEntry>) -> () {
    let value1 = current_frame.pop();
    let value2 = current_frame.pop();
    current_frame.push(value1.clone());
    current_frame.push(value2.clone());
    current_frame.push(value1.clone());
}
