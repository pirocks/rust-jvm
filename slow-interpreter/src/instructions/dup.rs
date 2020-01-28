use std::rc::Rc;
use runtime_common::StackEntry;

pub fn dup(current_frame: &Rc<StackEntry>) -> () {
    let val = current_frame.pop();
    current_frame.push(val.clone());
    current_frame.push(val.clone());
}
