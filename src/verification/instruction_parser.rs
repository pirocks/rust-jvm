use std::io::{Error, Write};

use classfile::attribute_infos::Code;
use interpreter::{InstructionType, read_opcode};


fn instruction_to_string(i: usize, code: &Vec<u8>) -> (&str, u64) {
    match read_opcode(code[i]) {
        InstructionType::aaload => { ("aaload", 0) },
        InstructionType::aastore => { ("aastore", 0) },
        InstructionType::aconst_null => { ("aconst_null", 0) },
        InstructionType::aload => {
            ("aload(Index)", unimplemented!())
        },
        InstructionType::aload_0 => { ("aload_0", 0) },
        InstructionType::aload_1 => { ("aload_1", 0) },
        InstructionType::aload_2 => { ("aload_2", 0) },
        InstructionType::aload_3 => { ("aload_3", 0) },
        InstructionType::anewarray => {
            ("anewarray(CP)", unimplemented!())
        },
        InstructionType::areturn => { ("areturn", 0) },
        InstructionType::arraylength => { ("arraylength", 0) },
        InstructionType::astore => {
            ("astore(Index)", unimplemented!())
        },
        InstructionType::astore_0 => { ("astore_0", 0) },
        InstructionType::astore_1 => { ("astore_1", 0) },
        InstructionType::astore_2 => { ("astore_2", 0) },
        InstructionType::astore_3 => { ("astore_3", 0) },
        InstructionType::athrow => { ("athrow", 0) },
        InstructionType::baload => { ("baload", 0) },
        InstructionType::bastore => { ("bastore", 0) },
        InstructionType::bipush => {
            ("bipush(Value)", unimplemented!())
        },
        InstructionType::caload => { ("caload", 0) },
        InstructionType::castore => { ("castore", 0) },
        InstructionType::checkcast => {
            ("checkcast(CP)", unimplemented!())
        },
        InstructionType::d2f => { ("d2f", 0) },
        InstructionType::d2i => { ("d2i", 0) },
        InstructionType::d2l => { ("d2l", 0) },
        InstructionType::dadd => { ("dadd", 0) },
        InstructionType::daload => { ("daload", 0) },
        InstructionType::dastore => { ("dastore", 0) },
        InstructionType::dcmpg => { ("dcmpg", 0) },
        InstructionType::dcmpl => { ("dcmpl", 0) },
        InstructionType::dconst_0 => { ("dconst_0", 0) },
        InstructionType::dconst_1 => { ("dconst_1", 0) },
        InstructionType::ddiv => { ("ddiv", 0) },
        InstructionType::dload => {
            ("dload(Index)", unimplemented!())
        },
        InstructionType::dload_0 => { ("dload_0", 0) },
        InstructionType::dload_1 => { ("dload_1", 0) },
        InstructionType::dload_2 => { ("dload_2", 0) },
        InstructionType::dload_3 => { ("dload_3", 0) },
        InstructionType::dmul => { ("dmul", 0) },
        InstructionType::dneg => { ("dneg", 0) },
        InstructionType::drem => { ("drem", 0) },
        InstructionType::dreturn => { ("dreturn", 0) },
        InstructionType::dstore => {
            ("dstore(Index)", unimplemented!())
        },
        InstructionType::dstore_0 => { ("dstore_0", 0) },
        InstructionType::dstore_1 => { ("dstore_1", 0) },
        InstructionType::dstore_2 => { ("dstore_2", 0) },
        InstructionType::dstore_3 => { ("dstore_3", 0) },
        InstructionType::dsub => { ("dsub", 0) },
        InstructionType::dup => { ("dup", 0) },
        InstructionType::dup_x1 => { ("dup_x1", 0) },
        InstructionType::dup_x2 => { ("dup_x2", 0) },
        InstructionType::dup2 => { ("dup2", 0) },
        InstructionType::dup2_x1 => { ("dup2_x1", 0) },
        InstructionType::dup2_x2 => { ("dup2_x2", 0) },
        InstructionType::f2d => { ("f2d", 0) },
        InstructionType::f2i => { ("f2i", 0) },
        InstructionType::f2l => { ("f2l", 0) },
        InstructionType::fadd => { ("fadd", 0) },
        InstructionType::faload => { ("faload", 0) },
        InstructionType::fastore => { ("fastore", 0) },
        InstructionType::fcmpg => { ("fcmpg", 0) },
        InstructionType::fcmpl => { ("fcmpl", 0) },
        InstructionType::fconst_0 => { ("fconst_0", 0) },
        InstructionType::fconst_1 => { ("fconst_1", 0) },
        InstructionType::fconst_2 => { ("fconst_2", 0) },
        InstructionType::fdiv => { ("fdiv", 0) },
        InstructionType::fload => {
            ("fload(Index)", unimplemented!())
        },
        InstructionType::fload_0 => { ("fload_0", 0) },
        InstructionType::fload_1 => { ("fload_1", 0) },
        InstructionType::fload_2 => { ("fload_2", 0) },
        InstructionType::fload_3 => { ("fload_3", 0) },
        InstructionType::fmul => { ("fmul", 0) },
        InstructionType::fneg => { ("fneg", 0) },
        InstructionType::frem => { ("frem", 0) },
        InstructionType::freturn => { ("freturn", 0) },
        InstructionType::fstore => {
            ("fstore(Index)", unimplemented!())
        },
        InstructionType::fstore_0 => { ("fstore_0", 0) },
        InstructionType::fstore_1 => { ("fstore_1", 0) },
        InstructionType::fstore_2 => { ("fstore_2", 0) },
        InstructionType::fstore_3 => { ("fstore_3", 0) },
        InstructionType::fsub => { ("fsub", 0) },
        InstructionType::getfield => {
            ("getfield(CP)", unimplemented!())
        },
        InstructionType::getstatic => {
            ("getstatic(CP)", unimplemented!())
        },
        InstructionType::goto_ => {
            ("goto_(Target)", unimplemented!())
        },
        InstructionType::goto_w => {
            ("goto_w(Target)", unimplemented!())
        },
        InstructionType::i2b => { ("i2b", 0) },
        InstructionType::i2c => { ("i2c", 0) },
        InstructionType::i2d => { ("i2d", 0) },
        InstructionType::i2f => { ("i2f", 0) },
        InstructionType::i2l => { ("i2l", 0) },
        InstructionType::i2s => { ("i2s", 0) },
        InstructionType::iadd => { ("iadd", 0) },
        InstructionType::iaload => { ("iaload", 0) },
        InstructionType::iand => { ("iand", 0) },
        InstructionType::iastore => { ("iastore", 0) },
        InstructionType::iconst_m1 => { ("iconst_m1", 0) },
        InstructionType::iconst_0 => { ("iconst_0", 0) },
        InstructionType::iconst_1 => { ("iconst_1", 0) },
        InstructionType::iconst_2 => { ("iconst_2", 0) },
        InstructionType::iconst_3 => { ("iconst_3", 0) },
        InstructionType::iconst_4 => { ("iconst_4", 0) },
        InstructionType::iconst_5 => { ("iconst_5", 0) },
        InstructionType::idiv => { ("idiv", 0) },
        InstructionType::if_acmpeq => {
            ("if_acmpeq(Target)", unimplemented!())
        },
        InstructionType::if_acmpne => {
            ("if_acmpne(Target)", unimplemented!())
        },
        InstructionType::if_icmpeq => {
            ("if_icmpeq(Target)", unimplemented!())
        },
        InstructionType::if_icmpne => {
            ("if_icmpne(Target)", unimplemented!())
        },
        InstructionType::if_icmplt => {
            ("if_icmplt(Target)", unimplemented!())
        },
        InstructionType::if_icmpge => {
            ("if_icmpge(Target)", unimplemented!())
        },
        InstructionType::if_icmpgt => {
            ("if_icmpgt(Target)", unimplemented!())
        },
        InstructionType::if_icmple => {
            ("if_icmple(Target)", unimplemented!())
        },
        InstructionType::ifeq => {
            ("ifeq(Target)", unimplemented!())
        },
        InstructionType::ifne => {
            ("ifne(Target)", unimplemented!())
        },
        InstructionType::iflt => {
            ("iflt(Target)", unimplemented!())
        },
        InstructionType::ifge => {
            ("ifge(Target)", unimplemented!())
        },
        InstructionType::ifgt => {
            ("ifgt(Target)", unimplemented!())
        },
        InstructionType::ifle => {
            ("ifle(Target)", unimplemented!())
        },
        InstructionType::ifnonnull => {
            ("ifnonnull(Target)", unimplemented!())
        },
        InstructionType::ifnull => {
            ("ifnull(Target)", unimplemented!())
        },
        InstructionType::iinc => {
            ("iinc(Index,_Value)", unimplemented!())
        },
        InstructionType::iload => {
            ("iload(Index)", unimplemented!())
        },
        InstructionType::iload_0 => { ("iload_0", 0) },
        InstructionType::iload_1 => { ("iload_1", 0) },
        InstructionType::iload_2 => { ("iload_2", 0) },
        InstructionType::iload_3 => { ("iload_3", 0) },
        InstructionType::imul => { ("imul", 0) },
        InstructionType::ineg => { ("ineg", 0) },
        InstructionType::instanceof => {
            ("instanceof(CP)", unimplemented!())
        },
        InstructionType::invokedynamic => {
            ("invokedynamic(CP,0,0)", unimplemented!())
        },
        InstructionType::invokeinterface => {
            ("invokeinterface(CP, Count, 0)", unimplemented!())
        },
        InstructionType::invokespecial => {
            ("invokespecial(CP)", unimplemented!())
        },
        InstructionType::invokestatic => {
            ("invokestatic(CP)", unimplemented!())
        },
        InstructionType::invokevirtual => {
            ("invokevirtual(CP)", unimplemented!())
        },
        InstructionType::ior => { ("ior", 0) },
        InstructionType::irem => { ("irem", 0) },
        InstructionType::ireturn => { ("ireturn", 0) },
        InstructionType::ishl => { ("ishl", 0) },
        InstructionType::ishr => { ("ishr", 0) },
        InstructionType::istore => {
            ("istore(Index)", unimplemented!())
        },
        InstructionType::istore_0 => { ("istore_0", 0) },
        InstructionType::istore_1 => { ("istore_1", 0) },
        InstructionType::istore_2 => { ("istore_2", 0) },
        InstructionType::istore_3 => { ("istore_3", 0) },
        InstructionType::isub => { ("isub", 0) },
        InstructionType::iushr => { ("iushr", 0) },
        InstructionType::ixor => { ("ixor", 0) },
        InstructionType::jsr => { ("jsr", 0) },
        InstructionType::jsr_w => { ("jsr_w", 0) },
        InstructionType::l2d => { ("l2d", 0) },
        InstructionType::l2f => { ("l2f", 0) },
        InstructionType::l2i => { ("l2i", 0) },
        InstructionType::ladd => { ("ladd", 0) },
        InstructionType::laload => { ("laload", 0) },
        InstructionType::land => { ("land", 0) },
        InstructionType::lastore => { ("lastore", 0) },
        InstructionType::lcmp => { ("lcmp", 0) },
        InstructionType::lconst_0 => { ("lconst_0", 0) },
        InstructionType::lconst_1 => { ("lconst_1", 0) },
        InstructionType::ldc => {
            ("ldc(CP)", unimplemented!())
        },
        InstructionType::ldc_w => {
            ("ldc_w(CP)", unimplemented!())
        },
        InstructionType::ldc2_w => {
            ("ldc2_w(CP)", unimplemented!())
        },
        InstructionType::ldiv => { ("ldiv", 0) },
        InstructionType::lload => {
            ("lload(Index)", unimplemented!())
        },
        InstructionType::lload_0 => { ("lload_0", 0) },
        InstructionType::lload_1 => { ("lload_1", 0) },
        InstructionType::lload_2 => { ("lload_2", 0) },
        InstructionType::lload_3 => { ("lload_3", 0) },
        InstructionType::lmul => { ("lmul", 0) },
        InstructionType::lneg => { ("lneg", 0) },
        InstructionType::lookupswitch => {
            ("lookupswitch(Targets, Keys)", unimplemented!())
        },
        InstructionType::lor => { ("lor", 0) },
        InstructionType::lrem => { ("lrem", 0) },
        InstructionType::lreturn => { ("lreturn", 0) },
        InstructionType::lshl => { ("lshl", 0) },
        InstructionType::lshr => { ("lshr", 0) },
        InstructionType::lstore => {
            ("lstore(Index)", unimplemented!())
        },
        InstructionType::lstore_0 => { ("lstore_0", 0) },
        InstructionType::lstore_1 => { ("lstore_1", 0) },
        InstructionType::lstore_2 => { ("lstore_2", 0) },
        InstructionType::lstore_3 => { ("lstore_3", 0) },
        InstructionType::lsub => { ("lsub", 0) },
        InstructionType::lushr => { ("lushr", 0) },
        InstructionType::lxor => { ("lxor", 0) },
        InstructionType::monitorenter => { ("monitorenter", 0) },
        InstructionType::monitorexit => { ("monitorexit", 0) },
        InstructionType::multianewarray => {
            ("multianewarray(CP, Dim)", unimplemented!())
        },
        InstructionType::new => {
            ("new(CP)", unimplemented!())
        },
        InstructionType::newarray => {
            ("newarray(TypeCode)", unimplemented!())
        },
        InstructionType::nop => { ("nop", 0) },
        InstructionType::pop => { ("pop", 0) },
        InstructionType::pop2 => { ("pop2", 0) },
        InstructionType::putfield => {
            ("putfield(CP)", unimplemented!())
        },
        InstructionType::putstatic => {
            ("putstatic(CP)", unimplemented!())
        },
        InstructionType::ret => { ("ret", 0) },
        InstructionType::return_ => { ("return_", 0) },
        InstructionType::saload => { ("saload", 0) },
        InstructionType::sastore => { ("sastore", 0) },
        InstructionType::sipush => {
            ("sipush(_Value)", unimplemented!())
        },
        InstructionType::swap => { ("swap", 0) },
        InstructionType::tableswitch => {
            ("tableswitch(Targets, Keys)", unimplemented!())
        },
        InstructionType::wide => {
            ("wide(WidenedInstruction)", unimplemented!())
        },
    }
}

fn output_instruction_info_for_code(code: Code, w: &mut dyn Write) -> Result<(), Error> {
    let mut skip = 0;
    for (i, code_byte) in code.code.iter().enumerate() {
        if skip > 0 {
            skip = skip - 1;
            continue
        }
        write!(w, "instruction(")?;
        write!(w, "{}", i)?;
        let (string, skip_copy) = instruction_to_string(i, &code.code);
        skip = skip_copy;
        write!(w, ", {}).\n", string)?;
    }
    Ok(())
}
