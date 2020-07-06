use crate::java_values::JavaValue;
use crate::StackEntry;

pub fn fcmpl(current_frame: &mut StackEntry) -> () {
    let value2 = current_frame.pop().unwrap_float();
    let value1 = current_frame.pop().unwrap_float();
    //todo check this actually handles Nan correctly
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

pub fn fcmpg(current_frame: &mut StackEntry) -> () {
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
