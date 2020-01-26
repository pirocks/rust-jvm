use runtime_common::CallStackEntry;
use std::rc::Rc;
use runtime_common::java_values::JavaValue;
use std::borrow::Borrow;
use std::cell::RefCell;

pub fn aload(current_frame: &Rc<CallStackEntry>, n: usize) -> () {
    let ref_ = current_frame.local_vars.borrow()[n].clone();
    match ref_.clone() {
        JavaValue::Object(_) | JavaValue::Array(_) => {}
        _ => {
            dbg!(ref_);
            panic!()
        }
    }
    current_frame.operand_stack.borrow_mut().push(ref_);
}

pub fn iload(current_frame: &Rc<CallStackEntry>, n: usize) {
    let java_val = &current_frame.local_vars.borrow()[n];
    match java_val {
        JavaValue::Int(_) | JavaValue::Boolean(_) => {}
        _ => {
            dbg!(java_val);
            panic!()
        }
    }
    current_frame.operand_stack.borrow_mut().push(java_val.clone())
}

pub fn fload(current_frame: &Rc<CallStackEntry>, n: usize) {
    let java_val = &current_frame.local_vars.borrow()[n];
    match java_val {
        JavaValue::Float(_) => {}
        _ => {
            dbg!(java_val);
            panic!()
        }
    }
    current_frame.operand_stack.borrow_mut().push(java_val.clone())
}





pub fn aaload(current_frame: &Rc<CallStackEntry>) -> () {
    let index = current_frame.operand_stack.borrow_mut().pop().unwrap().unwrap_int();
    let unborrowed = current_frame.operand_stack.borrow_mut().pop().unwrap().unwrap_array();
    let array_refcell: &RefCell<Vec<JavaValue>> = unborrowed.borrow();
    let second_borrow = array_refcell.borrow();
//    dbg!(&current_frame.operand_stack);
//    dbg!(&current_frame.local_vars);
    match second_borrow[index as usize]{
        JavaValue::Array(_) => {},
        JavaValue::Object(_) => {},
        _ => panic!(),
    }//.unwrap_object();
    current_frame.operand_stack.borrow_mut().push(second_borrow[index as usize].clone())
}
