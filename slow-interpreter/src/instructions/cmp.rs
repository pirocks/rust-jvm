use crate::java_values::JavaValue;
use crate::StackEntry;

//Floating-point comparison is performed in accordance with IEEE754
// this is the same as regular rust floats


pub fn fcmpl(current_frame: &mut StackEntry) {
    let value2 = current_frame.pop().unwrap_float();
    let value1 = current_frame.pop().unwrap_float();
    if value1.is_nan() || value2.is_nan() {
        current_frame.push(JavaValue::Int(-1));
        return;
    }
    fcmp_common(current_frame, value2, value1)
}

pub fn fcmpg(current_frame: &mut StackEntry) {
    let value2 = current_frame.pop().unwrap_float();
    let value1 = current_frame.pop().unwrap_float();
    if value1.is_nan() || value2.is_nan() {
        current_frame.push(JavaValue::Int(1));
        return;
    }
    fcmp_common(current_frame, value2, value1)
}

fn fcmp_common(current_frame: &mut StackEntry, value2: f32, value1: f32) {
    if value1.to_bits() == value2.to_bits() {
        current_frame.push(JavaValue::Int(0))
    } else if value1 > value2 {
        current_frame.push(JavaValue::Int(1))
    } else if value1 < value2 {
        current_frame.push(JavaValue::Int(-1))
    } else { panic!() }
}

