use crate::java_values::JavaValue;
use crate::StackEntry;

pub fn pop2(current_frame: &mut StackEntry) {
    if let _ = current_frame.pop() {
        if let JavaValue::Long(_) | JavaValue::Double(_) = current_frame.pop() {
            panic!()
        };
    };
}

pub fn pop(current_frame: &mut StackEntry) { current_frame.pop(); }
