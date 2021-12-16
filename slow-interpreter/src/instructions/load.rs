use rust_jvm_common::compressed_classfile::names::CClassName;
use rust_jvm_common::runtime_type::RuntimeType;

use crate::{InterpreterStateGuard, JVMState};
use crate::java_values::JavaValue;
use crate::stack_entry::StackEntryMut;
use crate::utils::throw_array_out_of_bounds;

pub fn aload<'gc_life, 'l>(/*jvm: &'gc_life JVMState<'gc_life>,*/ mut current_frame: StackEntryMut<'gc_life, 'l>, n: u16) {
    let ref_: JavaValue<'gc_life> = current_frame.local_vars().get(n, RuntimeType::object());
    match ref_ {
        JavaValue::Object(_) => {}
        _ => {
            dbg!(ref_);
            dbg!(n);
            // dbg!(&current_frame.local_vars(jvm));
            panic!()
        }
    }
    current_frame.push(ref_);
}

pub fn iload(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, n: u16) {
    let java_val = current_frame.local_vars().get(n, RuntimeType::IntType);
    java_val.unwrap_int();
    current_frame.push(java_val)
}

pub fn lload(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, n: u16) {
    let java_val = current_frame.local_vars().get(n, RuntimeType::LongType);
    match java_val {
        JavaValue::Long(_) => {}
        _ => {
            dbg!(java_val);
            // current_frame.print_stack_trace();
            // dbg!(&current_frame.local_vars(jvm)[1..]);
            panic!()
        }
    }
    current_frame.push(java_val)
}

pub fn fload(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, n: u16) {
    let java_val = current_frame.local_vars().get(n, RuntimeType::FloatType);
    match java_val {
        JavaValue::Float(_) => {}
        _ => {
            dbg!(java_val);
            panic!()
        }
    }
    current_frame.push(java_val)
}

pub fn dload(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, n: u16) {
    let java_val = current_frame.local_vars().get(n, RuntimeType::DoubleType);
    match java_val {
        JavaValue::Double(_) => {}
        _ => {
            dbg!(java_val);
            panic!()
        }
    }
    current_frame.push(java_val)
}

pub fn aaload(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life>) {
    let mut current_frame = int_state.current_frame_mut();
    let index = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let temp = current_frame.pop(Some(CClassName::object().into()));
    let unborrowed = temp.unwrap_array();
    let jv_res = unborrowed.get_i(jvm, index);
    if index < 0 || index >= unborrowed.len() {
        return throw_array_out_of_bounds(jvm, int_state, index);
    }
    match jv_res {
        JavaValue::Object(_) => {}
        _ => panic!(),
    };
    current_frame.push(jv_res.clone())
}

pub fn caload(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life>) {
    let index = int_state.pop_current_operand_stack(Some(RuntimeType::IntType)).unwrap_int();
    let temp = int_state.pop_current_operand_stack(Some(CClassName::object().into()));
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

pub fn iaload(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let index = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let temp = current_frame.pop(Some(RuntimeType::object()));
    let unborrowed = temp.unwrap_array();
    let as_int = unborrowed.get_i(jvm, index).unwrap_int();
    current_frame.push(JavaValue::Int(as_int))
}

pub fn laload(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let index = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let temp = current_frame.pop(Some(RuntimeType::object()));
    let unborrowed = temp.unwrap_array();
    let long = unborrowed.get_i(jvm, index).unwrap_long();
    current_frame.push(JavaValue::Long(long))
}

pub fn faload(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let index = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let temp = current_frame.pop(Some(RuntimeType::object()));
    let unborrowed = temp.unwrap_array();
    let f = unborrowed.get_i(jvm, index).unwrap_float();
    current_frame.push(JavaValue::Float(f))
}

pub fn daload(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let index = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let temp = current_frame.pop(Some(RuntimeType::object()));
    let unborrowed = temp.unwrap_array();
    let d = unborrowed.get_i(jvm, index).unwrap_double();
    current_frame.push(JavaValue::Double(d))
}

pub fn saload(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let index = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let temp = current_frame.pop(Some(RuntimeType::object()));
    let unborrowed = temp.unwrap_array();
    let d = unborrowed.get_i(jvm, index).unwrap_short();
    current_frame.push(JavaValue::Short(d))
}

pub fn baload(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let index = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let temp = current_frame.pop(Some(RuntimeType::object()));
    let unborrowed = temp.unwrap_array();
    let as_byte = match &unborrowed.get_i(jvm, index) {
        JavaValue::Byte(i) => *i,
        JavaValue::Boolean(i) => *i as i8,
        val => {
            dbg!(&unborrowed.elem_type);
            dbg!(val);
            panic!()
        }
    };
    current_frame.push(JavaValue::Int(as_byte as i32))
}