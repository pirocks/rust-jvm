use std::rc::Rc;
use runtime_common::StackEntry;
use runtime_common::java_values::JavaValue;

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



pub fn dup2(current_frame: &Rc<StackEntry>) -> () {
    let value1 = current_frame.pop();
    match value1.clone() {
        JavaValue::Long(_) | JavaValue::Double(_) => {
            current_frame.push(value1.clone());
            current_frame.push(value1);
        },
        _ => {
            let value2 = current_frame.pop();
            match value2.clone(){
                JavaValue::Long(_) | JavaValue::Double(_) => panic!(),
                _ => {},
            };
            current_frame.push(value2.clone());
            current_frame.push(value1.clone());
            current_frame.push(value2.clone());
            current_frame.push(value1.clone());
        },
    }

}
