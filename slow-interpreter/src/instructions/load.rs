use classfile_view::view::ptype_view::PTypeView;
use rust_jvm_common::classnames::ClassName;

use crate::{InterpreterStateGuard, JVMState};
use crate::java_values::JavaValue;
use crate::stack_entry::StackEntryMut;
use crate::utils::throw_array_out_of_bounds;

pub fn aload(mut current_frame: StackEntryMut, n: u16) {
    let ref_ = current_frame.local_vars().get(n, PTypeView::object());
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

pub fn iload(mut current_frame: StackEntryMut, n: u16) {
    let java_val = current_frame.local_vars().get(n, PTypeView::IntType);
    java_val.unwrap_int();
    current_frame.push(java_val)
}

pub fn lload(mut current_frame: StackEntryMut, n: u16) {
    let java_val = current_frame.local_vars().get(n, PTypeView::LongType);
    match java_val {
        JavaValue::Long(_) => {}
        _ => {
            dbg!(java_val);
            // current_frame.print_stack_trace();
            // dbg!(&current_frame.local_vars()[1..]);
            panic!()
        }
    }
    current_frame.push(java_val)
}

pub fn fload(mut current_frame: StackEntryMut, n: u16) {
    let java_val = current_frame.local_vars().get(n, PTypeView::FloatType);
    match java_val {
        JavaValue::Float(_) => {}
        _ => {
            dbg!(java_val);
            panic!()
        }
    }
    current_frame.push(java_val)
}

pub fn dload(mut current_frame: StackEntryMut, n: u16) {
    let java_val = current_frame.local_vars().get(n, PTypeView::DoubleType);
    match java_val {
        JavaValue::Double(_) => {}
        _ => {
            dbg!(java_val);
            panic!()
        }
    }
    current_frame.push(java_val)
}


pub fn aaload(int_state: &'_ mut InterpreterStateGuard<'gc_life, '_>) {
    let mut current_frame = int_state.current_frame_mut();
    let index = current_frame.pop(PTypeView::IntType).unwrap_int();
    let temp = current_frame.pop(ClassName::object().into());
    let unborrowed = temp.unwrap_array();
    let array_refcell = unborrowed.mut_array();
    match array_refcell[index as usize] {
        JavaValue::Object(_) => {}
        _ => panic!(),
    };
    current_frame.push(array_refcell[index as usize].clone())
}

pub fn caload(jvm: &'_ JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, '_>) {
    let index = int_state.pop_current_operand_stack(PTypeView::IntType).unwrap_int();
    let temp = int_state.pop_current_operand_stack(ClassName::object().into());
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


pub fn iaload(mut current_frame: StackEntryMut) {
    let index = current_frame.pop(PTypeView::IntType).unwrap_int();
    let temp = current_frame.pop(PTypeView::object());
    let unborrowed = temp.unwrap_array();
    let array_refcell = unborrowed.mut_array();
    let as_int = array_refcell[index as usize].clone().unwrap_int();
    current_frame.push(JavaValue::Int(as_int))
}


pub fn laload(mut current_frame: StackEntryMut) {
    let index = current_frame.pop(PTypeView::IntType).unwrap_int();
    let temp = current_frame.pop(PTypeView::object());
    let unborrowed = temp.unwrap_array();
    let array_refcell = unborrowed.mut_array();
    let long = array_refcell[index as usize].clone().unwrap_long();
    current_frame.push(JavaValue::Long(long))
}


pub fn faload(mut current_frame: StackEntryMut) {
    let index = current_frame.pop(PTypeView::IntType).unwrap_int();
    let temp = current_frame.pop(PTypeView::object());
    let unborrowed = temp.unwrap_array();
    let array_refcell = unborrowed.mut_array();
    let f = array_refcell[index as usize].clone().unwrap_float();
    current_frame.push(JavaValue::Float(f))
}

pub fn daload(mut current_frame: StackEntryMut) {
    let index = current_frame.pop(PTypeView::IntType).unwrap_int();
    let temp = current_frame.pop(PTypeView::object());
    let unborrowed = temp.unwrap_array();
    let array_refcell = unborrowed.mut_array();
    let d = array_refcell[index as usize].clone().unwrap_double();
    current_frame.push(JavaValue::Double(d))
}


pub fn saload(mut current_frame: StackEntryMut) {
    let index = current_frame.pop(PTypeView::IntType).unwrap_int();
    let temp = current_frame.pop(PTypeView::object());
    let unborrowed = temp.unwrap_array();
    let array_refcell = unborrowed.mut_array();
    let d = array_refcell[index as usize].clone().unwrap_short();
    current_frame.push(JavaValue::Short(d))
}


pub fn baload(mut current_frame: StackEntryMut) {
    let index = current_frame.pop(PTypeView::IntType).unwrap_int();
    let temp = current_frame.pop(PTypeView::object());
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
