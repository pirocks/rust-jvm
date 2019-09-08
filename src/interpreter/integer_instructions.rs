use interpreter::{InterpreterState};
use interpreter::interpreter_util::{push_int, pop_int, load_n_32};

pub fn do_ixor(state: &mut InterpreterState) -> () {
    let value2 = pop_int(state);
    let value1 = pop_int(state);
    push_int(value1 ^ value2, state);
}

pub fn do_iushr(state: &mut InterpreterState) -> () {
    let value2 = pop_int(state);
    let value1 = pop_int(state);
    let shift_amount = ((value2 << (32 - 5)) >> (32 - 5)) as i32;
    push_int(value1 >> shift_amount, state)
}

pub fn do_isub(state: &mut InterpreterState) -> () {
    let value2 = pop_int(state) as i32;
    let value1 = pop_int(state) as i32;
    push_int(value1 - value2, state)
}

pub fn do_istore(code: &[u8], state: &mut InterpreterState) -> () {
    load_n_32(state, code[1] as u64);
    state.pc_offset += 1; //offset code[]
}

pub fn do_ishr(state: &mut InterpreterState) -> () {
    let value2 = pop_int(state);
    let value1 = pop_int(state);
    let shift_amount = ((value2 << (32 - 5)) >> (32 - 5));
    push_int(value1 >> shift_amount, state)
}

pub fn do_ishl(state: &mut InterpreterState) -> () {
    let value2 = pop_int(state);
    let value1 = pop_int(state);
    let shift_amount = ((value2 << (32 - 5)) >> (32 - 5));
    push_int(value1 << shift_amount, state)
}

pub fn do_irem(state: &mut InterpreterState) -> () {
    let value2 = pop_int(state);
    let value1 = pop_int(state);
    push_int(value1 % value2, state);
}

pub fn do_ior(state: &mut InterpreterState) -> () {
    let value2 = pop_int(state);
    let value1 = pop_int(state);
    push_int(value1 | value2, state);
}

pub fn do_ineg(state: &mut InterpreterState) -> () {
    let value = pop_int(state) ;
    push_int(-value, state)
}

pub fn do_imul(state: &mut InterpreterState) -> () {
    let value2 = pop_int(state) ;
    let value1 = pop_int(state) ;
    push_int(value1 * value2, state);
}

pub fn do_idiv(state: &mut InterpreterState) -> () {
    let value2 = pop_int(state) ;
    let value1 = pop_int(state) ;
    push_int(value1 / value2, state);
}

pub fn do_iand(state: &mut InterpreterState) -> () {
    let value2 = pop_int(state);
    let value1 = pop_int(state);
    push_int(value1 & value2, state);
}

pub fn do_iadd(state: &mut InterpreterState) -> () {
    let a = pop_int(state);
    let b = pop_int(state);
    push_int(a + b, state);
}
