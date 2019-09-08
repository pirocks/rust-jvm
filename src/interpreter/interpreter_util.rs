use ::std::mem::transmute;

use interpreter::InterpreterState;

pub const EXECUTION_ERROR: &str = "Fatal Error, when executing, this is a bug.";

#[macro_export]
macro_rules! null_pointer_check {
($var_name:ident) => {
    if $var_name == 0 {
            unimplemented!("handle null pointers exceptions")
        }
};
}
#[macro_export]
macro_rules! array_out_of_bounds_check {
($index:expr,$array_length:ident) => {if ($index as u32) >= ($array_length as u32) {
        unimplemented!("handle array out of bounds exceptions")
    }};
}
#[macro_export]
macro_rules! load {
($type_:ident,$state:ident) => {
    use ::interpreter::interpreter_util::{EXECUTION_ERROR, pop_long};
    let index = $state.operand_stack.pop().expect(EXECUTION_ERROR);
    let array_ref = pop_long($state);
    use ::null_pointer_check;
    null_pointer_check!(array_ref);
    let array_elem:$type_ = unsafe {
        let array_64: *mut u64 = ::std::mem::transmute(array_ref);
        let array_length: u64 = *array_64.offset(-1);
        let array_type:* mut $type_ = array_ref as * mut $type_;
        use ::array_out_of_bounds_check;
        array_out_of_bounds_check!(index as u64,array_length);
        *(array_type.offset(index as isize)) as $type_
    };
    //todo this is more complicated in the u64 case
    $state.operand_stack.push(array_elem as u32);
};
}

#[macro_export]
macro_rules! store {
($type_:ident,$state:ident) => {
    use ::interpreter::interpreter_util::{EXECUTION_ERROR, pop_long};
    let value : $type_= $state.operand_stack.pop().expect(EXECUTION_ERROR) as $type_;
    let index = $state.operand_stack.pop().expect(EXECUTION_ERROR);
    let array_ref = pop_long($state);
    use ::null_pointer_check;
    null_pointer_check!(array_ref);
    unsafe {
        let array: *mut u64 = ::std::mem::transmute(array_ref);
        let array_length: u64 = *array.offset(-1);
        use ::array_out_of_bounds_check;
        array_out_of_bounds_check!(index as u64,array_length);
        let array_type : *mut $type_ = array_ref as *mut $type_;
        *(array_type.offset(index as isize)) = value;
    }
};
}

pub fn store_i64(state: &mut InterpreterState){
    let value  = pop_long(state);
    let index = state.operand_stack.pop().expect(EXECUTION_ERROR);
    let array_ref = pop_long(state);
    null_pointer_check!(array_ref);
    unsafe {
        let array: *mut i64 = transmute(array_ref);
        let array_length: i64 = *array.offset(-1);
        array_out_of_bounds_check!(index as u64,array_length);
        let array_type : *mut i64 = array_ref as *mut i64;
        *(array_type.offset(index as isize)) = value;
    }
}

pub fn load_u64(state: &mut InterpreterState){
    let index = state.operand_stack.pop().expect(EXECUTION_ERROR);
    let array_ref = pop_long(state);
    null_pointer_check!(array_ref);
    let array_elem: i64 = unsafe {
        let array_64: *mut i64 = transmute(array_ref);
        let array_length: i64 = *array_64.offset(-1);
        let array_type:* mut i64 = array_ref as *mut i64;
        array_out_of_bounds_check!(index as u64,array_length);
        *(array_type.offset(index as isize))
    };
    push_long(array_elem,state);
}

pub fn pop_long(state: &mut InterpreterState) -> i64 {
    let lower = state.operand_stack.pop().expect(EXECUTION_ERROR) as u64;
    let upper = state.operand_stack.pop().expect(EXECUTION_ERROR) as u64;
    return unsafe { transmute((upper << 32) | lower) }

}

pub fn push_long(to_push: i64, state: &mut InterpreterState) {
    state.operand_stack.push( (to_push >> 32) as u32);
    state.operand_stack.push( ((to_push << 32) >> 32) as u32);
}

pub fn push_byte(to_push: i8, state: &mut InterpreterState) {
    state.operand_stack.push(to_push as u32)
}

pub fn pop_byte(state: &mut InterpreterState) -> i8 {
    return state.operand_stack.pop().expect(EXECUTION_ERROR) as i8;
}

pub fn push_char(to_push: u16, state: &mut InterpreterState) {
    state.operand_stack.push(to_push as u32)
}

