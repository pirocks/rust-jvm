use crate::{InterpreterStateGuard, JVMState, StackEntry};
use crate::java_values::JavaValue;
use crate::utils::throw_array_out_of_bounds;

pub fn aload(current_frame: &mut StackEntry, n: usize) {
    let ref_ = current_frame.local_vars()[n].clone();
    match ref_ {
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


pub fn aaload(int_state: &mut InterpreterStateGuard) {
    // int_state.print_stack_trace();
    let current_frame: &mut StackEntry = int_state.current_frame_mut();
    let index = current_frame.pop().unwrap_int();
    let temp = current_frame.pop();
    let unborrowed = temp.unwrap_array();
    let array_refcell = unborrowed.mut_array();
    match array_refcell[index as usize] {
        JavaValue::Object(_) => {}
        _ => panic!(),
    };
    current_frame.push(array_refcell[index as usize].clone())
}

pub fn caload(jvm: &JVMState, int_state: &mut InterpreterStateGuard) {
    let index = int_state.pop_current_operand_stack().unwrap_int();
    let temp = int_state.pop_current_operand_stack();
    let unborrowed = temp.unwrap_array();
    let array_refcell = unborrowed.mut_array();
    if index < 0 || index >= array_refcell.len() as i32 {
        return throw_array_out_of_bounds(jvm, int_state, index);
    }
    let as_int = match array_refcell[index as usize] {
        JavaValue::Char(c) => c as i32,
        _ => panic!(),
    };
    int_state.push_current_operand_stack(JavaValue::Int(as_int))
}


pub fn iaload(current_frame: &mut StackEntry) {
    let index = current_frame.pop().unwrap_int();
    let temp = current_frame.pop();
    let unborrowed = temp.unwrap_array();
    let array_refcell = unborrowed.mut_array();
    let as_int = array_refcell[index as usize].clone().unwrap_int();
    current_frame.push(JavaValue::Int(as_int))
}


pub fn laload(current_frame: &mut StackEntry) {
    let index = current_frame.pop().unwrap_int();
    let temp = current_frame.pop();
    let unborrowed = temp.unwrap_array();
    let array_refcell = unborrowed.mut_array();
    let long = array_refcell[index as usize].clone().unwrap_long();
    current_frame.push(JavaValue::Long(long))
}


pub fn faload(current_frame: &mut StackEntry) {
    let index = current_frame.pop().unwrap_int();
    let temp = current_frame.pop();
    let unborrowed = temp.unwrap_array();
    let array_refcell = unborrowed.mut_array();
    let f = array_refcell[index as usize].clone().unwrap_float();
    current_frame.push(JavaValue::Float(f))
}

pub fn daload(current_frame: &mut StackEntry) {
    let index = current_frame.pop().unwrap_int();
    let temp = current_frame.pop();
    let unborrowed = temp.unwrap_array();
    let array_refcell = unborrowed.mut_array();
    let d = array_refcell[index as usize].clone().unwrap_double();
    current_frame.push(JavaValue::Double(d))
}


pub fn saload(current_frame: &mut StackEntry) {
    let index = current_frame.pop().unwrap_int();
    let temp = current_frame.pop();
    let unborrowed = temp.unwrap_array();
    let array_refcell = unborrowed.mut_array();
    let d = array_refcell[index as usize].clone().unwrap_short();
    current_frame.push(JavaValue::Short(d))
}


pub fn baload(current_frame: &mut StackEntry) {
    let index = current_frame.pop().unwrap_int();
    let temp = current_frame.pop();
    let unborrowed = temp.unwrap_array();
    let array_refcell = unborrowed.mut_array();
    let as_byte = match &array_refcell[index as usize] {
        JavaValue::Byte(i) => *i,
        val => {
            dbg!(&unborrowed.elem_type);
            dbg!(val);
            panic!()
        }
    };
    current_frame.push(JavaValue::Int(as_byte as i32))
}
