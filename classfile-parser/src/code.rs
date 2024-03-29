use std::slice::Iter;

use num_traits::FromPrimitive;

use rust_jvm_common::ByteCodeOffset;
use rust_jvm_common::classfile::{Atype, IInc, Instruction, InstructionInfo, InvokeInterface, LookupSwitch, MultiNewArray, TableSwitch, Wide, WideAload, WideAstore, WideDload, WideDstore, WideFload, WideFstore, WideIload, WideIstore, WideLload, WideLstore, WideRet};
use rust_jvm_common::classfile::instruction_info_nums::InstructionTypeNum;

use crate::ClassfileParsingError;

fn read_iinc(c: &mut CodeParserContext) -> Result<IInc, ClassfileParsingError> {
    let index = read_u8(c)?;
    let const_ = read_i8(c)?;
    Ok(IInc { index: index as u16, const_: const_ as i16 })
}

fn read_invoke_interface(c: &mut CodeParserContext) -> Result<InvokeInterface, ClassfileParsingError> {
    let index = read_u16(c)?;
    let count = read_u8(c)?;
    assert_ne!(count, 0);
    let zero = read_u8(c)?;
    assert_eq!(zero, 0);
    Ok(InvokeInterface { index, count })
}

fn read_lookup_switch(c: &mut CodeParserContext) -> Result<LookupSwitch, ClassfileParsingError> {
    while c.offset.0 % 4 != 0 {
        let _padding = read_u8(c)?;
    }
    let default = read_i32(c)?;
    let npairs = read_i32(c)?;
    assert!(npairs > 0);
    let mut pairs = vec![];
    for _ in 0..npairs {
        pairs.push((read_i32(c)?, read_i32(c)?));
    }
    Ok(LookupSwitch { default, pairs })
}

fn read_multi_new_array(c: &mut CodeParserContext) -> Result<MultiNewArray, ClassfileParsingError> {
    let index = read_u16(c)?;
    let dims = read_u8(c)?;
    Ok(MultiNewArray { index, dims })
}

fn read_atype(c: &mut CodeParserContext) -> Result<Atype, ClassfileParsingError> {
    let num = read_u8(c)?;
    FromPrimitive::from_u8(num).ok_or(ClassfileParsingError::ATypeWrong)
}

fn read_table_switch(c: &mut CodeParserContext) -> Result<TableSwitch, ClassfileParsingError> {
    while c.offset.0 % 4 != 0 {
        read_u8(c)?;
    }
    let default = read_i32(c)?;
    let low = read_i32(c)?;
    let high = read_i32(c)?;
    let num_to_read = high - low + 1;
    let mut offsets = vec![];
    for _ in 0..num_to_read {
        offsets.push(read_i32(c)?);
    }
    Ok(TableSwitch { default, low, high, offsets })
}

fn read_wide(c: &mut CodeParserContext) -> Result<Wide, ClassfileParsingError> {
    let opcode = read_opcode(read_u8(c)?)?;

    Ok(if opcode == InstructionTypeNum::iinc {
        let index = read_u16(c)?;
        let const_ = read_i16(c)?;
        Wide::IInc(IInc { index, const_ })
    } else {
        let index = read_u16(c)?;
        //iload, fload, aload, lload, dload, istore,
        // fstore, astore, lstore, dstore, or ret
        match opcode {
            InstructionTypeNum::iload => Wide::Iload(WideIload { index }),
            InstructionTypeNum::fload => Wide::Fload(WideFload { index }),
            InstructionTypeNum::aload => Wide::Aload(WideAload { index }),
            InstructionTypeNum::lload => Wide::Lload(WideLload { index }),
            InstructionTypeNum::dload => Wide::Dload(WideDload { index }),
            InstructionTypeNum::istore => Wide::Istore(WideIstore { index }),
            InstructionTypeNum::fstore => Wide::Fstore(WideFstore { index }),
            InstructionTypeNum::astore => Wide::Astore(WideAstore { index }),
            InstructionTypeNum::lstore => Wide::Lstore(WideLstore { index }),
            InstructionTypeNum::dstore => Wide::Dstore(WideDstore { index }),
            InstructionTypeNum::ret => Wide::Ret(WideRet { index }),
            _ => return Err(ClassfileParsingError::WrongTag),
        }
    })
}

