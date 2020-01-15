use crate::InterpreterState;
use rust_jvm_common::utils::code_attribute;
use classfile_parser::code::CodeParserContext;
use classfile_parser::code::parse_instruction;

pub fn run_function(state: &mut InterpreterState) {
    let current_frame = state.call_stack.last_mut().unwrap();
    let methods = &current_frame.class_pointer.classfile.methods;
    let method = &methods[current_frame.method_i as usize];
    let code = code_attribute(method).unwrap();

    assert!(!state.function_return);
    while !state.terminate && !state.function_return && !state.throw {
        let (instruct, instruction_size) = {
            let current = &code.code_raw[current_frame.pc..];
            let mut context = CodeParserContext { offset: 0, iter: current.iter() };
            (parse_instruction(&mut context).unwrap().clone(), context.offset)
        };
        current_frame.pc_offset = instruction_size as isize;
        match instruct {
            /*InstructionType::aaload => load_u64(_state),
            InstructionType::aastore => store_i64(_state),
            InstructionType::aconst_null => push_long(0, _state),
            InstructionType::aload => do_aload(code, _state),
            InstructionType::aload_0 => load_n_64(_state, 0),
            InstructionType::aload_1 => load_n_64(_state, 1),
            InstructionType::aload_2 => load_n_64(_state, 2),
            InstructionType::aload_3 => load_n_64(_state, 3),
            InstructionType::anewarray => do_anewarray(code, _state),
            InstructionType::areturn => {
                unimplemented!("Need to figure out function calls/returning from functions.");
            }
            InstructionType::arraylength => do_arraylength(_state),
            InstructionType::astore => do_astore(code, _state),
            InstructionType::astore_0 => store_n_64(_state, 0),
            InstructionType::astore_1 => store_n_64(_state, 1),
            InstructionType::astore_2 => store_n_64(_state, 2),
            InstructionType::astore_3 => store_n_64(_state, 3),
            InstructionType::athrow => { unimplemented!("Need to pass in  exception handlers somehow"); }
            InstructionType::baload => { load!(u8,_state); }
            InstructionType::bastore => { store!(u8,_state); }
            InstructionType::bipush => do_bipush(_state),
            InstructionType::caload => { load!(u16,_state); }
            InstructionType::castore => { store!(u16,_state); }
            InstructionType::checkcast => {
                unimplemented!("Need to increase pc by 3 and get constant pool involved");
            }
            InstructionType::d2f => do_d2f(_state),
            InstructionType::d2i => do_d2i(_state),
            InstructionType::d2l => do_d2l(_state),
            InstructionType::dadd => do_dadd(_state),
            InstructionType::daload => { load!(f64,_state); }
            InstructionType::dastore => { store!(f64,_state); }
            InstructionType::dcmpg => { unimplemented!("This one is kinda annoying to implement for now") }
            InstructionType::dcmpl => { unimplemented!("This one is also kinda annoying to implement for now") }
            InstructionType::dconst_0 => push_double(0.0, _state),
            InstructionType::dconst_1 => push_double(1.0, _state),
            InstructionType::ddiv => do_ddiv(_state),
            InstructionType::dload => do_dload(code, _state),
            InstructionType::dload_0 => load_n_64(_state, 0),
            InstructionType::dload_1 => load_n_64(_state, 1),
            InstructionType::dload_2 => load_n_64(_state, 2),
            InstructionType::dload_3 => load_n_64(_state, 3),
            InstructionType::dmul => do_dmul(_state),
            InstructionType::dneg => do_dneg(_state),
            InstructionType::drem => do_drem(_state),
            InstructionType::dreturn => {
                unimplemented!("need to figure out functions")
            }
            InstructionType::dstore => do_dstore(code, _state),
            InstructionType::dstore_0 => store_n_64(_state, 0),
            InstructionType::dstore_1 => store_n_64(_state, 1),
            InstructionType::dstore_2 => store_n_64(_state, 2),
            InstructionType::dstore_3 => store_n_64(_state, 3),
            InstructionType::dsub => do_dsub(_state),
            InstructionType::dup => do_dup(_state),
            InstructionType::dup_x1 => do_dup_x1(_state),
            InstructionType::dup_x2 => do_dup_x2(_state),
            InstructionType::dup2 => do_dup2(_state),
            InstructionType::dup2_x1 => do_dup2_x1(_state),
            InstructionType::dup2_x2 => do_dup2_x2(_state),
            InstructionType::f2d => do_f2d(_state),
            InstructionType::f2i => do_f2i(_state),
            InstructionType::f2l => do_f2l(_state),
            InstructionType::fadd => do_fadd(_state),
            InstructionType::faload => { load!(f32,_state); }
            InstructionType::fastore => { store!(f32,_state); }
            InstructionType::fcmpg => {
                unimplemented!("This one is kinda annoying to implement for now")
            }
            InstructionType::fcmpl => {
                unimplemented!("This one is kinda annoying to implement for now")
            }
            InstructionType::fconst_0 => push_float(0.0, _state),
            InstructionType::fconst_1 => push_float(1.0, _state),
            InstructionType::fconst_2 => push_float(2.0, _state),
            InstructionType::fdiv => do_fdiv(_state),
            InstructionType::fload => do_fload(code, _state),
            InstructionType::fload_0 => load_n_32(_state, 0),
            InstructionType::fload_1 => load_n_32(_state, 1),
            InstructionType::fload_2 => load_n_32(_state, 2),
            InstructionType::fload_3 => load_n_32(_state, 3),
            InstructionType::fmul => do_fmul(_state),
            InstructionType::fneg => do_fneg(_state),
            InstructionType::frem => {
                unimplemented!("not sure about differences with standard rem")
            }
            InstructionType::freturn => {
                unimplemented!("function return")
            }
            InstructionType::fstore => {
                let index = code[1];
                store_n_32(_state,index  as u64)
            }
            InstructionType::fstore_0 => store_n_32(_state, 0),
            InstructionType::fstore_1 => store_n_32(_state, 1),
            InstructionType::fstore_2 => store_n_32(_state, 2),
            InstructionType::fstore_3 => store_n_32(_state, 3),
            InstructionType::fsub => do_fsub(_state),
            InstructionType::getfield => {
                unimplemented!("constant pool")
            }
            InstructionType::getstatic => {
                unimplemented!("constant pool")
            }
            InstructionType::goto_ => do_goto(code),
            InstructionType::goto_w => do_goto_w(code),
            InstructionType::i2b => push_int(pop_byte(_state) as i8 as i32, _state),
            InstructionType::i2c => push_int(pop_short(_state) as u32 as i32, _state),
            InstructionType::i2d => push_double(pop_int(_state) as f64, _state),
            InstructionType::i2f => push_float(pop_int(_state) as f32, _state),
            InstructionType::i2l => push_long(pop_float(_state) as i32 as i64, _state),
            InstructionType::i2s => push_short(pop_int(_state) as u16 as i16, _state),//todo check
            InstructionType::iadd => do_iadd(_state),
            InstructionType::iaload => {
                load!(u32,_state);
            }
            InstructionType::iand => do_iand(_state),
            InstructionType::iastore => {
                store!(u32,_state);
            }
            InstructionType::iconst_m1 => push_int(-1, _state),
            InstructionType::iconst_0 => push_int(0, _state),
            InstructionType::iconst_1 => push_int(1, _state),
            InstructionType::iconst_2 => push_int(2, _state),
            InstructionType::iconst_3 => push_int(3, _state),
            InstructionType::iconst_4 => push_int(4, _state),
            InstructionType::iconst_5 => push_int(5, _state),
            InstructionType::idiv => do_idiv(_state),
            InstructionType::if_acmpeq => {
                unimplemented!("idk how to branch yet")
            }
            InstructionType::if_acmpne => {
                unimplemented!("idk how to branch yet")
            }
            InstructionType::if_icmpeq => {
                unimplemented!("idk how to branch yet")
            }
            InstructionType::if_icmpne => {
                unimplemented!("idk how to branch yet")
            }
            InstructionType::if_icmplt => {
                unimplemented!("idk how to branch yet")
            }
            InstructionType::if_icmpge => {
                unimplemented!("idk how to branch yet")
            }
            InstructionType::if_icmpgt => {
                unimplemented!("idk how to branch yet")
            }
            InstructionType::if_icmple => {
                unimplemented!("idk how to branch yet")
            }
            InstructionType::ifeq => {
                unimplemented!("idk how to branch yet")
            }
            InstructionType::ifne => {
                unimplemented!("idk how to branch yet")
            }
            InstructionType::iflt => {
                unimplemented!("idk how to branch yet")
            }
            InstructionType::ifge => {
                unimplemented!("idk how to branch yet")
            }
            InstructionType::ifgt => {
                unimplemented!("idk how to branch yet")
            }
            InstructionType::ifle => {
                unimplemented!("idk how to branch yet")
            }
            InstructionType::ifnonnull => {
                unimplemented!("idk how to branch yet")
            }
            InstructionType::ifnull => {
                unimplemented!("idk how to branch yet")
            }
            InstructionType::iinc => {
                _state.local_vars[code[1] as usize] += code[2] as u32;
                unimplemented!("Increase pc by 2")
            }
            InstructionType::iload => {
                load_n_32(_state,code[1] as u64);
                unimplemented!("Increase pc by 2")
            }
            InstructionType::iload_0 => load_n_32(_state, 0),
            InstructionType::iload_1 => load_n_32(_state, 1),
            InstructionType::iload_2 => load_n_32(_state, 2),
            InstructionType::iload_3 => load_n_32(_state, 3),
            InstructionType::imul => do_imul(_state),
            InstructionType::ineg => do_ineg(_state),
            InstructionType::instanceof => {
                unimplemented!("needs constant pool")
            }
            InstructionType::invokedynamic => {
                unimplemented!("needs constant pool")
            }
            InstructionType::invokeinterface => {
                unimplemented!("needs constant pool")
            }
            InstructionType::invokespecial => {
                unimplemented!("needs constant pool")
            }*/
//        InstructionTypeNum::invokestatic => {
//
//            unimplemented!("needs constant pool")
//        }
            /*InstructionType::invokevirtual => {
                unimplemented!("needs constant pool")
            }
            InstructionType::ior => do_ior(_state),
            InstructionType::irem => do_irem(_state),
            InstructionType::ireturn => {
                unimplemented!("functions need implementing")
            }
            InstructionType::ishl => do_ishl(_state),
            InstructionType::ishr => do_ishr(_state),
            InstructionType::istore => do_istore(code, _state),
            InstructionType::istore_0 => load_n_32(_state, 0),
            InstructionType::istore_1 => load_n_32(_state, 1),
            InstructionType::istore_2 => load_n_32(_state, 2),
            InstructionType::istore_3 => load_n_32(_state, 3),
            InstructionType::isub => do_isub(_state),
            InstructionType::iushr => do_iushr(_state),
            InstructionType::ixor => do_ixor(_state),
            InstructionType::jsr => {
                unimplemented!("functions")
            }
            InstructionType::jsr_w => {
                unimplemented!("functions")
            }
            InstructionType::l2d => push_double(pop_long(_state) as f64, _state),
            InstructionType::l2f => push_float(pop_long(_state) as f32, _state),
            InstructionType::l2i => push_int(pop_long(_state) as i32, _state),//todo check truncation
            InstructionType::ladd => do_ladd(_state),
            InstructionType::laload => {}
            InstructionType::land => {}
            InstructionType::lastore => {}
            InstructionType::lcmp => {}
            InstructionType::lconst_0 => {}
            InstructionType::lconst_1 => {}
            InstructionType::ldc => {}
            InstructionType::ldc_w => {}
            InstructionType::ldc2_w => {}
            InstructionType::ldiv => {}
            InstructionType::lload => {}
            InstructionType::lload_0 => {}
            InstructionType::lload_1 => {}
            InstructionType::lload_2 => {}
            InstructionType::lload_3 => {}
            InstructionType::lmul => {}
            InstructionType::lneg => {}
            InstructionType::lookupswitch => {}
            InstructionType::lor => {}
            InstructionType::lrem => {}
            InstructionType::lreturn => {}
            InstructionType::lshl => {}
            InstructionType::lshr => {}
            InstructionType::lstore => {}
            InstructionType::lstore_0 => {}
            InstructionType::lstore_1 => {}
            InstructionType::lstore_2 => {}
            InstructionType::lstore_3 => {}
            InstructionType::lsub => {}
            InstructionType::lushr => {}
            InstructionType::lxor => {}
            InstructionType::monitorenter => {}
            InstructionType::monitorexit => {}
            InstructionType::multianewarray => {}
            InstructionType::new => {}
            InstructionType::newarray => {}
            InstructionType::nop => {}
            InstructionType::pop => {}
            InstructionType::pop2 => {}
            InstructionType::putfield => {}
            InstructionType::putstatic => {}
            InstructionType::ret => {}
            InstructionType::return_ => {}
            InstructionType::saload => {}
            InstructionType::sastore => {}
            InstructionType::sipush => {}
            InstructionType::swap => {}
            InstructionType::tableswitch => {}
            InstructionType::wide => {}*/
            _ => {
                dbg!(instruct);
                unimplemented!()
            }
        }
        if current_frame.pc_offset > 0 {
            current_frame.pc += current_frame.pc_offset as usize;
        } else {
            current_frame.pc -= (-current_frame.pc_offset) as usize;//todo perhaps i don't have to do this bs if I use u64 instead of usize
        }
    }
}


