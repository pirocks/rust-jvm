use std::io::{Error, Write};

use classfile::attribute_infos::Code;
use interpreter::{InstructionType, read_opcode};


//todo for stuff which refers to CP, need to use functor representation. See 4.10.1.3.

fn instruction_to_string(i: usize, whole_code: &Vec<u8>) -> (String, u64) {
    let code = &whole_code[i..whole_code.len()];
    match read_opcode(whole_code[i]) {
        InstructionType::aaload => { ("aaload".to_string(), 0) },
        InstructionType::aastore => { ("aastore".to_string(), 0) },
        InstructionType::aconst_null => { ("aconst_null".to_string(), 0) },
        InstructionType::aload => {
            let index = code[1];
            (format!("aload({})",index), 1)
        },
        InstructionType::aload_0 => { ("aload_0".to_string(), 0) },
        InstructionType::aload_1 => { ("aload_1".to_string(), 0) },
        InstructionType::aload_2 => { ("aload_2".to_string(), 0) },
        InstructionType::aload_3 => { ("aload_3".to_string(), 0) },
        InstructionType::anewarray => {
            let indexbyte1 = code[1] as u16;
            let indexbyte2 = code[2] as u16;
            let index = (indexbyte1 << 8) | indexbyte2;
            (format!("anewarray({})",index), 2)
        },
        InstructionType::areturn => { ("areturn".to_string(), 0) },
        InstructionType::arraylength => { ("arraylength".to_string(), 0) },
        InstructionType::astore => {
            let index = code[1];
            (format!("astore({})",index), 1)
        },
        InstructionType::astore_0 => { ("astore_0".to_string(), 0) },
        InstructionType::astore_1 => { ("astore_1".to_string(), 0) },
        InstructionType::astore_2 => { ("astore_2".to_string(), 0) },
        InstructionType::astore_3 => { ("astore_3".to_string(), 0) },
        InstructionType::athrow => { ("athrow".to_string(), 0) },
        InstructionType::baload => { ("baload".to_string(), 0) },
        InstructionType::bastore => { ("bastore".to_string(), 0) },
        InstructionType::bipush => {
            let byte = code[1];
            (format!("bipush({})",byte), 1)
        },
        InstructionType::caload => { ("caload".to_string(), 0) },
        InstructionType::castore => { ("castore".to_string(), 0) },
        InstructionType::checkcast => {
            let indexbyte1 = code[1] as u16;
            let indexbyte2 = code[2] as u16;
            let index = (indexbyte1 << 8) | indexbyte2;
            (format!("checkcast({})",index), 2)
        },
        InstructionType::d2f => { ("d2f".to_string(), 0) },
        InstructionType::d2i => { ("d2i".to_string(), 0) },
        InstructionType::d2l => { ("d2l".to_string(), 0) },
        InstructionType::dadd => { ("dadd".to_string(), 0) },
        InstructionType::daload => { ("daload".to_string(), 0) },
        InstructionType::dastore => { ("dastore".to_string(), 0) },
        InstructionType::dcmpg => { ("dcmpg".to_string(), 0) },
        InstructionType::dcmpl => { ("dcmpl".to_string(), 0) },
        InstructionType::dconst_0 => { ("dconst_0".to_string(), 0) },
        InstructionType::dconst_1 => { ("dconst_1".to_string(), 0) },
        InstructionType::ddiv => { ("ddiv".to_string(), 0) },
        InstructionType::dload => {
            let index = code[1];
            (format!("dload({})",index), 1)
        },
        InstructionType::dload_0 => { ("dload_0".to_string(), 0) },
        InstructionType::dload_1 => { ("dload_1".to_string(), 0) },
        InstructionType::dload_2 => { ("dload_2".to_string(), 0) },
        InstructionType::dload_3 => { ("dload_3".to_string(), 0) },
        InstructionType::dmul => { ("dmul".to_string(), 0) },
        InstructionType::dneg => { ("dneg".to_string(), 0) },
        InstructionType::drem => { ("drem".to_string(), 0) },
        InstructionType::dreturn => { ("dreturn".to_string(), 0) },
        InstructionType::dstore => {
            let index = code[1];
            (format!("dstore({})",index), 1)
        },
        InstructionType::dstore_0 => { ("dstore_0".to_string(), 0) },
        InstructionType::dstore_1 => { ("dstore_1".to_string(), 0) },
        InstructionType::dstore_2 => { ("dstore_2".to_string(), 0) },
        InstructionType::dstore_3 => { ("dstore_3".to_string(), 0) },
        InstructionType::dsub => { ("dsub".to_string(), 0) },
        InstructionType::dup => { ("dup".to_string(), 0) },
        InstructionType::dup_x1 => { ("dup_x1".to_string(), 0) },
        InstructionType::dup_x2 => { ("dup_x2".to_string(), 0) },
        InstructionType::dup2 => { ("dup2".to_string(), 0) },
        InstructionType::dup2_x1 => { ("dup2_x1".to_string(), 0) },
        InstructionType::dup2_x2 => { ("dup2_x2".to_string(), 0) },
        InstructionType::f2d => { ("f2d".to_string(), 0) },
        InstructionType::f2i => { ("f2i".to_string(), 0) },
        InstructionType::f2l => { ("f2l".to_string(), 0) },
        InstructionType::fadd => { ("fadd".to_string(), 0) },
        InstructionType::faload => { ("faload".to_string(), 0) },
        InstructionType::fastore => { ("fastore".to_string(), 0) },
        InstructionType::fcmpg => { ("fcmpg".to_string(), 0) },
        InstructionType::fcmpl => { ("fcmpl".to_string(), 0) },
        InstructionType::fconst_0 => { ("fconst_0".to_string(), 0) },
        InstructionType::fconst_1 => { ("fconst_1".to_string(), 0) },
        InstructionType::fconst_2 => { ("fconst_2".to_string(), 0) },
        InstructionType::fdiv => { ("fdiv".to_string(), 0) },
        InstructionType::fload => {
            let index = code[1];
            (format!("fload({})",index), 1)
        },
        InstructionType::fload_0 => { ("fload_0".to_string(), 0) },
        InstructionType::fload_1 => { ("fload_1".to_string(), 0) },
        InstructionType::fload_2 => { ("fload_2".to_string(), 0) },
        InstructionType::fload_3 => { ("fload_3".to_string(), 0) },
        InstructionType::fmul => { ("fmul".to_string(), 0) },
        InstructionType::fneg => { ("fneg".to_string(), 0) },
        InstructionType::frem => { ("frem".to_string(), 0) },
        InstructionType::freturn => { ("freturn".to_string(), 0) },
        InstructionType::fstore => {
            let index = code[1];
            (format!("fstore({})",index), 1)
        },
        InstructionType::fstore_0 => { ("fstore_0".to_string(), 0) },
        InstructionType::fstore_1 => { ("fstore_1".to_string(), 0) },
        InstructionType::fstore_2 => { ("fstore_2".to_string(), 0) },
        InstructionType::fstore_3 => { ("fstore_3".to_string(), 0) },
        InstructionType::fsub => { ("fsub".to_string(), 0) },
        InstructionType::getfield => {
            let indexbyte1 = code[1] as u16;
            let indexbyte2 = code[2] as u16;
            let index = (indexbyte1 << 8) | indexbyte2;
            (format!("getfield({})",index), 2)
        },
        InstructionType::getstatic => {
            let indexbyte1 = code[1] as u16;
            let indexbyte2 = code[2] as u16;
            let index = (indexbyte1 << 8) | indexbyte2;
            (format!("getstatic({})",index), 2)
        },
        InstructionType::goto_ => {
            let indexbyte1 = code[1] as u16;
            let indexbyte2 = code[2] as u16;
            let index = ((indexbyte1 << 8) | indexbyte2) as i16;
            (format!("goto({})",index + i as i16), 2)
        },
        InstructionType::goto_w => {
            let branchbyte1 = code[1] as u32;
            let branchbyte2 = code[2] as u32;
            let branchbyte3 = code[3] as u32;
            let branchbyte4 = code[4] as u32;
            let branch = ((branchbyte1 << 24) | (branchbyte2 << 16)
                | (branchbyte3 << 8) | branchbyte4) as i32;
            (format!("goto_w({})",branch + i as i32), 4)//todo overflow risk here and other places where +i is used
        },
        InstructionType::i2b => { ("i2b".to_string(), 0) },
        InstructionType::i2c => { ("i2c".to_string(), 0) },
        InstructionType::i2d => { ("i2d".to_string(), 0) },
        InstructionType::i2f => { ("i2f".to_string(), 0) },
        InstructionType::i2l => { ("i2l".to_string(), 0) },
        InstructionType::i2s => { ("i2s".to_string(), 0) },
        InstructionType::iadd => { ("iadd".to_string(), 0) },
        InstructionType::iaload => { ("iaload".to_string(), 0) },
        InstructionType::iand => { ("iand".to_string(), 0) },
        InstructionType::iastore => { ("iastore".to_string(), 0) },
        InstructionType::iconst_m1 => { ("iconst_m1".to_string(), 0) },
        InstructionType::iconst_0 => { ("iconst_0".to_string(), 0) },
        InstructionType::iconst_1 => { ("iconst_1".to_string(), 0) },
        InstructionType::iconst_2 => { ("iconst_2".to_string(), 0) },
        InstructionType::iconst_3 => { ("iconst_3".to_string(), 0) },
        InstructionType::iconst_4 => { ("iconst_4".to_string(), 0) },
        InstructionType::iconst_5 => { ("iconst_5".to_string(), 0) },
        InstructionType::idiv => { ("idiv".to_string(), 0) },
        InstructionType::if_acmpeq => {
            let indexbyte1 = code[1] as u16;
            let indexbyte2 = code[2] as u16;
            let index = ((indexbyte1 << 8) | indexbyte2) as i16;
            (format!("if_acmpeq({})",index + i as i16), 2)//todo duplication
        },
        InstructionType::if_acmpne => {
            let indexbyte1 = code[1] as u16;
            let indexbyte2 = code[2] as u16;
            let index = ((indexbyte1 << 8) | indexbyte2) as i16;
            (format!("if_acmpne({})",index + i as i16), 2)//todo duplication
        },
        InstructionType::if_icmpeq => {
            let indexbyte1 = code[1] as u16;
            let indexbyte2 = code[2] as u16;
            let index = ((indexbyte1 << 8) | indexbyte2) as i16;
            (format!("if_icmpeq({})",index + i as i16), 2)//todo duplication
        },
        InstructionType::if_icmpne => {
            let indexbyte1 = code[1] as u16;
            let indexbyte2 = code[2] as u16;
            let index = ((indexbyte1 << 8) | indexbyte2) as i16;
            (format!("if_icmpne({})",index + i as i16), 2)//todo duplication
        },
        InstructionType::if_icmplt => {
            let indexbyte1 = code[1] as u16;
            let indexbyte2 = code[2] as u16;
            let index = ((indexbyte1 << 8) | indexbyte2) as i16;
            (format!("if_icmplt({})",index + i as i16), 2)//todo duplication
        },
        InstructionType::if_icmpge => {
            let indexbyte1 = code[1] as u16;
            let indexbyte2 = code[2] as u16;
            let index = ((indexbyte1 << 8) | indexbyte2) as i16;
            (format!("if_icmpge({})",index + i as i16), 2)//todo duplication
        },
        InstructionType::if_icmpgt => {
            let indexbyte1 = code[1] as u16;
            let indexbyte2 = code[2] as u16;
            let index = ((indexbyte1 << 8) | indexbyte2) as i16;
            (format!("if_icmpgt({})",index + i as i16), 2)//todo duplication
        },
        InstructionType::if_icmple => {
            let indexbyte1 = code[1] as u16;
            let indexbyte2 = code[2] as u16;
            let index = ((indexbyte1 << 8) | indexbyte2) as i16;
            (format!("if_icmple({})",index + i as i16), 2)//todo duplication
        },
        InstructionType::ifeq => {
            let indexbyte1 = code[1] as u16;
            let indexbyte2 = code[2] as u16;
            let index = ((indexbyte1 << 8) | indexbyte2) as i16;
            (format!("ifeq({})",index + i as i16), 2)//todo duplication
        },
        InstructionType::ifne => {
            let indexbyte1 = code[1] as u16;
            let indexbyte2 = code[2] as u16;
            let index = ((indexbyte1 << 8) | indexbyte2) as i16;
            (format!("ifne({})",index + i as i16), 2)//todo duplication
        },
        InstructionType::iflt => {
            let indexbyte1 = code[1] as u16;
            let indexbyte2 = code[2] as u16;
            let index = ((indexbyte1 << 8) | indexbyte2) as i16;
            (format!("iflt({})",index + i as i16), 2)//todo duplication
        },
        InstructionType::ifge => {
            let indexbyte1 = code[1] as u16;
            let indexbyte2 = code[2] as u16;
            let index = ((indexbyte1 << 8) | indexbyte2) as i16;
            (format!("ifge({})",index + i as i16), 2)//todo duplication
        },
        InstructionType::ifgt => {
            let indexbyte1 = code[1] as u16;
            let indexbyte2 = code[2] as u16;
            let index = ((indexbyte1 << 8) | indexbyte2) as i16;
            (format!("ifgt({})",index + i as i16), 2)//todo duplication
        },
        InstructionType::ifle => {
            let indexbyte1 = code[1] as u16;
            let indexbyte2 = code[2] as u16;
            let index = ((indexbyte1 << 8) | indexbyte2) as i16;
            (format!("ifle({})",index + i as i16), 2)//todo duplication
        },
        InstructionType::ifnonnull => {
            let indexbyte1 = code[1] as u16;
            let indexbyte2 = code[2] as u16;
            let index = ((indexbyte1 << 8) | indexbyte2) as i16;
            (format!("ifnonnull({})",index + i as i16), 2)//todo duplication
        },
        InstructionType::ifnull => {
            let indexbyte1 = code[1] as u16;
            let indexbyte2 = code[2] as u16;
            let index = ((indexbyte1 << 8) | indexbyte2) as i16;
            (format!("ifnull({})",index + i as i16), 2)//todo duplication
        },
        InstructionType::iinc => {
            let index = code[1];
            let const_ = code[2];
            (format!("iinc({},{})", index, const_), 2)
        },
        InstructionType::iload => {
            let index = code[1];
            (format!("iload({})", index), 1)
        },
        InstructionType::iload_0 => { ("iload_0".to_string(), 0) },
        InstructionType::iload_1 => { ("iload_1".to_string(), 0) },
        InstructionType::iload_2 => { ("iload_2".to_string(), 0) },
        InstructionType::iload_3 => { ("iload_3".to_string(), 0) },
        InstructionType::imul => { ("imul".to_string(), 0) },
        InstructionType::ineg => { ("ineg".to_string(), 0) },
        InstructionType::instanceof => {
            let indexbyte1 = code[1] as u16;
            let indexbyte2 = code[2] as u16;
            let index = ((indexbyte1 << 8) | indexbyte2) as u16;
            (format!("instanceof({})",index), 2)
        },
        InstructionType::invokedynamic => {
            let indexbyte1 = code[1] as u16;
            let indexbyte2 = code[2] as u16;
            let index = ((indexbyte1 << 8) | indexbyte2) as u16;
            (format!("invokedynamic({},0,0)",index), 4)
        },
        InstructionType::invokeinterface => {
            let indexbyte1 = code[1] as u16;
            let indexbyte2 = code[2] as u16;
            let index = ((indexbyte1 << 8) | indexbyte2) as u16;
            let count = code[3];
            (format!("invokeinterface({}, {}, 0)",index,count),4)
        },
        InstructionType::invokespecial => {
            let indexbyte1 = code[1] as u16;
            let indexbyte2 = code[2] as u16;
            let index = ((indexbyte1 << 8) | indexbyte2) as u16;
            (format!("invokespecial({})",index), 2)
        },
        InstructionType::invokestatic => {
            let indexbyte1 = code[1] as u16;
            let indexbyte2 = code[2] as u16;
            let index = ((indexbyte1 << 8) | indexbyte2) as u16;
            (format!("invokestatic({})",index), 2)
        },
        InstructionType::invokevirtual => {
            let indexbyte1 = code[1] as u16;
            let indexbyte2 = code[2] as u16;
            let index = ((indexbyte1 << 8) | indexbyte2) as u16;
            (format!("invokevirtual({})",index), 2)
        },
        InstructionType::ior => { ("ior".to_string(), 0) },
        InstructionType::irem => { ("irem".to_string(), 0) },
        InstructionType::ireturn => { ("ireturn".to_string(), 0) },
        InstructionType::ishl => { ("ishl".to_string(), 0) },
        InstructionType::ishr => { ("ishr".to_string(), 0) },
        InstructionType::istore => {
            (format!("istore({})",code[1]), 1)
        },
        InstructionType::istore_0 => { ("istore_0".to_string(), 0) },
        InstructionType::istore_1 => { ("istore_1".to_string(), 0) },
        InstructionType::istore_2 => { ("istore_2".to_string(), 0) },
        InstructionType::istore_3 => { ("istore_3".to_string(), 0) },
        InstructionType::isub => { ("isub".to_string(), 0) },
        InstructionType::iushr => { ("iushr".to_string(), 0) },
        InstructionType::ixor => { ("ixor".to_string(), 0) },
        InstructionType::jsr => { ("jsr".to_string(), 0) },
        InstructionType::jsr_w => { ("jsr_w".to_string(), 0) },
        InstructionType::l2d => { ("l2d".to_string(), 0) },
        InstructionType::l2f => { ("l2f".to_string(), 0) },
        InstructionType::l2i => { ("l2i".to_string(), 0) },
        InstructionType::ladd => { ("ladd".to_string(), 0) },
        InstructionType::laload => { ("laload".to_string(), 0) },
        InstructionType::land => { ("land".to_string(), 0) },
        InstructionType::lastore => { ("lastore".to_string(), 0) },
        InstructionType::lcmp => { ("lcmp".to_string(), 0) },
        InstructionType::lconst_0 => { ("lconst_0".to_string(), 0) },
        InstructionType::lconst_1 => { ("lconst_1".to_string(), 0) },
        InstructionType::ldc => {
            (format!("ldc({})",code[1]), 1)
        },
        InstructionType::ldc_w => {
            let indexbyte1 = code[1] as u16;
            let indexbyte2 = code[2] as u16;
            let index = ((indexbyte1 << 8) | indexbyte2) as u16;
            (format!("ldc_w({})",index), 2)
        },
        InstructionType::ldc2_w => {
            let indexbyte1 = code[1] as u16;
            let indexbyte2 = code[2] as u16;
            let index = ((indexbyte1 << 8) | indexbyte2) as u16;
            (format!("ldc2_w({})",index), 2)
        },
        InstructionType::ldiv => { ("ldiv".to_string(), 0) },
        InstructionType::lload => {
            (format!("lload({})",code[1]), 1)
        },
        InstructionType::lload_0 => { ("lload_0".to_string(), 0) },
        InstructionType::lload_1 => { ("lload_1".to_string(), 0) },
        InstructionType::lload_2 => { ("lload_2".to_string(), 0) },
        InstructionType::lload_3 => { ("lload_3".to_string(), 0) },
        InstructionType::lmul => { ("lmul".to_string(), 0) },
        InstructionType::lneg => { ("lneg".to_string(), 0) },
        InstructionType::lookupswitch => {
            ("lookupswitch(Targets, Keys)".to_string(), unimplemented!())
        },
        InstructionType::lor => { ("lor".to_string(), 0) },
        InstructionType::lrem => { ("lrem".to_string(), 0) },
        InstructionType::lreturn => { ("lreturn".to_string(), 0) },
        InstructionType::lshl => { ("lshl".to_string(), 0) },
        InstructionType::lshr => { ("lshr".to_string(), 0) },
        InstructionType::lstore => {
            (format!("lstore({})",code[1]), 1)
        },
        InstructionType::lstore_0 => { ("lstore_0".to_string(), 0) },
        InstructionType::lstore_1 => { ("lstore_1".to_string(), 0) },
        InstructionType::lstore_2 => { ("lstore_2".to_string(), 0) },
        InstructionType::lstore_3 => { ("lstore_3".to_string(), 0) },
        InstructionType::lsub => { ("lsub".to_string(), 0) },
        InstructionType::lushr => { ("lushr".to_string(), 0) },
        InstructionType::lxor => { ("lxor".to_string(), 0) },
        InstructionType::monitorenter => { ("monitorenter".to_string(), 0) },
        InstructionType::monitorexit => { ("monitorexit".to_string(), 0) },
        InstructionType::multianewarray => {
            let indexbyte1 = code[1] as u16;
            let indexbyte2 = code[2] as u16;
            let index = ((indexbyte1 << 8) | indexbyte2) as u16;
            let dimensions = code[3];
            (format!("multianewarray({}, {})",index,dimensions), 3)
        },
        InstructionType::new => {
            let indexbyte1 = code[1] as u16;
            let indexbyte2 = code[2] as u16;
            let index = ((indexbyte1 << 8) | indexbyte2) as u16;
            (format!("new({})",index), 2)
        },
        InstructionType::newarray => {
            (format!("newarray({})",code[1]), 1)
        },
        InstructionType::nop => { ("nop".to_string(), 0) },
        InstructionType::pop => { ("pop".to_string(), 0) },
        InstructionType::pop2 => { ("pop2".to_string(), 0) },
        InstructionType::putfield => {
            let indexbyte1 = code[1] as u16;
            let indexbyte2 = code[2] as u16;
            let index = ((indexbyte1 << 8) | indexbyte2) as u16;
            (format!("putfield({})",index), 2)
        },
        InstructionType::putstatic => {
            let indexbyte1 = code[1] as u16;
            let indexbyte2 = code[2] as u16;
            let index = ((indexbyte1 << 8) | indexbyte2) as u16;
            (format!("putstatic({})",index), 2)
        },
        InstructionType::ret => { ("ret".to_string(), 0) },
        InstructionType::return_ => { ("return".to_string(), 0) },
        InstructionType::saload => { ("saload".to_string(), 0) },
        InstructionType::sastore => { ("sastore".to_string(), 0) },
        InstructionType::sipush => {
            let byte1 = code[1] as u16;
            let byte2 = code[2] as u16;
            let value = ((byte1 << 8) | byte2) as i16;
            (format!("sipush({})",value), 2)
        },
        InstructionType::swap => { ("swap".to_string(), 0) },
        InstructionType::tableswitch => {
            ("tableswitch(Targets, Keys)".to_string(), unimplemented!())
        },
        InstructionType::wide => {
            ("wide(WidenedInstruction)".to_string(), unimplemented!())
        },
    }
}

pub fn output_instruction_info_for_code(code: &Code, w: &mut dyn Write) -> Result<(), Error> {
    let mut skip = 0;
    write!(w,"[")?;
    //todo simplify
    let mut final_offset = 0;
    for (i, _) in code.code.iter().enumerate() {
        if skip > 0 {
            skip = skip - 1;
            continue
        }else if i != 0{
            write!(w,",")?;
        }
        write!(w, "instruction(")?;
        write!(w, "{}", i)?;
        let (string, skip_copy) = instruction_to_string(i, &code.code);
        skip = skip_copy;
        write!(w, ", {})", string)?;
        final_offset = i;
    }
    write!(w,",endOfCode({})",final_offset)?;
    write!(w,"],")?;
    Ok(())
}
