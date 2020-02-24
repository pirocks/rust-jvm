use runtime_common::java_values::JavaValue;
use std::rc::Rc;
use runtime_common::StackEntry;

pub fn pop2(current_frame: &Rc<StackEntry>) {
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

pub fn pop(current_frame: &Rc<StackEntry>) -> () { current_frame.pop(); }
