use rust_jvm_common::classfile::ConstantInfo;

pub struct InterpreterState {
    pub local_vars: Vec<u32>,
    pub operand_stack: Vec<u32>,
    pub pc: usize,
    //the pc_offset is set by every instruction. branch instructions and others may us it to jump
    pub pc_offset: isize,
    pub terminate: bool,
}

pub fn do_instruction(_code: &[u8], _state: &mut InterpreterState, _constant_pool: &Vec<ConstantInfo>) {
//    use interpreter::interpreter_util::*;
//    use interpreter::branch_instructions::*;
//    use interpreter::double_instructions::*;
//    use interpreter::dup_instructions::*;
//    use interpreter::float_instructions::*;
//    use interpreter::integer_instructions::*;
//    use interpreter::long_instructions::*;


    let _opcode = unimplemented!();//read_opcode(code[0]);
//    _state.pc_offset = 1;//offset the opcode which was just read
//    match opcode {
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
//        _ => {dbg!(opcode);unimplemented!()}
//    }
}

pub mod double_instructions;

pub mod integer_instructions;

pub mod long_instructions;

pub mod branch_instructions;
pub mod float_instructions;
pub mod dup_instructions;
#[macro_use]
pub mod interpreter_util;
