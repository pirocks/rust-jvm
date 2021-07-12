use jvmti_jni_bindings::jbyte;
use rust_jvm_common::compressed_classfile::CPDType;
use rust_jvm_common::runtime_type::RuntimeType;

use crate::interpreter_state::InterpreterStateGuard;
use crate::java_values::JavaValue;
use crate::jvm_state::JVMState;
use crate::stack_entry::StackEntryMut;
use crate::utils::throw_npe;

pub fn astore(mut current_frame: StackEntryMut<'gc_life, 'l>, n: u16) {
    let object_ref: JavaValue<'gc_life> = current_frame.pop(Some(RuntimeType::object()));
    match &object_ref {
        JavaValue::Object(_) => {}
        _ => {
            dbg!(&object_ref);
            panic!()
        }
    }
    let mut vars_mut = current_frame.local_vars_mut();
    vars_mut.set(n, object_ref);
}

pub fn lstore(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, n: u16) {
    let val = current_frame.pop(Some(RuntimeType::LongType));
    match val {
        JavaValue::Long(_) => {}
        _ => {
            dbg!(&val);
            panic!()
        }
    }
    current_frame.local_vars_mut().set(n, val);
}

pub fn dstore(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, n: u16) {
    let jv = current_frame.pop(Some(RuntimeType::DoubleType));
    match jv {
        JavaValue::Double(_) => {}
        _ => {
            dbg!(&jv);
            panic!()
        }
    }
    current_frame.local_vars_mut().set(n, jv);
}

pub fn fstore(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, n: u16) {
    let jv = current_frame.pop(Some(RuntimeType::FloatType));
    jv.unwrap_float();
    current_frame.local_vars_mut().set(n, jv);
}

pub fn castore(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>) {
    let mut current_frame = int_state.current_frame_mut();
    let val = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let index = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let arrar_ref_o = match current_frame.pop(Some(RuntimeType::object())).unwrap_object() {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state);
        }
    };
    let char_ = val as u16;
    arrar_ref_o.unwrap_array().set_i(jvm, index, JavaValue::Char(char_));
}

pub fn bastore(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>) {
    let mut current_frame = int_state.current_frame_mut();
    let val = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int() as jbyte;// int value is truncated
    let index = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let array_ref_o = match current_frame.pop(Some(RuntimeType::object())).unwrap_object() {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state);
        }
    };
    assert!(array_ref_o.unwrap_array().elem_type == CPDType::ByteType || array_ref_o.unwrap_array().elem_type == CPDType::BooleanType);
    let array_ref = array_ref_o.unwrap_array();
    array_ref.set_i(jvm, index, JavaValue::Byte(val));
}


pub fn sastore(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>) {
    let mut current_frame = int_state.current_frame_mut();
    let val = current_frame.pop(Some(RuntimeType::IntType)).unwrap_short();
    let index = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let array_ref_o = match current_frame.pop(Some(RuntimeType::object())).unwrap_object() {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state);
        }
    };
    assert_eq!(array_ref_o.unwrap_array().elem_type, CPDType::ShortType);
    let array_ref = array_ref_o.unwrap_array();
    array_ref.set_i(jvm, index, JavaValue::Short(val));
}


pub fn fastore(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>) {
    let mut current_frame = int_state.current_frame_mut();
    let val = current_frame.pop(Some(RuntimeType::FloatType)).unwrap_float();
    let index = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let array_ref_o = match current_frame.pop(Some(RuntimeType::object())).unwrap_object() {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state);
        }
    };
    let array_ref = array_ref_o.unwrap_array();
    array_ref.set_i(jvm, index, JavaValue::Float(val));
}


pub fn dastore(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>) {
    let mut current_frame = int_state.current_frame_mut();
    let val = current_frame.pop(Some(RuntimeType::DoubleType)).unwrap_double();
    let index = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let array_ref_o = match current_frame.pop(Some(RuntimeType::object())).unwrap_object() {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state);
        }
    };
    let array_ref = array_ref_o.unwrap_array();
    array_ref.set_i(jvm, index, JavaValue::Double(val));
}


pub fn iastore(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>) {
    let mut current_frame = int_state.current_frame_mut();
    let val = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let index = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let arrar_ref_o = match current_frame.pop(Some(RuntimeType::object())).unwrap_object() {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state);
        }
    };
    let array_ref = arrar_ref_o.unwrap_array();
    let int_ = val;
    array_ref.set_i(jvm, index, JavaValue::Int(int_));
}


pub fn aastore(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>) {
    let mut current_frame = int_state.current_frame_mut();
    let val = current_frame.pop(Some(RuntimeType::object()));
    let index = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let arrary_ref_o = match current_frame.pop(Some(RuntimeType::object())).unwrap_object() {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state);
        }
    };
    let array_ref = arrary_ref_o.unwrap_array();
    match val {
        JavaValue::Object(_) => {}
        _ => panic!(),
    }
    array_ref.set_i(jvm, index, val);
}


pub fn istore(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, n: u16) {
    let object_ref = current_frame.pop(Some(RuntimeType::IntType));
    current_frame.local_vars_mut().set(n, JavaValue::Int(object_ref.unwrap_int()));
}


pub fn lastore(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>) {
    let mut current_frame = int_state.current_frame_mut();
    let val = current_frame.pop(Some(RuntimeType::LongType)).unwrap_long();
    let index = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let arrar_ref_o = match current_frame.pop(Some(RuntimeType::object())).unwrap_object() {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state);
        }
    };
    let array_ref = arrar_ref_o.unwrap_array();
    let long = val;
    array_ref.set_i(jvm, index, JavaValue::Long(long));
}
