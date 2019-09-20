use interpreter::{InterpreterState};
use interpreter::interpreter_util::{push_long, pop_long};

pub fn do_ladd(state: &mut InterpreterState) -> () {
    let value2 = pop_long(state) as i64;
    let value1 = pop_long(state) as i64;
    push_long(value2 + value1, state)
}
