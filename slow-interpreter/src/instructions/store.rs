use classfile_view::view::ptype_view::PTypeView;
use jvmti_jni_bindings::jbyte;

use crate::interpreter_state::InterpreterStateGuard;
use crate::java_values::JavaValue;
use crate::jvm_state::JVMState;
use crate::StackEntry;
use crate::utils::throw_npe;

pub fn astore(current_frame: &mut StackEntry, n: usize) {
    let object_ref = current_frame.pop();
    match object_ref {
        JavaValue::Object(_) => {}
        _ => {
            dbg!(&object_ref);
            panic!()
        }
    }
    current_frame.local_vars_mut()[n] = object_ref;
}

pub fn lstore(current_frame: &mut StackEntry, n: usize) {
    let val = current_frame.pop();
    match val {
        JavaValue::Long(_) => {}
        _ => {
            dbg!(&val);
            panic!()
        }
    }
    current_frame.local_vars_mut()[n] = val;
}

pub fn dstore(current_frame: &mut StackEntry, n: usize) {
    let jv = current_frame.pop();
    match jv {
        JavaValue::Double(_) => {}
        _ => {
            dbg!(&jv);
            panic!()
        }
    }
    current_frame.local_vars_mut()[n] = jv;
}

pub fn fstore(current_frame: &mut StackEntry, n: usize) {
    let jv = current_frame.pop();
    jv.unwrap_float();
    current_frame.local_vars_mut()[n] = jv;
}

pub fn castore(jvm: &JVMState, int_state: &mut InterpreterStateGuard) {
    let current_frame: &mut StackEntry = int_state.current_frame_mut();
    let val = current_frame.pop().unwrap_int();
    let index = current_frame.pop().unwrap_int();
    let arrar_ref_o = match current_frame.pop().unwrap_object() {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state);
        },
    };
    let array_ref = &mut arrar_ref_o.unwrap_array().mut_array();
    let char_ = val as u16;
    array_ref[index as usize] = JavaValue::Char(char_);
}

pub fn bastore(jvm: &JVMState, int_state: &mut InterpreterStateGuard) {
    let current_frame: &mut StackEntry = int_state.current_frame_mut();
    let val = current_frame.pop().unwrap_int() as jbyte;// int value is truncated
    let index = current_frame.pop().unwrap_int();
    let array_ref_o = match current_frame.pop().unwrap_object() {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state);
        },
    };
    assert!(array_ref_o.unwrap_array().elem_type == PTypeView::ByteType || array_ref_o.unwrap_array().elem_type == PTypeView::BooleanType);
    let array_ref = &mut array_ref_o.unwrap_array().mut_array();
    array_ref[index as usize] = JavaValue::Byte(val);
}


pub fn sastore(jvm: &JVMState, int_state: &mut InterpreterStateGuard) {
    let current_frame: &mut StackEntry = int_state.current_frame_mut();
    let val = current_frame.pop().unwrap_short();
    let index = current_frame.pop().unwrap_int();
    let array_ref_o = match current_frame.pop().unwrap_object() {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state);
        },
    };
    assert_eq!(array_ref_o.unwrap_array().elem_type, PTypeView::ShortType);
    let array_ref = &mut array_ref_o.unwrap_array().mut_array();
    array_ref[index as usize] = JavaValue::Short(val);
}


pub fn fastore(jvm: &JVMState, int_state: &mut InterpreterStateGuard) {
    let current_frame: &mut StackEntry = int_state.current_frame_mut();
    let val = current_frame.pop().unwrap_float();
    let index = current_frame.pop().unwrap_int();
    let array_ref_o = match current_frame.pop().unwrap_object() {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state);
        },
    };
    let array_ref = &mut array_ref_o.unwrap_array().mut_array();
    array_ref[index as usize] = JavaValue::Float(val);
}


pub fn dastore(jvm: &JVMState, int_state: &mut InterpreterStateGuard) {
    let current_frame: &mut StackEntry = int_state.current_frame_mut();
    let val = current_frame.pop().unwrap_double();
    let index = current_frame.pop().unwrap_int();
    let array_ref_o = match current_frame.pop().unwrap_object() {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state);
        }
    };
    let array_ref = &mut array_ref_o.unwrap_array().mut_array();
    array_ref[index as usize] = JavaValue::Double(val);
}


pub fn iastore(jvm: &JVMState, int_state: &mut InterpreterStateGuard) {
    let current_frame: &mut StackEntry = int_state.current_frame_mut();
    let val = current_frame.pop().unwrap_int();
    let index = current_frame.pop().unwrap_int();
    let arrar_ref_o = match current_frame.pop().unwrap_object() {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state);
        },
    };
    let array_ref = &mut arrar_ref_o.unwrap_array().mut_array();
    let int_ = val;
    array_ref[index as usize] = JavaValue::Int(int_);
}


pub fn aastore(jvm: &JVMState, int_state: &mut InterpreterStateGuard) {
    let current_frame: &mut StackEntry = int_state.current_frame_mut();
    let val = current_frame.pop();
    let index = current_frame.pop().unwrap_int();
    let arrary_ref_o = match current_frame.pop().unwrap_object() {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state);
        },
    };
    let array_ref = arrary_ref_o.unwrap_array().mut_array();
    match val {
        JavaValue::Object(_) => {}
        _ => panic!(),
    }
    array_ref[index as usize] = val;
}


pub fn istore(current_frame: &mut StackEntry, n: usize) {
    let object_ref = current_frame.pop();
    current_frame.local_vars_mut()[n] = JavaValue::Int(object_ref.unwrap_int());
}


pub fn lastore(jvm: &JVMState, int_state: &mut InterpreterStateGuard) {
    let current_frame: &mut StackEntry = int_state.current_frame_mut();
    let val = current_frame.pop().unwrap_long();
    let index = current_frame.pop().unwrap_int();
    let arrar_ref_o = match current_frame.pop().unwrap_object() {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state);
        },
    };
    let array_ref = &mut arrar_ref_o.unwrap_array().mut_array();
    let long = val;
    array_ref[index as usize] = JavaValue::Long(long);
}
