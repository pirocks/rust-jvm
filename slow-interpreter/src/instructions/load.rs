use classfile_view::view::ptype_view::PTypeView;
use rust_jvm_common::classnames::ClassName;

use crate::{InterpreterStateGuard, JVMState};
use crate::java_values::JavaValue;
use crate::stack_entry::{LocalVarsRef, StackEntryMut};
use crate::utils::throw_array_out_of_bounds;

pub fn aload(jvm: &'l JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, n: u16) {
    let local_vars_ref: LocalVarsRef<'l, 'gc_life> = current_frame.local_vars(jvm);
    let ref_ = local_vars_ref.get(n, PTypeView::object());
    match ref_ {
        JavaValue::Object(_) => {}
        _ => {
            dbg!(ref_);
            dbg!(n);
            // dbg!(&current_frame.local_vars(jvm));
            panic!()
        }
    }
    current_frame.push(jvm, ref_);
}

pub fn iload(jvm: &'l JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, n: u16) {
    let java_val = current_frame.local_vars(jvm).get(n, PTypeView::IntType);
    java_val.unwrap_int();
    current_frame.push(jvm, java_val)
}

pub fn lload(jvm: &'l JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, n: u16) {
    let java_val = current_frame.local_vars(jvm).get(n, PTypeView::LongType);
    match java_val {
        JavaValue::Long(_) => {}
        _ => {
            dbg!(java_val);
            // current_frame.print_stack_trace();
            // dbg!(&current_frame.local_vars(jvm)[1..]);
            panic!()
        }
    }
    current_frame.push(jvm, java_val)
}

pub fn fload(jvm: &'l JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, n: u16) {
    let java_val = current_frame.local_vars(jvm).get(n, PTypeView::FloatType);
    match java_val {
        JavaValue::Float(_) => {}
        _ => {
            dbg!(java_val);
            panic!()
        }
    }
    current_frame.push(jvm, java_val)
}

pub fn dload(jvm: &'l JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, n: u16) {
    let java_val = current_frame.local_vars(jvm).get(n, PTypeView::DoubleType);
    match java_val {
        JavaValue::Double(_) => {}
        _ => {
            dbg!(java_val);
            panic!()
        }
    }
    current_frame.push(jvm, java_val)
}


pub fn aaload(int_state: &'_ mut InterpreterStateGuard<'gc_life, '_>) {
    let jvm = int_state.jvm;
    let mut current_frame = int_state.current_frame_mut();
    let index = current_frame.pop(jvm, PTypeView::IntType).unwrap_int();
    let temp = current_frame.pop(jvm, ClassName::object().into());
    let unborrowed = temp.unwrap_array();
    let jv_res = unborrowed.get_i(int_state.jvm, index);
    if index < 0 || index >= unborrowed.len() {
        return throw_array_out_of_bounds(jvm, int_state, index);
    }
    match jv_res {
        JavaValue::Object(_) => {}
        _ => panic!(),
    };
    current_frame.push(int_state.jvm, jv_res.clone())
}

pub fn caload(jvm: &'l JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, '_>) {
    let index = int_state.pop_current_operand_stack(PTypeView::IntType).unwrap_int();
    let temp = int_state.pop_current_operand_stack(ClassName::object().into());
    let unborrowed = temp.unwrap_array();
    if index < 0 || index >= unborrowed.len() as i32 {
        return throw_array_out_of_bounds(jvm, int_state, index);
    }
    let as_int = match unborrowed.get_i(jvm, index) {
        JavaValue::Char(c) => c as i32,
        _ => panic!(),
    };
    int_state.push_current_operand_stack(JavaValue::Int(as_int))
}


pub fn iaload(jvm: &'l JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let index = current_frame.pop(jvm, PTypeView::IntType).unwrap_int();
    let temp = current_frame.pop(jvm, PTypeView::object());
    let unborrowed = temp.unwrap_array();
    let as_int = unborrowed.get_i(jvm, index).unwrap_int();
    current_frame.push(jvm, JavaValue::Int(as_int))
}


pub fn laload(jvm: &'l JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let index = current_frame.pop(jvm, PTypeView::IntType).unwrap_int();
    let temp = current_frame.pop(jvm, PTypeView::object());
    let unborrowed = temp.unwrap_array();
    let long = unborrowed.get_i(jvm, index).unwrap_long();
    current_frame.push(jvm, JavaValue::Long(long))
}


pub fn faload(jvm: &'l JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let index = current_frame.pop(jvm, PTypeView::IntType).unwrap_int();
    let temp = current_frame.pop(jvm, PTypeView::object());
    let unborrowed = temp.unwrap_array();
    let f = unborrowed.get_i(jvm, index).unwrap_float();
    current_frame.push(jvm, JavaValue::Float(f))
}

pub fn daload(jvm: &'l JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let index = current_frame.pop(jvm, PTypeView::IntType).unwrap_int();
    let temp = current_frame.pop(jvm, PTypeView::object());
    let unborrowed = temp.unwrap_array();
    let d = unborrowed.get_i(jvm, index).unwrap_double();
    current_frame.push(jvm, JavaValue::Double(d))
}


pub fn saload(jvm: &'l JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let index = current_frame.pop(jvm, PTypeView::IntType).unwrap_int();
    let temp = current_frame.pop(jvm, PTypeView::object());
    let unborrowed = temp.unwrap_array();
    let d = unborrowed.get_i(jvm, index).unwrap_short();
    current_frame.push(jvm, JavaValue::Short(d))
}


pub fn baload(jvm: &'l JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let index = current_frame.pop(jvm, PTypeView::IntType).unwrap_int();
    let temp = current_frame.pop(jvm, PTypeView::object());
    let unborrowed = temp.unwrap_array();
    let as_byte = match &unborrowed.get_i(jvm, index) {
        JavaValue::Byte(i) => *i,
        val => {
            dbg!(&unborrowed.elem_type);
            dbg!(val);
            panic!()
        }
    };
    current_frame.push(jvm, JavaValue::Int(as_byte as i32))
}
