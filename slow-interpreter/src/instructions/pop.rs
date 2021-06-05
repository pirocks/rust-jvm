use crate::java_values::JavaValue;
use crate::stack_entry::StackEntryMut;

pub fn pop2(mut current_frame: StackEntryMut) {
    match current_frame.pop() {
        JavaValue::Long(_) | JavaValue::Double(_) => {}
        _ => {
            if let JavaValue::Long(_) | JavaValue::Double(_) = current_frame.pop() {
                panic!()
            };
        }
    };
}

pub fn pop(mut current_frame: StackEntryMut) { current_frame.pop(); }
