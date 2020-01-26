use runtime_common::CallStackEntry;
use std::rc::Rc;
use runtime_common::java_values::JavaValue;

pub fn astore(current_frame: &Rc<CallStackEntry>, n: usize) -> () {
    let object_ref = current_frame.operand_stack.borrow_mut().pop().unwrap();
    match object_ref.clone() {
        JavaValue::Object(_) | JavaValue::Array(_) => {}
        _ => {
            dbg!(&object_ref);
            panic!()
        }
    }
    current_frame.local_vars.borrow_mut()[n] = object_ref;
}



pub fn castore(current_frame: &Rc<CallStackEntry>) -> () {
    let val = current_frame.operand_stack.borrow_mut().pop().unwrap().unwrap_int();
    let index = current_frame.operand_stack.borrow_mut().pop().unwrap().unwrap_int();
    let array_ref = current_frame.operand_stack.borrow_mut().pop().unwrap().unwrap_array();
    let char_ = val as u8 as char;
    array_ref.borrow_mut()[index as usize] = JavaValue::Char(char_);
}


pub fn aastore(current_frame: &Rc<CallStackEntry>) -> () {
    let val = current_frame.operand_stack.borrow_mut().pop().unwrap();
    let index = current_frame.operand_stack.borrow_mut().pop().unwrap().unwrap_int();
    let array_ref = current_frame.operand_stack.borrow_mut().pop().unwrap().unwrap_array();
    match val {
        JavaValue::Object(_) => {},
        _ => panic!(),
    }
    array_ref.borrow_mut()[index as usize] = val.clone();
}
