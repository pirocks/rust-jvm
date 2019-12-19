use crate::interpreter::InterpreterState;
use crate::interpreter::interpreter_util::{load_n_64, push_double, store_n_64, pop_double, push_long};

pub fn do_dsub(state: &mut InterpreterState) -> () {
    let value2 = pop_double(state);
    let value1 = pop_double(state);
    push_double(value1 - value2, state);
}

pub fn do_dstore(code: &[u8], state: &mut InterpreterState) -> ! {
    let var_index = code[1];
    store_n_64(state, var_index as u64);
    unimplemented!("Need to increase pc by 2")
}

pub fn do_drem(state: &mut InterpreterState) -> () {
    let a = pop_double(state);
    let b = pop_double(state);
    push_double(a % b, state);//todo not sure if that is correct since rem is non-standard in java
}

pub fn do_dneg(state: &mut InterpreterState) -> () {
    let a = pop_double(state);
    push_double(-1.0 * a, state);
}

pub fn do_dmul(state: &mut InterpreterState) -> () {
    let a = pop_double(state);
    let b = pop_double(state);
    push_double(a * b, state);
}

pub fn do_dload(code: &[u8], state: &mut InterpreterState) -> ! {
    let var_index = code[1];
    load_n_64(state, var_index as u64);
    unimplemented!("Need to increase pc by 2")
}

pub fn do_ddiv(state: &mut InterpreterState) -> () {
    let bottom = pop_double(state);
    let top = pop_double(state);
    push_double(bottom / top, state)
}

pub fn do_dadd(state: &mut InterpreterState) -> () {
    let a = pop_double(state);
    let b = pop_double(state);
    let sum = a + b;
    push_double(sum, state)
}

pub fn do_d2l(state: &mut InterpreterState) -> () {
    let double = pop_double(state);
    push_long(double as i64, state)
}

pub fn do_d2i(state: &mut InterpreterState) -> () {
    let double = pop_double(state);
    state.operand_stack.push(double as u32)
}

/*
pub(crate) fn do_d2f(state: &mut InterpreterState) -> () {
    let double = pop_double(state);
    let converted_to_float = double as f32;
    push_float(converted_to_float, state);
}
*/