//use ::std::mem::transmute;
//
//use crate::InterpreterState;
//
//pub const EXECUTION_ERROR: &str = "Fatal Error, when executing, this is a bug.";
//
//#[macro_export]
//macro_rules! null_pointer_check {
//($var_name:ident) => {
//    if $var_name == 0 {
//            unimplemented!("handle null pointers exceptions")
//        }
//};
//}
//#[macro_export]
//macro_rules! array_out_of_bounds_check {
//($index:expr,$array_length:ident) => {if ($index as u32) >= ($array_length as u32) {
//        unimplemented!("handle array out of bounds exceptions")
//    }};
//}
//#[macro_export]
//macro_rules! load {
//($type_:ident,$state:ident) => {
//    use ::interpreter::interpreter_util::{EXECUTION_ERROR, pop_long};
//    let index = $state.operand_stack.pop().expect(EXECUTION_ERROR);
//    let array_ref = pop_long($state);
//    use ::null_pointer_check;
//    null_pointer_check!(array_ref);
//    let array_elem:$type_ = unsafe {
//        let array_64: *mut u64 = ::std::mem::transmute(array_ref);
//        let array_length: u64 = *array_64.offset(-1);
//        let array_type:* mut $type_ = array_ref as * mut $type_;
//        use ::array_out_of_bounds_check;
//        array_out_of_bounds_check!(index as u64,array_length);
//        *(array_type.offset(index as isize)) as $type_
//    };
//    //todo this is more complicated in the u64 case
//    $state.operand_stack.push(array_elem as u32);
//};
//}
//
//#[macro_export]
//macro_rules! store {
//($type_:ident,$state:ident) => {
//    use ::interpreter::interpreter_util::{EXECUTION_ERROR, pop_long};
//    let value : $type_= $state.operand_stack.pop().expect(EXECUTION_ERROR) as $type_;
//    let index = $state.operand_stack.pop().expect(EXECUTION_ERROR);
//    let array_ref = pop_long($state);
//    use ::null_pointer_check;
//    null_pointer_check!(array_ref);
//    unsafe {
//        let array: *mut u64 = ::std::mem::transmute(array_ref);
//        let array_length: u64 = *array.offset(-1);
//        use ::array_out_of_bounds_check;
//        array_out_of_bounds_check!(index as u64,array_length);
//        let array_type : *mut $type_ = array_ref as *mut $type_;
//        *(array_type.offset(index as isize)) = value;
//    }
//};
//}
//
//pub fn store_i64(state: &mut InterpreterState){
//    let value  = pop_long(state);
//    let index = state.operand_stack.pop().expect(EXECUTION_ERROR);
//    let array_ref = pop_long(state);
//    null_pointer_check!(array_ref);
//    unsafe {
//        let array: *mut i64 = transmute(array_ref);
//        let array_length: i64 = *array.offset(-1);
//        array_out_of_bounds_check!(index as u64,array_length);
//        let array_type : *mut i64 = array_ref as *mut i64;
//        *(array_type.offset(index as isize)) = value;
//    }
//}
//
//pub fn load_u64(state: &mut InterpreterState){
//    let index = state.operand_stack.pop().expect(EXECUTION_ERROR);
//    let array_ref = pop_long(state);
//    null_pointer_check!(array_ref);
//    let array_elem: i64 = unsafe {
//        let array_64: *mut i64 = transmute(array_ref);
//        let array_length: i64 = *array_64.offset(-1);
//        let array_type:* mut i64 = array_ref as *mut i64;
//        array_out_of_bounds_check!(index as u64,array_length);
//        *(array_type.offset(index as isize))
//    };
//    push_long(array_elem,state);
//}
//
//pub fn pop_long(state: &mut InterpreterState) -> i64 {
//    let lower = state.operand_stack.pop().expect(EXECUTION_ERROR) as u64;
//    let upper = state.operand_stack.pop().expect(EXECUTION_ERROR) as u64;
//    return unsafe { transmute((upper << 32) | lower) }
//
//}
//
//pub fn push_long(to_push: i64, state: &mut InterpreterState) {
//    state.operand_stack.push( (to_push >> 32) as u32);
//    state.operand_stack.push( ((to_push << 32) >> 32) as u32);
//}
//
//pub fn push_byte(to_push: i8, state: &mut InterpreterState) {
//    state.operand_stack.push(to_push as u32)
//}
//
//pub fn pop_byte(state: &mut InterpreterState) -> i8 {
//    return state.operand_stack.pop().expect(EXECUTION_ERROR) as i8;
//}
//
//pub fn push_char(to_push: u16, state: &mut InterpreterState) {
//    state.operand_stack.push(to_push as u32)
//}
//
//pub fn pop_char(state: &mut InterpreterState) -> u16 {
//    return state.operand_stack.pop().expect(EXECUTION_ERROR) as u16;
//}
//
//pub fn push_short(to_push: i16, state: &mut InterpreterState) {
//    state.operand_stack.push(to_push as u32)
//}
//
//pub fn pop_short(state: &mut InterpreterState) -> i16 {
//    return state.operand_stack.pop().expect(EXECUTION_ERROR) as i16;
//}
//
//
//pub fn push_int(to_push: i32, state: &mut InterpreterState) {
//    state.operand_stack.push(unsafe { transmute(to_push) })
//}
//
//pub fn pop_int(state: &mut InterpreterState) -> i32 {
//    return unsafe { transmute(state.operand_stack.pop().expect(EXECUTION_ERROR)) };
//}
//
//
//pub fn push_float(to_push: f32, state: &mut InterpreterState) {
//    state.operand_stack.push(unsafe { ::std::mem::transmute(to_push) })
//}
//
//pub fn pop_float(state: &mut InterpreterState) -> f32 {
//    let value = state.operand_stack.pop().expect(EXECUTION_ERROR) as u32;
//    return unsafe { transmute(value) }
//}
//
//
//pub fn push_double(to_push: f64, state: &mut InterpreterState) {
//    push_long(unsafe { transmute(to_push) }, state)
//}
//
//pub fn pop_double(state: &mut InterpreterState) -> f64 {
//    let value = pop_long(state);
//    return unsafe {
//        ::std::mem::transmute(value)
//    }
//}
//
//pub fn store_n_32(state: &mut InterpreterState, n: u64) {
//    let reference = state.operand_stack.pop().expect(EXECUTION_ERROR);
//    state.local_vars[n as usize] = reference as u32;
//}
//
//
//pub fn store_n_64(state: &mut InterpreterState, n: u64) {
//    let reference = pop_long(state);
//    state.local_vars[n as usize] = reference as u32;
//    state.local_vars[(n + 1) as usize] = (reference >> 32) as u32;//todo is this really the correct order
//}
//
//pub fn load_n_32(state: &mut InterpreterState, n: u64) {
//    let reference = state.local_vars[n as usize];
//    state.operand_stack.push(reference as u32)
//}
//
//pub fn load_n_64(state: &mut InterpreterState, n: u64) {
//    let least_significant = state.local_vars[n as usize];
//    let most_significant = state.local_vars[(n + 1) as usize];
//    state.operand_stack.push(most_significant );
//    state.operand_stack.push(least_significant );
//}
//
//
/*
//pub(crate) fn do_bipush(state: &mut InterpreterState) -> () {
//    let byte = pop_int(state) as i8;
//    push_int(byte as i32, state);
//}
//
//pub(crate) fn do_astore(code: &[u8], state: &mut InterpreterState) -> ! {
//    let index = code[1];
//    store_n_32(state, index as u64);
//    unimplemented!("Need to increase pc by 2");
//}
//
//pub(crate) fn do_anewarray(code: &[u8], state: &mut InterpreterState) -> ! {
//    let indexbyte1 = code[1] as u16;
//    let indexbyte2 = code[2] as u16;
//    let _index = (indexbyte1 << 8) | indexbyte2;
//    let _count = state.operand_stack.pop().expect(EXECUTION_ERROR);
//    unimplemented!("Need to figure out how to get the constant pool in here.");
////    unimplemented!("Need to increase pc by 3");
//}
//
//pub(crate) fn do_aload(code: &[u8], state: &mut InterpreterState) -> ! {
//    let var_index = code[1];
//    load_n_64(state, var_index as u64);
//    unimplemented!("Need to increase pc by 2")
//}
//
//
//pub(crate) fn do_arraylength(state: &mut InterpreterState) -> () {
//    let array_ref = pop_long(state);
//    let length = unsafe {
//        let array: *mut i64 = transmute(array_ref);
//        *(array.offset(-1 as isize)) as i64
//    };
//    push_long(length,state)
//}
*/
//
//
//#[cfg(test)]
//pub mod tests{
//    use super::*;
//
//    #[test]
//    fn test_int_pop_push() {
//        let int_ = -654545864;
//        let state: &mut InterpreterState = &mut InterpreterState {
//            pc_offset: 0,pc:0,local_vars:Vec::new(),operand_stack:Vec::new(),
//            terminate: false
//        };
//        push_int(int_,state);
//        assert_eq!(int_,pop_int(state));
//    }
//
//    #[test]
//    fn test_long_pop_push() {
//        let long_ = -654545864*435657687;
//        let state: &mut InterpreterState = &mut InterpreterState {
//            pc_offset: 0,pc:0,local_vars:Vec::new(),operand_stack:Vec::new(),
//            terminate: false
//        };
//        push_long(long_,state);
//        assert_eq!(long_,pop_long(state));
//    }
//
//    #[test]
//    fn test_char_pop_push() {
//        let char_ = 'g';
//        let state: &mut InterpreterState = &mut InterpreterState {
//            pc_offset: 0,pc:0,local_vars:Vec::new(),operand_stack:Vec::new(),
//            terminate: false
//        };
//        push_char(char_ as u16, state);
//        assert_eq!(char_ as u16, pop_char(state));
//    }
//
//    #[test]
//    fn test_double_pop_push() {
//        let double_ = 0.4546545613512652;
//        let state: &mut InterpreterState = &mut InterpreterState {
//            pc_offset: 0,pc:0,local_vars:Vec::new(),operand_stack:Vec::new(),
//            terminate: false
//        };
//        push_double(double_,state);
//        assert_eq!(double_,pop_double(state));
//    }
//
//
//    #[test]
//    fn test_float_pop_push() {
//        let float_ = -56.045f32;
//        let state: &mut InterpreterState = &mut InterpreterState {
//            pc_offset: 0,pc:0,local_vars:Vec::new(),operand_stack:Vec::new(),
//            terminate: false
//        };
//        push_float(float_,state);
//        assert_eq!(float_,pop_float(state));
//    }
//
//    #[test]
//    fn test_byte_pop_push() {
//        let byte_  = -120i8;
//        let state: &mut InterpreterState = &mut InterpreterState {
//            pc_offset: 0,pc:0,local_vars:Vec::new(),operand_stack:Vec::new(),
//            terminate: false
//        };
//        push_byte(byte_, state);//todo need to pop push i8
//        assert_eq!(byte_, pop_byte(state));
//    }
//
//}