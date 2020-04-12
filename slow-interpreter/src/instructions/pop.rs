use std::rc::Rc;
use crate::java_values::JavaValue;
use crate::StackEntry;

pub fn pop2(current_frame: &StackEntry) {
    match current_frame.pop() {
        JavaValue::Long(_) | JavaValue::Double(_) => {}
        _ => {
            match current_frame.pop() {
                JavaValue::Long(_) | JavaValue::Double(_) => panic!(),
                _ => {}
            };
        }
    };
}

pub fn pop(current_frame: &StackEntry) -> () { current_frame.pop(); }
