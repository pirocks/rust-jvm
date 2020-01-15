use crate::InterpreterState;
use rust_jvm_common::utils::code_attribute;
use classfile_parser::code::CodeParserContext;
use classfile_parser::code::parse_instruction;
use rust_jvm_common::classfile::InstructionInfo;
use verification::verifier::codecorrectness::Environment;
use verification::verifier::ClassWithLoaderMethod;
use rust_jvm_common::unified_types::VerificationType;
use verification::verifier::instructions::special::extract_field_descriptor;
use crate::CallStackEntry;
use std::sync::Arc;
use crate::runtime_class::RuntimeClass;
use crate::runtime_class::prepare_class;
use crate::runtime_class::initialize_class;

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
            InstructionInfo::aaload => unimplemented!(),
            InstructionInfo::aastore => unimplemented!(),
            InstructionInfo::aconst_null => unimplemented!(),
            InstructionInfo::aload(_) => unimplemented!(),
            InstructionInfo::aload_0 => unimplemented!(),
            InstructionInfo::aload_1 => unimplemented!(),
            InstructionInfo::aload_2 => unimplemented!(),
            InstructionInfo::aload_3 => unimplemented!(),
            InstructionInfo::anewarray(_) => unimplemented!(),
            InstructionInfo::areturn => unimplemented!(),
            InstructionInfo::arraylength => unimplemented!(),
            InstructionInfo::astore(_) => unimplemented!(),
            InstructionInfo::astore_0 => unimplemented!(),
            InstructionInfo::astore_1 => unimplemented!(),
            InstructionInfo::astore_2 => unimplemented!(),
            InstructionInfo::astore_3 => unimplemented!(),
            InstructionInfo::athrow => unimplemented!(),
            InstructionInfo::baload => unimplemented!(),
            InstructionInfo::bastore => unimplemented!(),
            InstructionInfo::bipush(_) => unimplemented!(),
            InstructionInfo::caload => unimplemented!(),
            InstructionInfo::castore => unimplemented!(),
            InstructionInfo::checkcast(_) => unimplemented!(),
            InstructionInfo::d2f => unimplemented!(),
            InstructionInfo::d2i => unimplemented!(),
            InstructionInfo::d2l => unimplemented!(),
            InstructionInfo::dadd => unimplemented!(),
            InstructionInfo::daload => unimplemented!(),
            InstructionInfo::dastore => unimplemented!(),
            InstructionInfo::dcmpg => unimplemented!(),
            InstructionInfo::dcmpl => unimplemented!(),
            InstructionInfo::dconst_0 => unimplemented!(),
            InstructionInfo::dconst_1 => unimplemented!(),
            InstructionInfo::ddiv => unimplemented!(),
            InstructionInfo::dload(_) => unimplemented!(),
            InstructionInfo::dload_0 => unimplemented!(),
            InstructionInfo::dload_1 => unimplemented!(),
            InstructionInfo::dload_2 => unimplemented!(),
            InstructionInfo::dload_3 => unimplemented!(),
            InstructionInfo::dmul => unimplemented!(),
            InstructionInfo::dneg => unimplemented!(),
            InstructionInfo::drem => unimplemented!(),
            InstructionInfo::dreturn => unimplemented!(),
            InstructionInfo::dstore(_) => unimplemented!(),
            InstructionInfo::dstore_0 => unimplemented!(),
            InstructionInfo::dstore_1 => unimplemented!(),
            InstructionInfo::dstore_2 => unimplemented!(),
            InstructionInfo::dstore_3 => unimplemented!(),
            InstructionInfo::dsub => unimplemented!(),
            InstructionInfo::dup => unimplemented!(),
            InstructionInfo::dup_x1 => unimplemented!(),
            InstructionInfo::dup_x2 => unimplemented!(),
            InstructionInfo::dup2 => unimplemented!(),
            InstructionInfo::dup2_x1 => unimplemented!(),
            InstructionInfo::dup2_x2 => unimplemented!(),
            InstructionInfo::f2d => unimplemented!(),
            InstructionInfo::f2i => unimplemented!(),
            InstructionInfo::f2l => unimplemented!(),
            InstructionInfo::fadd => unimplemented!(),
            InstructionInfo::faload => unimplemented!(),
            InstructionInfo::fastore => unimplemented!(),
            InstructionInfo::fcmpg => unimplemented!(),
            InstructionInfo::fcmpl => unimplemented!(),
            InstructionInfo::fconst_0 => unimplemented!(),
            InstructionInfo::fconst_1 => unimplemented!(),
            InstructionInfo::fconst_2 => unimplemented!(),
            InstructionInfo::fdiv => unimplemented!(),
            InstructionInfo::fload(_) => unimplemented!(),
            InstructionInfo::fload_0 => unimplemented!(),
            InstructionInfo::fload_1 => unimplemented!(),
            InstructionInfo::fload_2 => unimplemented!(),
            InstructionInfo::fload_3 => unimplemented!(),
            InstructionInfo::fmul => unimplemented!(),
            InstructionInfo::fneg => unimplemented!(),
            InstructionInfo::frem => unimplemented!(),
            InstructionInfo::freturn => unimplemented!(),
            InstructionInfo::fstore(_) => unimplemented!(),
            InstructionInfo::fstore_0 => unimplemented!(),
            InstructionInfo::fstore_1 => unimplemented!(),
            InstructionInfo::fstore_2 => unimplemented!(),
            InstructionInfo::fstore_3 => unimplemented!(),
            InstructionInfo::fsub => unimplemented!(),
            InstructionInfo::getfield(_) => unimplemented!(),
            InstructionInfo::getstatic(cp) => {
                //todo make sure class pointer is updated correctly
                let classfile = &current_frame.class_pointer.classfile;
                let loader_arc = &current_frame.class_pointer.loader;
                let (field_class_name, field_name, field_descriptor) = extract_field_descriptor(cp, classfile.clone(), loader_arc.clone());
                let bl = state.bootstrap_loader.clone();
                let target_classfile = loader_arc.clone().load_class(loader_arc, &field_class_name, bl).unwrap();
                let inited_target =  initialize_class(prepare_class(target_classfile,loader_arc),state);//todo need to store initted class in state.




                run_function(state);
            },
            InstructionInfo::goto_(_) => unimplemented!(),
            InstructionInfo::goto_w(_) => unimplemented!(),
            InstructionInfo::i2b => unimplemented!(),
            InstructionInfo::i2c => unimplemented!(),
            InstructionInfo::i2d => unimplemented!(),
            InstructionInfo::i2f => unimplemented!(),
            InstructionInfo::i2l => unimplemented!(),
            InstructionInfo::i2s => unimplemented!(),
            InstructionInfo::iadd => unimplemented!(),
            InstructionInfo::iaload => unimplemented!(),
            InstructionInfo::iand => unimplemented!(),
            InstructionInfo::iastore => unimplemented!(),
            InstructionInfo::iconst_m1 => unimplemented!(),
            InstructionInfo::iconst_0 => unimplemented!(),
            InstructionInfo::iconst_1 => unimplemented!(),
            InstructionInfo::iconst_2 => unimplemented!(),
            InstructionInfo::iconst_3 => unimplemented!(),
            InstructionInfo::iconst_4 => unimplemented!(),
            InstructionInfo::iconst_5 => unimplemented!(),
            InstructionInfo::idiv => unimplemented!(),
            InstructionInfo::if_acmpeq(_) => unimplemented!(),
            InstructionInfo::if_acmpne(_) => unimplemented!(),
            InstructionInfo::if_icmpeq(_) => unimplemented!(),
            InstructionInfo::if_icmpne(_) => unimplemented!(),
            InstructionInfo::if_icmplt(_) => unimplemented!(),
            InstructionInfo::if_icmpge(_) => unimplemented!(),
            InstructionInfo::if_icmpgt(_) => unimplemented!(),
            InstructionInfo::if_icmple(_) => unimplemented!(),
            InstructionInfo::ifeq(_) => unimplemented!(),
            InstructionInfo::ifne(_) => unimplemented!(),
            InstructionInfo::iflt(_) => unimplemented!(),
            InstructionInfo::ifge(_) => unimplemented!(),
            InstructionInfo::ifgt(_) => unimplemented!(),
            InstructionInfo::ifle(_) => unimplemented!(),
            InstructionInfo::ifnonnull(_) => unimplemented!(),
            InstructionInfo::ifnull(_) => unimplemented!(),
            InstructionInfo::iinc(_) => unimplemented!(),
            InstructionInfo::iload(_) => unimplemented!(),
            InstructionInfo::iload_0 => unimplemented!(),
            InstructionInfo::iload_1 => unimplemented!(),
            InstructionInfo::iload_2 => unimplemented!(),
            InstructionInfo::iload_3 => unimplemented!(),
            InstructionInfo::imul => unimplemented!(),
            InstructionInfo::ineg => unimplemented!(),
            InstructionInfo::instanceof(_) => unimplemented!(),
            InstructionInfo::invokedynamic(_) => unimplemented!(),
            InstructionInfo::invokeinterface(_) => unimplemented!(),
            InstructionInfo::invokespecial(_) => unimplemented!(),
            InstructionInfo::invokestatic(_) => unimplemented!(),
            InstructionInfo::invokevirtual(_) => unimplemented!(),
            InstructionInfo::ior => unimplemented!(),
            InstructionInfo::irem => unimplemented!(),
            InstructionInfo::ireturn => unimplemented!(),
            InstructionInfo::ishl => unimplemented!(),
            InstructionInfo::ishr => unimplemented!(),
            InstructionInfo::istore(_) => unimplemented!(),
            InstructionInfo::istore_0 => unimplemented!(),
            InstructionInfo::istore_1 => unimplemented!(),
            InstructionInfo::istore_2 => unimplemented!(),
            InstructionInfo::istore_3 => unimplemented!(),
            InstructionInfo::isub => unimplemented!(),
            InstructionInfo::iushr => unimplemented!(),
            InstructionInfo::ixor => unimplemented!(),
            InstructionInfo::jsr(_) => unimplemented!(),
            InstructionInfo::jsr_w(_) => unimplemented!(),
            InstructionInfo::l2d => unimplemented!(),
            InstructionInfo::l2f => unimplemented!(),
            InstructionInfo::l2i => unimplemented!(),
            InstructionInfo::ladd => unimplemented!(),
            InstructionInfo::laload => unimplemented!(),
            InstructionInfo::land => unimplemented!(),
            InstructionInfo::lastore => unimplemented!(),
            InstructionInfo::lcmp => unimplemented!(),
            InstructionInfo::lconst_0 => unimplemented!(),
            InstructionInfo::lconst_1 => unimplemented!(),
            InstructionInfo::ldc(_) => unimplemented!(),
            InstructionInfo::ldc_w(_) => unimplemented!(),
            InstructionInfo::ldc2_w(_) => unimplemented!(),
            InstructionInfo::ldiv => unimplemented!(),
            InstructionInfo::lload(_) => unimplemented!(),
            InstructionInfo::lload_0 => unimplemented!(),
            InstructionInfo::lload_1 => unimplemented!(),
            InstructionInfo::lload_2 => unimplemented!(),
            InstructionInfo::lload_3 => unimplemented!(),
            InstructionInfo::lmul => unimplemented!(),
            InstructionInfo::lneg => unimplemented!(),
            InstructionInfo::lookupswitch(_) => unimplemented!(),
            InstructionInfo::lor => unimplemented!(),
            InstructionInfo::lrem => unimplemented!(),
            InstructionInfo::lreturn => unimplemented!(),
            InstructionInfo::lshl => unimplemented!(),
            InstructionInfo::lshr => unimplemented!(),
            InstructionInfo::lstore(_) => unimplemented!(),
            InstructionInfo::lstore_0 => unimplemented!(),
            InstructionInfo::lstore_1 => unimplemented!(),
            InstructionInfo::lstore_2 => unimplemented!(),
            InstructionInfo::lstore_3 => unimplemented!(),
            InstructionInfo::lsub => unimplemented!(),
            InstructionInfo::lushr => unimplemented!(),
            InstructionInfo::lxor => unimplemented!(),
            InstructionInfo::monitorenter => unimplemented!(),
            InstructionInfo::monitorexit => unimplemented!(),
            InstructionInfo::multianewarray(_) => unimplemented!(),
            InstructionInfo::new(_) => unimplemented!(),
            InstructionInfo::newarray(_) => unimplemented!(),
            InstructionInfo::nop => unimplemented!(),
            InstructionInfo::pop => unimplemented!(),
            InstructionInfo::pop2 => unimplemented!(),
            InstructionInfo::putfield(_) => unimplemented!(),
            InstructionInfo::putstatic(_) => unimplemented!(),
            InstructionInfo::ret(_) => unimplemented!(),
            InstructionInfo::return_ => unimplemented!(),
            InstructionInfo::saload => unimplemented!(),
            InstructionInfo::sastore => unimplemented!(),
            InstructionInfo::sipush(_) => unimplemented!(),
            InstructionInfo::swap => unimplemented!(),
            InstructionInfo::tableswitch(_) => unimplemented!(),
            InstructionInfo::wide(_) => unimplemented!(),
            InstructionInfo::EndOfCode => unimplemented!(),
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