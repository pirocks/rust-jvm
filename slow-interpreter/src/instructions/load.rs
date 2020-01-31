use runtime_common::StackEntry;
use std::rc::Rc;
use runtime_common::java_values::JavaValue;

pub fn aload(current_frame: &Rc<StackEntry>, n: usize) -> () {
    let ref_ = current_frame.local_vars.borrow()[n].clone();
    match ref_.clone() {
        JavaValue::Object(_)  => {}
        _ => {
            dbg!(ref_);
            panic!()
        }
    }
    current_frame.push(ref_);
}

pub fn iload(current_frame: &Rc<StackEntry>, n: usize) {
    let java_val = &current_frame.local_vars.borrow()[n];
    match java_val {
        JavaValue::Int(_) | JavaValue::Boolean(_) | JavaValue::Char(_) => {}
        _ => {
            dbg!(java_val);
            panic!()
        }
    }
    current_frame.push(java_val.clone())
}

pub fn fload(current_frame: &Rc<StackEntry>, n: usize) {
    let java_val = &current_frame.local_vars.borrow()[n];
    match java_val {
        JavaValue::Float(_) => {}
        _ => {
            dbg!(java_val);
            panic!()
        }
    }
    current_frame.push(java_val.clone())
}


pub fn aaload(current_frame: &Rc<StackEntry>) -> () {
    let index = current_frame.pop().unwrap_int();
    let arc = current_frame.pop().unwrap_object().unwrap();
    let unborrowed = arc.unwrap_array();
    let array_refcell= unborrowed.elems.borrow();
//    dbg!(&current_frame.operand_stack);
//    dbg!(&current_frame.local_vars);
    match array_refcell[index as usize] {
        JavaValue::Object(_) => {}
        _ => panic!(),
    }//.unwrap_object();
    current_frame.push(array_refcell[index as usize].clone())
}

pub fn caload(current_frame: &Rc<StackEntry>) -> () {
    let index = current_frame.pop().unwrap_int();
    let arc = current_frame.pop().unwrap_object().unwrap();
    let unborrowed = arc.unwrap_array();
    let array_refcell= unborrowed.elems.borrow();
//    dbg!(&current_frame.operand_stack);
//    dbg!(&current_frame.local_vars);
    match array_refcell[index as usize] {
        JavaValue::Char(_) => {}
        _ => panic!(),
    }//.unwrap_object();
    current_frame.push(array_refcell[index as usize].clone())
}