pub struct CodeParserContext<'l> {
    pub offset: ByteCodeOffset,
    pub iter: Iter<'l, u8>,
}

pub fn parse_code_raw(raw: &[u8]) -> Result<Vec<Instruction>, ClassfileParsingError> {
    //is this offset of 0 even correct?
    // what if code starts at non-aligned?
    let mut c = CodeParserContext { iter: raw.iter(), offset: ByteCodeOffset(0) };
    parse_code_impl(&mut c)
}

fn read_u8(c: &mut CodeParserContext) -> Result<u8, ClassfileParsingError> {
    c.offset.0 += 1;
    let next = c.iter.next().ok_or(ClassfileParsingError::EndOfInstructions);
    Ok(*next?)
}

fn read_i8(c: &mut CodeParserContext) -> Result<i8, ClassfileParsingError> {
    assert_eq!(255u8 as i8, -1i8); //please don't judge future reader
    Ok(read_u8(c)? as i8)
}

fn read_u16(c: &mut CodeParserContext) -> Result<u16, ClassfileParsingError> {
    let byte1 = read_u8(c)? as u16;
    let byte2 = read_u8(c)? as u16;
    Ok(byte1 << 8 | byte2)
}

fn read_i16(c: &mut CodeParserContext) -> Result<i16, ClassfileParsingError> {
    Ok(read_u16(c)? as i16)
}

fn read_u32(c: &mut CodeParserContext) -> Result<u32, ClassfileParsingError> {
    let byte1 = read_u8(c)? as u32;
    let byte2 = read_u8(c)? as u32;
    let byte3 = read_u8(c)? as u32;
    let byte4 = read_u8(c)? as u32;
    Ok(byte1 << 24 | byte2 << 16 | byte3 << 8 | byte4)
}

fn read_i32(c: &mut CodeParserContext) -> Result<i32, ClassfileParsingError> {
    Ok(read_u32(c)? as i32)
}

pub fn read_opcode(b: u8) -> Result<InstructionTypeNum, ClassfileParsingError> {
    FromPrimitive::from_u8(b).ok_or(ClassfileParsingError::WrongInstructionType)
}


