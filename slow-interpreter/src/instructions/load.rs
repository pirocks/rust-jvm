use runtime_common::CallStackEntry;
use std::rc::Rc;
use runtime_common::java_values::JavaValue;

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
//    dbg!(&current_frame.local_vars);
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
//    dbg!(&current_frame.local_vars);
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
