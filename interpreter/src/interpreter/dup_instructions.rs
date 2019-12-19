use crate::interpreter::{InterpreterState};
use crate::interpreter::interpreter_util::{push_long, push_int, pop_long, EXECUTION_ERROR, pop_int};

pub fn do_dup2_x1(state: &mut InterpreterState) -> () {
    let value1 = pop_long(state);
    let value2 = pop_long(state);
    push_long(value1, state);
    push_long(value2, state);
    push_long(value1, state);
    //todo us this the correct one?
}

pub fn do_dup2_x2(state: &mut InterpreterState) -> () {
    let value1 = pop_long(state);
    let value2 = state.operand_stack.pop().expect(EXECUTION_ERROR);
    push_long(value1, state);
    state.operand_stack.push(value2);
    push_long(value1, state);
}

pub fn do_dup2(state: &mut InterpreterState) -> () {
    let value1 = pop_long(state);
    push_long(value1, state);
    push_long(value1, state);
}

pub fn do_dup_x2(state: &mut InterpreterState) -> () {
    let value1 = pop_int(state);
    let value2 = pop_long(state);
    push_int(value1, state);
    push_long(value2, state);
    push_int(value1, state);
}

pub fn do_dup_x1(state: &mut InterpreterState) -> () {
    let value1 = pop_int(state);
    let value2 = pop_int(state);
    push_int(value1, state);
    push_int(value2, state);
    push_int(value1, state);
}

pub fn do_dup(state: &mut InterpreterState) -> () {
    let to_dup = pop_int(state);
    push_int(to_dup, state);
    push_int(to_dup, state);
}
