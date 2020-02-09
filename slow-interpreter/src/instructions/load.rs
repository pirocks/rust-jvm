use runtime_common::{StackEntry, InterpreterState};
use std::rc::Rc;
use runtime_common::java_values::JavaValue;
use crate::interpreter_util::{check_inited_class, run_constructor, push_new_object};
use rust_jvm_common::classnames::ClassName;

pub fn aload(current_frame: &Rc<StackEntry>, n: usize) -> () {
    let ref_ = current_frame.local_vars.borrow()[n].clone();
    match ref_.clone() {
        JavaValue::Object(_) => {}
        _ => {
            dbg!(ref_);
            dbg!(n);
//            current_frame.print_stack_trace();
            dbg!(&current_frame.local_vars.borrow());
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

pub fn lload(current_frame: &Rc<StackEntry>, n: usize) {
    let java_val = &current_frame.local_vars.borrow()[n];
    match java_val {
        JavaValue::Long(_) => {}
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
    let array_refcell = unborrowed.elems.borrow();
//    dbg!(&current_frame.operand_stack);
//    dbg!(&current_frame.local_vars);
    match array_refcell[index as usize] {
        JavaValue::Object(_) => {}
        _ => panic!(),
    }//.unwrap_object();
    current_frame.push(array_refcell[index as usize].clone())
}

fn throw_array_out_of_bounds(state: &mut InterpreterState, current_frame: &Rc<StackEntry>) {
    let bounds_class = check_inited_class(state, &ClassName::new("java/lang/ArrayIndexOutOfBoundsException"), current_frame.clone().into(), current_frame.class_pointer.loader.clone());
    push_new_object(current_frame.clone(),&bounds_class);
    let obj = current_frame.pop();
    run_constructor(state,current_frame.clone(),bounds_class,vec![obj.clone()],"()V".to_string());
    state.throw = obj.unwrap_object().into();
}

pub fn caload(state: &mut InterpreterState, current_frame: &Rc<StackEntry>) -> () {
    let index = current_frame.pop().unwrap_int();
    let arc = current_frame.pop().unwrap_object().unwrap();
    let unborrowed = arc.unwrap_array();
    let array_refcell = unborrowed.elems.borrow();
//    dbg!(&current_frame.operand_stack);
//    dbg!(&current_frame.local_vars);
    if index < 0 || index >= array_refcell.len() as i32 {
        throw_array_out_of_bounds(state, current_frame);
        return;
    }
    let as_int = match array_refcell[index as usize] {
        JavaValue::Char(c) => c as i32,
        _ => panic!(),
    };//.unwrap_object();
    current_frame.push(JavaValue::Int(as_int))
}


pub fn iaload(current_frame: &Rc<StackEntry>) -> () {
    let index = current_frame.pop().unwrap_int();
    let arc = current_frame.pop().unwrap_object().unwrap();
    let unborrowed = arc.unwrap_array();
    let array_refcell = unborrowed.elems.borrow();
    let as_int = match array_refcell[index as usize] {
        JavaValue::Int(i) => i,
        _ => panic!(),
    };//.unwrap_object();
    current_frame.push(JavaValue::Int(as_int))
}
