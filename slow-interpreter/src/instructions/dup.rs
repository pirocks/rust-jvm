use crate::java_values::JavaValue;
use crate::StackEntry;

pub fn dup(current_frame: &mut StackEntry) -> () {
    let val = current_frame.pop();
    current_frame.push(val.clone());
    current_frame.push(val.clone());
}

pub fn dup_x1(current_frame: &mut StackEntry) -> () {
    let value1 = current_frame.pop();
    let value2 = current_frame.pop();
    current_frame.push(value1.clone());
    current_frame.push(value2.clone());
    current_frame.push(value1.clone());
}

//todo add impl methods on JavaValue to indicate size

pub fn dup_x2(current_frame: &mut StackEntry) -> () {
    let value1 = current_frame.pop();
    let value2 = current_frame.pop();
    match value2 {
        JavaValue::Long(_) | JavaValue::Double(_) => {
            current_frame.push(value1.clone());
            current_frame.push(value2);
            current_frame.push(value1);
        }
        _ => {
            let value3 = current_frame.pop();
            current_frame.push(value1.clone());
            current_frame.push(value3);
            current_frame.push(value2);
            current_frame.push(value1);
        }
    }
}


pub fn dup2(current_frame: &mut StackEntry) -> () {
    let value1 = current_frame.pop();
    match value1.clone() {
        JavaValue::Long(_) | JavaValue::Double(_) => {
            current_frame.push(value1.clone());
            current_frame.push(value1);
        }
        _ => {
            let value2 = current_frame.pop();
            match value2.clone() {
                JavaValue::Long(_) | JavaValue::Double(_) => panic!(),
                _ => {}
            };
            current_frame.push(value2.clone());
            current_frame.push(value1.clone());
            current_frame.push(value2.clone());
            current_frame.push(value1.clone());
        }
    }
}


pub fn dup2_x1(current_frame: &mut StackEntry) -> () {
    let value1 = current_frame.pop();
    match value1.clone() {
        JavaValue::Long(_) | JavaValue::Double(_) => {
            let value2 = current_frame.pop();
            current_frame.push(value1.clone());
            current_frame.push(value2);
            current_frame.push(value1);
        }
        _ => {
            let value2 = current_frame.pop();
            let value3 = current_frame.pop();
            match value2.clone() {
                JavaValue::Long(_) | JavaValue::Double(_) => panic!(),
                _ => {}
            };
            match value3.clone() {
                JavaValue::Long(_) | JavaValue::Double(_) => panic!(),
                _ => {}
            };
            current_frame.push(value2.clone());
            current_frame.push(value1.clone());
            current_frame.push(value3);
            current_frame.push(value2.clone());
            current_frame.push(value1.clone());
        }
    }
}
