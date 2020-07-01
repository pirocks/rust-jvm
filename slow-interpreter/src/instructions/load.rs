use crate::interpreter_util::{check_inited_class, run_constructor, push_new_object};
use rust_jvm_common::classnames::ClassName;
use crate::java_values::JavaValue;
use crate::{StackEntry, JVMState};

pub fn aload(current_frame: &mut StackEntry, n: usize) -> () {
    let ref_ = current_frame.local_vars[n].clone();
    match ref_.clone() {
        JavaValue::Object(_) => {}
        _ => {
            dbg!(ref_);
            dbg!(n);
            dbg!(&current_frame.local_vars);
            panic!()
        }
    }
    current_frame.push(ref_);
}

pub fn iload(current_frame: &mut StackEntry, n: usize) {
    let java_val = &current_frame.local_vars[n];
    java_val.unwrap_int();
    current_frame.push(java_val.clone())
}

pub fn lload(current_frame: &mut StackEntry, n: usize) {
    let java_val = &current_frame.local_vars[n];
    match java_val {
        JavaValue::Long(_) => {}
        _ => {
            dbg!(java_val);
            // current_frame.print_stack_trace();
            dbg!(&current_frame.local_vars[1..]);
            panic!()
        }
    }
    current_frame.push(java_val.clone())
}

pub fn fload(current_frame: &mut StackEntry, n: usize) {
    let java_val = &current_frame.local_vars[n];
    match java_val {
        JavaValue::Float(_) => {}
        _ => {
            dbg!(java_val);
            panic!()
        }
    }
    current_frame.push(java_val.clone())
}

pub fn dload(current_frame: &mut StackEntry, n: usize) {
    let java_val = &current_frame.local_vars[n];
    match java_val {
        JavaValue::Double(_) => {}
        _ => {
            dbg!(java_val);
            panic!()
        }
    }
    current_frame.push(java_val.clone())
}


pub fn aaload(current_frame: &StackEntry) -> () {
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

fn throw_array_out_of_bounds(jvm: &'static JVMState, current_frame: &mut StackEntry) {
    let bounds_class = check_inited_class(
        jvm,
        &ClassName::new("java/lang/ArrayIndexOutOfBoundsException").into(),
        current_frame.class_pointer.loader(jvm).clone()
    );
    push_new_object(jvm, current_frame, &bounds_class, None);
    let obj = current_frame.pop();
    run_constructor(jvm, current_frame, bounds_class, vec![obj.clone()], "()V".to_string());
    *jvm.thread_state.get_current_thread().interpreter_state.throw.write().unwrap() = obj.unwrap_object().into();
}

pub fn caload(state: &'static JVMState, current_frame: &mut StackEntry) -> () {
    let index = current_frame.pop().unwrap_int();
    let temp = current_frame.pop();
    let unborrowed = temp.unwrap_array();
    let array_refcell = unborrowed.elems.borrow();
    if index < 0 || index >= array_refcell.len() as i32 {
        throw_array_out_of_bounds(state, current_frame);
        return;
    }
    let as_int = match array_refcell[index as usize] {
        JavaValue::Char(c) => c as i32,
        _ => panic!(),
    };
    current_frame.push(JavaValue::Int(as_int))
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