pub fn pop_char(state: &mut InterpreterState) -> u16 {
    return state.operand_stack.pop().expect(EXECUTION_ERROR) as u16;
}

pub fn push_short(to_push: i16, state: &mut InterpreterState) {
    state.operand_stack.push(to_push as u32)
}

pub fn pop_short(state: &mut InterpreterState) -> i16 {
    return state.operand_stack.pop().expect(EXECUTION_ERROR) as i16;
}


pub fn push_int(to_push: i32, state: &mut InterpreterState) {
    state.operand_stack.push(unsafe { transmute(to_push) })
}

pub fn pop_int(state: &mut InterpreterState) -> i32 {
    return unsafe { transmute(state.operand_stack.pop().expect(EXECUTION_ERROR)) };
}


pub fn push_float(to_push: f32, state: &mut InterpreterState) {
    state.operand_stack.push(unsafe { ::std::mem::transmute(to_push) })
}

pub fn pop_float(state: &mut InterpreterState) -> f32 {
    let value = state.operand_stack.pop().expect(EXECUTION_ERROR) as u32;
    return unsafe { transmute(value) }
}


pub fn push_double(to_push: f64, state: &mut InterpreterState) {
    push_long(unsafe { transmute(to_push) }, state)
}

pub fn pop_double(state: &mut InterpreterState) -> f64 {
    let value = pop_long(state);
    return unsafe {
        ::std::mem::transmute(value)
    }
}

pub fn store_n_32(state: &mut InterpreterState, n: u64) {
    let reference = state.operand_stack.pop().expect(EXECUTION_ERROR);
    state.local_vars[n as usize] = reference as u32;
}


pub fn store_n_64(state: &mut InterpreterState, n: u64) {
    let reference = pop_long(state);
    state.local_vars[n as usize] = reference as u32;
    state.local_vars[(n + 1) as usize] = (reference >> 32) as u32;//todo is this really the correct order
}

pub fn load_n_32(state: &mut InterpreterState, n: u64) {
    let reference = state.local_vars[n as usize];
    state.operand_stack.push(reference as u32)
}

pub fn load_n_64(state: &mut InterpreterState, n: u64) {
    let least_significant = state.local_vars[n as usize];
    let most_significant = state.local_vars[(n + 1) as usize];
    state.operand_stack.push(most_significant );
    state.operand_stack.push(least_significant );
}

#[cfg(test)]
pub mod tests{
    use super::*;

    #[test]
    fn test_int_pop_push() {
        let int_ = -654545864;
        let state: &mut InterpreterState = &mut InterpreterState {
            pc_offset: 0,pc:0,local_vars:Vec::new(),operand_stack:Vec::new()
        };
        push_int(int_,state);
        assert_eq!(int_,pop_int(state));
    }

    #[test]
    fn test_long_pop_push() {
        let long_ = -654545864*435657687;
        let state: &mut InterpreterState = &mut InterpreterState {
            pc_offset: 0,pc:0,local_vars:Vec::new(),operand_stack:Vec::new()
        };
        push_long(long_,state);
        assert_eq!(long_,pop_long(state));
    }

    #[test]
    fn test_char_pop_push() {
        let char_ = 'g';
        let state: &mut InterpreterState = &mut InterpreterState {
            pc_offset: 0,pc:0,local_vars:Vec::new(),operand_stack:Vec::new()
        };
        push_char(char_ as u16, state);
        assert_eq!(char_ as u16, pop_char(state));
    }

    #[test]
    fn test_double_pop_push() {
        let double_ = 0.4546545613512652;
        let state: &mut InterpreterState = &mut InterpreterState {
            pc_offset: 0,pc:0,local_vars:Vec::new(),operand_stack:Vec::new()
        };
        push_double(double_,state);
        assert_eq!(double_,pop_double(state));
    }


    #[test]
    fn test_float_pop_push() {
        let float_ = -56.045f32;
        let state: &mut InterpreterState = &mut InterpreterState {
            pc_offset: 0,pc:0,local_vars:Vec::new(),operand_stack:Vec::new()
        };
        push_float(float_,state);
        assert_eq!(float_,pop_float(state));
    }

    #[test]
    fn test_byte_pop_push() {
        let byte_  = -120i8;
        let state: &mut InterpreterState = &mut InterpreterState {
            pc_offset: 0,pc:0,local_vars:Vec::new(),operand_stack:Vec::new()
        };
        push_byte(byte_, state);//todo need to pop push i8
        assert_eq!(byte_, pop_byte(state));
    }

}