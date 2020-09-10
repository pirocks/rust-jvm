use rust_jvm_common::classnames::ClassName;

use crate::{InterpreterStateGuard, JVMState, StackEntry};
use crate::interpreter_util::{check_inited_class, push_new_object, run_constructor};
use crate::java_values::JavaValue;

pub fn aload(current_frame: &mut StackEntry, n: usize) -> () {
    let ref_ = current_frame.local_vars()[n].clone();
    match ref_.clone() {
        JavaValue::Object(_) => {}
        _ => {
            dbg!(ref_);
            dbg!(n);
            dbg!(&current_frame.local_vars());
            panic!()
        }
    }
    current_frame.push(ref_);
}

pub fn iload(current_frame: &mut StackEntry, n: usize) {
    let java_val = current_frame.local_vars()[n].clone();
    java_val.unwrap_int();
    current_frame.push(java_val)
}

pub fn lload(current_frame: &mut StackEntry, n: usize) {
    let java_val = current_frame.local_vars()[n].clone();
    match java_val {
        JavaValue::Long(_) => {}
        _ => {
            dbg!(java_val);
            // current_frame.print_stack_trace();
            dbg!(&current_frame.local_vars()[1..]);
            panic!()
        }
    }
    current_frame.push(java_val)
}

pub fn fload(current_frame: &mut StackEntry, n: usize) {
    let java_val = current_frame.local_vars()[n].clone();
    match java_val {
        JavaValue::Float(_) => {}
        _ => {
            dbg!(java_val);
            panic!()
        }
    }
    current_frame.push(java_val)
}

pub fn dload(current_frame: &mut StackEntry, n: usize) {
    let java_val = current_frame.local_vars()[n].clone();
    match java_val {
        JavaValue::Double(_) => {}
        _ => {
            dbg!(java_val);
            panic!()
        }
    }
    current_frame.push(java_val)
}


pub fn aaload(current_frame: &mut StackEntry) -> () {
    let index = current_frame.pop().unwrap_int();
    let temp = current_frame.pop();
    let unborrowed = temp.unwrap_array();
    let array_refcell = unborrowed.elems.borrow();
    match array_refcell[index as usize] {
        JavaValue::Object(_) => {}
        _ => panic!(),
    };
    current_frame.push(array_refcell[index as usize].clone())
}

fn throw_array_out_of_bounds<'l>(jvm: &'static JVMState, int_state: &mut InterpreterStateGuard) {
    let bounds_class = check_inited_class(
        jvm,
        int_state,
        &ClassName::new("java/lang/ArrayIndexOutOfBoundsException").into(),
        int_state.current_loader(jvm),
    );
    push_new_object(jvm, int_state, &bounds_class, None);
    let obj = int_state.current_frame_mut().pop();
    run_constructor(jvm, int_state, bounds_class, vec![obj.clone()], "()V".to_string());
    int_state.set_throw(obj.unwrap_object().into());
}

pub fn caload<'l>(state: &'static JVMState, int_state: &mut InterpreterStateGuard) -> () {
    let index = int_state.pop_current_operand_stack().unwrap_int();
    let temp = int_state.pop_current_operand_stack();
    let unborrowed = temp.unwrap_array();
    let array_refcell = unborrowed.elems.borrow();
    if index < 0 || index >= array_refcell.len() as i32 {
        throw_array_out_of_bounds(state, int_state);
        return;
    }
    let as_int = match array_refcell[index as usize] {
        JavaValue::Char(c) => c as i32,
        _ => panic!(),
    };
    int_state.push_current_operand_stack(JavaValue::Int(as_int))
}


pub fn iaload(current_frame: &mut StackEntry) -> () {
    let index = current_frame.pop().unwrap_int();
    let temp = current_frame.pop();
    let unborrowed = temp.unwrap_array();
    let array_refcell = unborrowed.elems.borrow();
    let as_int = array_refcell[index as usize].clone().unwrap_int();
    current_frame.push(JavaValue::Int(as_int))
}


pub fn laload(current_frame: &mut StackEntry) -> () {
    let index = current_frame.pop().unwrap_int();
    let temp = current_frame.pop();
    let unborrowed = temp.unwrap_array();
    let array_refcell = unborrowed.elems.borrow();
    let long = array_refcell[index as usize].clone().unwrap_long();
    current_frame.push(JavaValue::Long(long))
}


pub fn baload(current_frame: &mut StackEntry) -> () {
    let index = current_frame.pop().unwrap_int();
    let temp = current_frame.pop();
    let unborrowed = temp.unwrap_array();
    let array_refcell = unborrowed.elems.borrow();
    let as_byte = match array_refcell[index as usize] {
        JavaValue::Byte(i) => i,
        _ => panic!(),
    };
    current_frame.push(JavaValue::Int(as_byte as i32))
}
