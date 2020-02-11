use runtime_common::StackEntry;
use std::rc::Rc;
use runtime_common::java_values::JavaValue;

pub fn fcmpl(current_frame: &Rc<StackEntry>) -> () {
    //todo dup
    let value2 = current_frame.pop().unwrap_float();
//    dbg!(value2);
    let value1 = current_frame.pop().unwrap_float();
//    dbg!(value1);
    if value1 == value2 {
        current_frame.push(JavaValue::Int(0))
    } else if value1 > value2 {
        current_frame.push(JavaValue::Int(1))
    } else if value1 < value2 {
        current_frame.push(JavaValue::Int(-1))
    } else {
        current_frame.push(JavaValue::Int(-1))
    }
}

pub fn fcmpg(current_frame: &Rc<StackEntry>) -> () {
    let value2 = current_frame.pop().unwrap_float();
    let value1 = current_frame.pop().unwrap_float();
    if value1 == value2 {
        current_frame.push(JavaValue::Int(0))
    } else if value1 > value2 {
        current_frame.push(JavaValue::Int(1))
    } else if value1 < value2 {
        current_frame.push(JavaValue::Int(-1))
    } else {
        current_frame.push(JavaValue::Int(1))
    }
}
