use crate::{ InterpreterState};
use crate::interpreter_util::{push_float, load_n_32, push_long, push_double, pop_float};

pub fn do_fsub(state: &mut InterpreterState) -> () {
    let value2 = pop_float(state);
    let value1 = pop_float(state);
    push_float(value1 - value2, state)
}

pub fn do_fneg(state: &mut InterpreterState) -> () {
    push_float(-pop_float(state), state)
}

pub fn do_fmul(state: &mut InterpreterState) -> () {
    let value2 = pop_float(state);
    let value1 = pop_float(state);
    push_float(value1 + value2, state)
}

pub fn do_fload(code: &[u8], state: &mut InterpreterState) -> ! {
    let index = code[1];
    load_n_32(state, index as u64);
    unimplemented!("need pc by 2")
}

pub fn do_fdiv(state: &mut InterpreterState) -> () {
    let value2 = pop_float(state);
    let value1 = pop_float(state);
    push_float(value1 / value2, state)
}

pub fn do_fadd(state: &mut InterpreterState) -> () {
    let a = pop_float(state);
    let b = pop_float(state);
    push_float(a + b, state)
}

pub fn do_f2l(state: &mut InterpreterState) -> () {
    let float = pop_float(state);
    push_long(float as i64, state);
}

pub fn do_f2i(state: &mut InterpreterState) -> () {
    let float = pop_float(state);
    state.operand_stack.push(float as u32);
}

pub fn do_f2d(state: &mut InterpreterState) -> () {
    let float = pop_float(state);
    push_double(float as f64, state);
}
