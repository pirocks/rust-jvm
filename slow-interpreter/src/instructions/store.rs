use runtime_common::StackEntry;
use std::rc::Rc;
use runtime_common::java_values::JavaValue;

pub fn astore(current_frame: &Rc<StackEntry>, n: usize) -> () {
    let object_ref = current_frame.pop();
    match object_ref.clone() {
        JavaValue::Object(_) | JavaValue::Array(_) => {}
        _ => {
            dbg!(&object_ref);
            panic!()
        }
    }
    current_frame.local_vars.borrow_mut()[n] = object_ref;
}



pub fn castore(current_frame: &Rc<StackEntry>) -> () {
    let val = current_frame.pop().unwrap_int();
    let index = current_frame.pop().unwrap_int();
    let array_ref = current_frame.pop().unwrap_array();
    let char_ = val as u8 as char;
    array_ref.borrow_mut()[index as usize] = JavaValue::Char(char_);
}


pub fn aastore(current_frame: &Rc<StackEntry>) -> () {
    let val = current_frame.pop();
    let index = current_frame.pop().unwrap_int();
    let array_ref = current_frame.pop().unwrap_array();
    match val {
        JavaValue::Object(_) => {},
        _ => panic!(),
    }
    array_ref.borrow_mut()[index as usize] = val.clone();
}