pub fn parse_instruction(c: &mut CodeParserContext) -> Result<InstructionInfo, ClassfileParsingError> {
    let opcode = read_opcode(read_u8(c)?)?;
    Ok(match opcode {
        InstructionTypeNum::aaload => InstructionInfo::aaload,
        InstructionTypeNum::aastore => InstructionInfo::aastore,
        InstructionTypeNum::aconst_null => InstructionInfo::aconst_null,
        InstructionTypeNum::aload => InstructionInfo::aload(read_u8(c)?),
        InstructionTypeNum::aload_0 => InstructionInfo::aload_0,
        InstructionTypeNum::aload_1 => InstructionInfo::aload_1,
        InstructionTypeNum::aload_2 => InstructionInfo::aload_2,
        InstructionTypeNum::aload_3 => InstructionInfo::aload_3,
        InstructionTypeNum::anewarray => InstructionInfo::anewarray(read_u16(c)?),
        InstructionTypeNum::areturn => InstructionInfo::areturn,
        InstructionTypeNum::arraylength => InstructionInfo::arraylength,
        InstructionTypeNum::astore => InstructionInfo::astore(read_u8(c)?),
        InstructionTypeNum::astore_0 => InstructionInfo::astore_0,
        InstructionTypeNum::astore_1 => InstructionInfo::astore_1,
        InstructionTypeNum::astore_2 => InstructionInfo::astore_2,
        InstructionTypeNum::astore_3 => InstructionInfo::astore_3,
        InstructionTypeNum::athrow => InstructionInfo::athrow,
        InstructionTypeNum::baload => InstructionInfo::baload,
        InstructionTypeNum::bastore => InstructionInfo::bastore,
        InstructionTypeNum::bipush => InstructionInfo::bipush(read_i8(c)?),
        InstructionTypeNum::caload => InstructionInfo::caload,
        InstructionTypeNum::castore => InstructionInfo::castore,
        InstructionTypeNum::checkcast => InstructionInfo::checkcast(read_u16(c)?),
        InstructionTypeNum::d2f => InstructionInfo::d2f,
        InstructionTypeNum::d2i => InstructionInfo::d2i,
        InstructionTypeNum::d2l => InstructionInfo::d2l,
        InstructionTypeNum::dadd => InstructionInfo::dadd,
        InstructionTypeNum::daload => InstructionInfo::daload,
        InstructionTypeNum::dastore => InstructionInfo::dastore,
        InstructionTypeNum::dcmpg => InstructionInfo::dcmpg,
        InstructionTypeNum::dcmpl => InstructionInfo::dcmpl,
        InstructionTypeNum::dconst_0 => InstructionInfo::dconst_0,
        InstructionTypeNum::dconst_1 => InstructionInfo::dconst_1,
        InstructionTypeNum::ddiv => InstructionInfo::ddiv,
        InstructionTypeNum::dload => InstructionInfo::dload(read_u8(c)?),
        InstructionTypeNum::dload_0 => InstructionInfo::dload_0,
        InstructionTypeNum::dload_1 => InstructionInfo::dload_1,
        InstructionTypeNum::dload_2 => InstructionInfo::dload_2,
        InstructionTypeNum::dload_3 => InstructionInfo::dload_3,
        InstructionTypeNum::dmul => InstructionInfo::dmul,
        InstructionTypeNum::dneg => InstructionInfo::dneg,
        InstructionTypeNum::drem => InstructionInfo::drem,
        InstructionTypeNum::dreturn => InstructionInfo::dreturn,
        InstructionTypeNum::dstore => InstructionInfo::dstore(read_u8(c)?),
        InstructionTypeNum::dstore_0 => InstructionInfo::dstore_0,
        InstructionTypeNum::dstore_1 => InstructionInfo::dstore_1,
        InstructionTypeNum::dstore_2 => InstructionInfo::dstore_2,
        InstructionTypeNum::dstore_3 => InstructionInfo::dstore_3,
        InstructionTypeNum::dsub => InstructionInfo::dsub,
        InstructionTypeNum::dup => InstructionInfo::dup,
        InstructionTypeNum::dup_x1 => InstructionInfo::dup_x1,
        InstructionTypeNum::dup_x2 => InstructionInfo::dup_x2,
        InstructionTypeNum::dup2 => InstructionInfo::dup2,
        InstructionTypeNum::dup2_x1 => InstructionInfo::dup2_x1,
        InstructionTypeNum::dup2_x2 => InstructionInfo::dup2_x2,
        InstructionTypeNum::f2d => InstructionInfo::f2d,
        InstructionTypeNum::f2i => InstructionInfo::f2i,
        InstructionTypeNum::f2l => InstructionInfo::f2l,
        InstructionTypeNum::fadd => InstructionInfo::fadd,
        InstructionTypeNum::faload => InstructionInfo::faload,
        InstructionTypeNum::fastore => InstructionInfo::fastore,
        InstructionTypeNum::fcmpg => InstructionInfo::fcmpg,
        InstructionTypeNum::fcmpl => InstructionInfo::fcmpl,
        InstructionTypeNum::fconst_0 => InstructionInfo::fconst_0,
        InstructionTypeNum::fconst_1 => InstructionInfo::fconst_1,
        InstructionTypeNum::fconst_2 => InstructionInfo::fconst_2,
        InstructionTypeNum::fdiv => InstructionInfo::fdiv,
        InstructionTypeNum::fload => InstructionInfo::fload(read_u8(c)?),
        InstructionTypeNum::fload_0 => InstructionInfo::fload_0,
        InstructionTypeNum::fload_1 => InstructionInfo::fload_1,
        InstructionTypeNum::fload_2 => InstructionInfo::fload_2,
        InstructionTypeNum::fload_3 => InstructionInfo::fload_3,
        InstructionTypeNum::fmul => InstructionInfo::fmul,
        InstructionTypeNum::fneg => InstructionInfo::fneg,
        InstructionTypeNum::frem => InstructionInfo::frem,
        InstructionTypeNum::freturn => InstructionInfo::freturn,
        InstructionTypeNum::fstore => InstructionInfo::fstore(read_u8(c)?),
        InstructionTypeNum::fstore_0 => InstructionInfo::fstore_0,
        InstructionTypeNum::fstore_1 => InstructionInfo::fstore_1,
        InstructionTypeNum::fstore_2 => InstructionInfo::fstore_2,
        InstructionTypeNum::fstore_3 => InstructionInfo::fstore_3,
        InstructionTypeNum::fsub => InstructionInfo::fsub,
        InstructionTypeNum::getfield => InstructionInfo::getfield(read_u16(c)?),
        InstructionTypeNum::getstatic => InstructionInfo::getstatic(read_u16(c)?),
        InstructionTypeNum::goto_ => InstructionInfo::goto_(read_i16(c)?),
        InstructionTypeNum::goto_w => InstructionInfo::goto_w(read_i32(c)?),
        InstructionTypeNum::i2b => InstructionInfo::i2b,
        InstructionTypeNum::i2c => InstructionInfo::i2c,
        InstructionTypeNum::i2d => InstructionInfo::i2d,
        InstructionTypeNum::i2f => InstructionInfo::i2f,
        InstructionTypeNum::i2l => InstructionInfo::i2l,
        InstructionTypeNum::i2s => InstructionInfo::i2s,
        InstructionTypeNum::iadd => InstructionInfo::iadd,
        InstructionTypeNum::iaload => InstructionInfo::iaload,
        InstructionTypeNum::iand => InstructionInfo::iand,
        InstructionTypeNum::iastore => InstructionInfo::iastore,
        InstructionTypeNum::iconst_m1 => InstructionInfo::iconst_m1,
        InstructionTypeNum::iconst_0 => InstructionInfo::iconst_0,
        InstructionTypeNum::iconst_1 => InstructionInfo::iconst_1,
        InstructionTypeNum::iconst_2 => InstructionInfo::iconst_2,
        InstructionTypeNum::iconst_3 => InstructionInfo::iconst_3,
        InstructionTypeNum::iconst_4 => InstructionInfo::iconst_4,
        InstructionTypeNum::iconst_5 => InstructionInfo::iconst_5,
        InstructionTypeNum::idiv => InstructionInfo::idiv,
        InstructionTypeNum::if_acmpeq => InstructionInfo::if_acmpeq(read_i16(c)?),
        InstructionTypeNum::if_acmpne => InstructionInfo::if_acmpne(read_i16(c)?),
        InstructionTypeNum::if_icmpeq => InstructionInfo::if_icmpeq(read_i16(c)?),
        InstructionTypeNum::if_icmpne => InstructionInfo::if_icmpne(read_i16(c)?),
        InstructionTypeNum::if_icmplt => InstructionInfo::if_icmplt(read_i16(c)?),
        InstructionTypeNum::if_icmpge => InstructionInfo::if_icmpge(read_i16(c)?),
        InstructionTypeNum::if_icmpgt => InstructionInfo::if_icmpgt(read_i16(c)?),
        InstructionTypeNum::if_icmple => InstructionInfo::if_icmple(read_i16(c)?),
        InstructionTypeNum::ifeq => InstructionInfo::ifeq(read_i16(c)?),
        InstructionTypeNum::ifne => InstructionInfo::ifne(read_i16(c)?),
        InstructionTypeNum::iflt => InstructionInfo::iflt(read_i16(c)?),
        InstructionTypeNum::ifge => InstructionInfo::ifge(read_i16(c)?),
        InstructionTypeNum::ifgt => InstructionInfo::ifgt(read_i16(c)?),
        InstructionTypeNum::ifle => InstructionInfo::ifle(read_i16(c)?),
        InstructionTypeNum::ifnonnull => InstructionInfo::ifnonnull(read_i16(c)?),
        InstructionTypeNum::ifnull => InstructionInfo::ifnull(read_i16(c)?),
        InstructionTypeNum::iinc => InstructionInfo::iinc(read_iinc(c)?),
        InstructionTypeNum::iload => InstructionInfo::iload(read_u8(c)?),
        InstructionTypeNum::iload_0 => InstructionInfo::iload_0,
        InstructionTypeNum::iload_1 => InstructionInfo::iload_1,
        InstructionTypeNum::iload_2 => InstructionInfo::iload_2,
        InstructionTypeNum::iload_3 => InstructionInfo::iload_3,
        InstructionTypeNum::imul => InstructionInfo::imul,
        InstructionTypeNum::ineg => InstructionInfo::ineg,
        InstructionTypeNum::instanceof => InstructionInfo::instanceof(read_u16(c)?),
        InstructionTypeNum::invokedynamic => {
            let res = InstructionInfo::invokedynamic(read_u16(c)?);
            let zero = read_u16(c)?;
            assert_eq!(zero, 0);
            res
        }
        InstructionTypeNum::invokeinterface => InstructionInfo::invokeinterface(read_invoke_interface(c)?),
        InstructionTypeNum::invokespecial => InstructionInfo::invokespecial(read_u16(c)?),
        InstructionTypeNum::invokestatic => InstructionInfo::invokestatic(read_u16(c)?),
        InstructionTypeNum::invokevirtual => InstructionInfo::invokevirtual(read_u16(c)?),
        InstructionTypeNum::ior => InstructionInfo::ior,
        InstructionTypeNum::irem => InstructionInfo::irem,
        InstructionTypeNum::ireturn => InstructionInfo::ireturn,
        InstructionTypeNum::ishl => InstructionInfo::ishl,
        InstructionTypeNum::ishr => InstructionInfo::ishr,
        InstructionTypeNum::istore => InstructionInfo::istore(read_u8(c)?),
        InstructionTypeNum::istore_0 => InstructionInfo::istore_0,
        InstructionTypeNum::istore_1 => InstructionInfo::istore_1,
        InstructionTypeNum::istore_2 => InstructionInfo::istore_2,
        InstructionTypeNum::istore_3 => InstructionInfo::istore_3,
        InstructionTypeNum::isub => InstructionInfo::isub,
        InstructionTypeNum::iushr => InstructionInfo::iushr,
        InstructionTypeNum::ixor => InstructionInfo::ixor,
        InstructionTypeNum::jsr => InstructionInfo::jsr(read_i16(c)?),
        InstructionTypeNum::jsr_w => InstructionInfo::jsr_w(read_i32(c)?),
        InstructionTypeNum::l2d => InstructionInfo::l2d,
        InstructionTypeNum::l2f => InstructionInfo::l2f,
        InstructionTypeNum::l2i => InstructionInfo::l2i,
        InstructionTypeNum::ladd => InstructionInfo::ladd,
        InstructionTypeNum::laload => InstructionInfo::laload,
        InstructionTypeNum::land => InstructionInfo::land,
        InstructionTypeNum::lastore => InstructionInfo::lastore,
        InstructionTypeNum::lcmp => InstructionInfo::lcmp,
        InstructionTypeNum::lconst_0 => InstructionInfo::lconst_0,
        InstructionTypeNum::lconst_1 => InstructionInfo::lconst_1,
        InstructionTypeNum::ldc => InstructionInfo::ldc(read_u8(c)?),
        InstructionTypeNum::ldc_w => InstructionInfo::ldc_w(read_u16(c)?),
        InstructionTypeNum::ldc2_w => InstructionInfo::ldc2_w(read_u16(c)?),
        InstructionTypeNum::ldiv => InstructionInfo::ldiv,
        InstructionTypeNum::lload => InstructionInfo::lload(read_u8(c)?),
        InstructionTypeNum::lload_0 => InstructionInfo::lload_0,
        InstructionTypeNum::lload_1 => InstructionInfo::lload_1,
        InstructionTypeNum::lload_2 => InstructionInfo::lload_2,
        InstructionTypeNum::lload_3 => InstructionInfo::lload_3,
        InstructionTypeNum::lmul => InstructionInfo::lmul,
        InstructionTypeNum::lneg => InstructionInfo::lneg,
        InstructionTypeNum::lookupswitch => InstructionInfo::lookupswitch(read_lookup_switch(c)?),
        InstructionTypeNum::lor => InstructionInfo::lor,
        InstructionTypeNum::lrem => InstructionInfo::lrem,
        InstructionTypeNum::lreturn => InstructionInfo::lreturn,
        InstructionTypeNum::lshl => InstructionInfo::lshl,
        InstructionTypeNum::lshr => InstructionInfo::lshr,
        InstructionTypeNum::lstore => InstructionInfo::lstore(read_u8(c)?),
        InstructionTypeNum::lstore_0 => InstructionInfo::lstore_0,
        InstructionTypeNum::lstore_1 => InstructionInfo::lstore_1,
        InstructionTypeNum::lstore_2 => InstructionInfo::lstore_2,
        InstructionTypeNum::lstore_3 => InstructionInfo::lstore_3,
        InstructionTypeNum::lsub => InstructionInfo::lsub,
        InstructionTypeNum::lushr => InstructionInfo::lushr,
        InstructionTypeNum::lxor => InstructionInfo::lxor,
        InstructionTypeNum::monitorenter => InstructionInfo::monitorenter,
        InstructionTypeNum::monitorexit => InstructionInfo::monitorexit,
        InstructionTypeNum::multianewarray => InstructionInfo::multianewarray(read_multi_new_array(c)?),
        InstructionTypeNum::new => InstructionInfo::new(read_u16(c)?),
        InstructionTypeNum::newarray => InstructionInfo::newarray(read_atype(c)?),
        InstructionTypeNum::nop => InstructionInfo::nop,
        InstructionTypeNum::pop => InstructionInfo::pop,
        InstructionTypeNum::pop2 => InstructionInfo::pop2,
        InstructionTypeNum::putfield => InstructionInfo::putfield(read_u16(c)?),
        InstructionTypeNum::putstatic => InstructionInfo::putstatic(read_u16(c)?),
        InstructionTypeNum::ret => InstructionInfo::ret(read_u8(c)?),
        InstructionTypeNum::return_ => InstructionInfo::return_,
        InstructionTypeNum::saload => InstructionInfo::saload,
        InstructionTypeNum::sastore => InstructionInfo::sastore,
        InstructionTypeNum::sipush => InstructionInfo::sipush(read_i16(c)?),
        InstructionTypeNum::swap => InstructionInfo::swap,
        InstructionTypeNum::tableswitch => InstructionInfo::tableswitch(read_table_switch(c)?),
        InstructionTypeNum::wide => InstructionInfo::wide(read_wide(c)?),
    })
}

fn parse_code_impl(c: &mut CodeParserContext) -> Result<Vec<Instruction>, ClassfileParsingError> {
    let mut res = vec![];
    loop {
        let offset = c.offset;
        let instruction_option = parse_instruction(c);
        match instruction_option {
            Ok(instruction) => res.push(Instruction { offset, size: (c.offset.0 - offset.0) as u16, instruction }),
            Err(ClassfileParsingError::EndOfInstructions) => {
                break;
            }
            Err(_) => {}
        }
    }
    Ok(res)
}
