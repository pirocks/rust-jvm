use std::slice::Iter;
use rust_jvm_common::classfile::{IInc, InvokeInterface, MultiNewArray, LookupSwitch, TableSwitch, Atype, Wide, Instruction, InstructionInfo};


fn read_iinc(c: &mut CodeParserContext) -> Option<IInc> {
    let index = read_u8(c)?;
    let const_ = read_i8(c)?;
    return Some(IInc { index, const_ });
}

fn read_invoke_interface(c: &mut CodeParserContext) -> Option<InvokeInterface> {
    let index = read_u16(c)?;
    let count = read_u8(c)?;
    assert_ne!(count, 0);
    let zero = read_u8(c)?;
    assert_eq!(zero, 0);
    return Some(InvokeInterface { index, count });
}


fn read_lookup_switch(c: &mut CodeParserContext) -> Option<LookupSwitch> {
    while !(c.offset % 4 == 0) {
        let padding = read_u8(c);
        padding.expect("Unexpected end of code");
        dbg!(padding);
    };
//    dbg!(c.offset);
    let default = read_i32(c).unwrap();
//    dbg!(default);
    let npairs = read_i32(c).unwrap();
    assert!(npairs > 0);
    let mut pairs = vec![];
//    dbg!(npairs);
    for _ in 0..npairs {
        //key target
        pairs.push((read_i32(c).unwrap(), read_i32(c).unwrap()));
    }
    return Some(LookupSwitch {
        default,
        pairs,
    });
}


fn read_multi_new_array(c: &mut CodeParserContext) -> Option<MultiNewArray> {
    let index = read_u16(c)?;
    let dims = read_u8(c)?;
    Some(MultiNewArray { index, dims })
}


fn read_atype(c: &mut CodeParserContext) -> Option<Atype> {
    return Some(unsafe { ::std::mem::transmute(read_u8(c)?) });
}


fn read_table_switch(c: &mut CodeParserContext) -> Option<TableSwitch> {
    while !(c.offset % 4 == 0) {
        read_u8(c).expect("Uneexpected end of code");
    };
    let default = read_i32(c)?;
    let low = read_i32(c)?;
    let high = read_i32(c)?;//technically these should all be expects
    let num_to_read = high - low + 1;
    let mut offsets = vec![];
    for _ in 0..num_to_read {
        offsets.push(read_i32(c)?);
    }
    return Some(TableSwitch { default, low, high, offsets });
}


fn read_wide(_c: &mut CodeParserContext) -> Option<Wide> {
    unimplemented!();
}


pub struct CodeParserContext<'l> {
    pub offset: usize,
    pub iter: Iter<'l, u8>,
}

pub fn parse_code_raw(raw: &[u8]) -> Vec<Instruction> {
    //is this offset of 0 even correct?
    // what if code starts at non-aligned?
    let mut c = CodeParserContext { iter: raw.iter(), offset: 0 };
    return parse_code_impl(&mut c);
}

fn read_u8(c: &mut CodeParserContext) -> Option<u8> {
    c.offset += 1;
    let next = c.iter.next();
//    match next{
//        None => {
//            dbg!(&c.offset);
//            dbg!(&c.iter);
//        },
//        Some(_) => {},
//    }
    return Some(*next?);

}

fn read_i8(c: &mut CodeParserContext) -> Option<i8> {
    return Some(unsafe { ::std::mem::transmute(read_u8(c)?) });
}

fn read_u16(c: &mut CodeParserContext) -> Option<u16> {
    let byte1 = read_u8(c)? as u16;
    let byte2 = read_u8(c)? as u16;
    return Some(byte1 << 8 | byte2);
}

fn read_i16(c: &mut CodeParserContext) -> Option<i16> {
    return Some(unsafe { ::std::mem::transmute(read_u16(c)?) });
}

fn read_u32(c: &mut CodeParserContext) -> Option<u32> {
    let byte1 = read_u8(c)? as u32;
    let byte2 = read_u8(c)? as u32;
    let byte3 = read_u8(c)? as u32;
    let byte4 = read_u8(c)? as u32;
    return Some(byte1 << 24 | byte2 << 16 | byte3 << 8 | byte4);
}

fn read_i32(c: &mut CodeParserContext) -> Option<i32> {
    return Some(unsafe { ::std::mem::transmute(read_u32(c)?) });
}

pub fn read_opcode(b: u8) -> InstructionTypeNum {
    return unsafe { ::std::mem::transmute(b) };
}


#[allow(non_camel_case_types)]
#[repr(u8)]
#[derive(Debug)]
pub enum InstructionTypeNum {
    aaload = 50,
    aastore = 83,
    aconst_null = 1,
    aload = 25,
    aload_0 = 42,
    aload_1 = 43,
    aload_2 = 44,
    aload_3 = 45,
    anewarray = 189,
    areturn = 176,
    arraylength = 190,
    astore = 58,
    astore_0 = 75,
    astore_1 = 76,
    astore_2 = 77,
    astore_3 = 78,
    athrow = 191,
    baload = 51,
    bastore = 84,
    bipush = 16,
    caload = 52,
    castore = 85,
    checkcast = 192,
    d2f = 144,
    d2i = 142,
    d2l = 143,
    dadd = 99,
    daload = 49,
    dastore = 82,
    dcmpg = 152,
    dcmpl = 151,
    dconst_0 = 14,
    dconst_1 = 15,
    ddiv = 111,
    dload = 24,
    dload_0 = 38,
    dload_1 = 39,
    dload_2 = 40,
    dload_3 = 41,
    dmul = 107,
    dneg = 119,
    drem = 115,
    dreturn = 175,
    dstore = 57,
    dstore_0 = 71,
    dstore_1 = 72,
    dstore_2 = 73,
    dstore_3 = 74,
    dsub = 103,
    dup = 89,
    dup_x1 = 90,
    dup_x2 = 91,
    dup2 = 92,
    dup2_x1 = 93,
    dup2_x2 = 94,
    f2d = 141,
    f2i = 139,
    f2l = 140,
    fadd = 98,
    faload = 48,
    fastore = 81,
    fcmpg = 150,
    fcmpl = 149,
    fconst_0 = 11,
    fconst_1 = 12,
    fconst_2 = 13,
    fdiv = 110,
    fload = 23,
    fload_0 = 34,
    fload_1 = 35,
    fload_2 = 36,
    fload_3 = 37,
    fmul = 106,
    fneg = 118,
    frem = 114,
    freturn = 174,
    fstore = 56,
    fstore_0 = 67,
    fstore_1 = 68,
    fstore_2 = 69,
    fstore_3 = 70,
    fsub = 102,
    getfield = 180,
    getstatic = 178,
    goto_ = 167,
    goto_w = 200,
    i2b = 145,
    i2c = 146,
    i2d = 135,
    i2f = 134,
    i2l = 133,
    i2s = 147,
    iadd = 96,
    iaload = 46,
    iand = 126,
    iastore = 79,
    iconst_m1 = 2,
    iconst_0 = 3,
    iconst_1 = 4,
    iconst_2 = 5,
    iconst_3 = 6,
    iconst_4 = 7,
    iconst_5 = 8,
    idiv = 108,
    if_acmpeq = 165,
    if_acmpne = 166,
    if_icmpeq = 159,
    if_icmpne = 160,
    if_icmplt = 161,
    if_icmpge = 162,
    if_icmpgt = 163,
    if_icmple = 164,
    ifeq = 153,
    ifne = 154,
    iflt = 155,
    ifge = 156,
    ifgt = 157,
    ifle = 158,
    ifnonnull = 199,
    ifnull = 198,
    iinc = 132,
    iload = 21,
    iload_0 = 26,
    iload_1 = 27,
    iload_2 = 28,
    iload_3 = 29,
    imul = 104,
    ineg = 116,
    instanceof = 193,
    invokedynamic = 186,
    invokeinterface = 185,
    invokespecial = 183,
    invokestatic = 184,
    invokevirtual = 182,
    ior = 128,
    irem = 112,
    ireturn = 172,
    ishl = 120,
    ishr = 122,
    istore = 54,
    istore_0 = 59,
    istore_1 = 60,
    istore_2 = 61,
    istore_3 = 62,
    isub = 100,
    iushr = 124,
    ixor = 130,
    jsr = 168,
    jsr_w = 201,
    l2d = 138,
    l2f = 137,
    l2i = 136,
    ladd = 97,
    laload = 47,
    land = 127,
    lastore = 80,
    lcmp = 148,
    lconst_0 = 9,
    lconst_1 = 10,
    ldc = 18,
    ldc_w = 19,
    ldc2_w = 20,
    ldiv = 109,
    lload = 22,
    lload_0 = 30,
    lload_1 = 31,
    lload_2 = 32,
    lload_3 = 33,
    lmul = 105,
    lneg = 117,
    lookupswitch = 171,
    lor = 129,
    lrem = 113,
    lreturn = 173,
    lshl = 121,
    lshr = 123,
    lstore = 55,
    lstore_0 = 63,
    lstore_1 = 64,
    lstore_2 = 65,
    lstore_3 = 66,
    lsub = 101,
    lushr = 125,
    lxor = 131,
    monitorenter = 194,
    monitorexit = 195,
    multianewarray = 197,
    new = 187,
    newarray = 188,
    nop = 0,
    pop = 87,
    pop2 = 88,
    putfield = 181,
    putstatic = 179,
    ret = 169,
    return_ = 177,
    saload = 53,
    sastore = 86,
    sipush = 17,
    swap = 95,
    tableswitch = 170,
    wide = 196,
}

pub fn parse_instruction(c: &mut CodeParserContext) -> Option<InstructionInfo> {
    let opcode = read_opcode(read_u8(c)?);
    Some(match opcode {
        InstructionTypeNum::aaload => { InstructionInfo::aaload }
        InstructionTypeNum::aastore => { InstructionInfo::aastore }
        InstructionTypeNum::aconst_null => { InstructionInfo::aconst_null }
        InstructionTypeNum::aload => { InstructionInfo::aload(read_u8(c).unwrap()) }
        InstructionTypeNum::aload_0 => { InstructionInfo::aload_0 }
        InstructionTypeNum::aload_1 => { InstructionInfo::aload_1 }
        InstructionTypeNum::aload_2 => { InstructionInfo::aload_2 }
        InstructionTypeNum::aload_3 => { InstructionInfo::aload_3 }
        InstructionTypeNum::anewarray => { InstructionInfo::anewarray(read_u16(c).unwrap()) }
        InstructionTypeNum::areturn => { InstructionInfo::areturn }
        InstructionTypeNum::arraylength => { InstructionInfo::arraylength }
        InstructionTypeNum::astore => { InstructionInfo::astore(read_u8(c).unwrap()) }
        InstructionTypeNum::astore_0 => { InstructionInfo::astore_0 }
        InstructionTypeNum::astore_1 => { InstructionInfo::astore_1 }
        InstructionTypeNum::astore_2 => { InstructionInfo::astore_2 }
        InstructionTypeNum::astore_3 => { InstructionInfo::astore_3 }
        InstructionTypeNum::athrow => { InstructionInfo::athrow }
        InstructionTypeNum::baload => { InstructionInfo::baload }
        InstructionTypeNum::bastore => { InstructionInfo::bastore }
        InstructionTypeNum::bipush => { InstructionInfo::bipush(read_u8(c).unwrap()) }
        InstructionTypeNum::caload => { InstructionInfo::caload }
        InstructionTypeNum::castore => { InstructionInfo::castore }
        InstructionTypeNum::checkcast => { InstructionInfo::checkcast(read_u16(c).unwrap()) }
        InstructionTypeNum::d2f => { InstructionInfo::d2f }
        InstructionTypeNum::d2i => { InstructionInfo::d2i }
        InstructionTypeNum::d2l => { InstructionInfo::d2l }
        InstructionTypeNum::dadd => { InstructionInfo::dadd }
        InstructionTypeNum::daload => { InstructionInfo::daload }
        InstructionTypeNum::dastore => { InstructionInfo::dastore }
        InstructionTypeNum::dcmpg => { InstructionInfo::dcmpg }
        InstructionTypeNum::dcmpl => { InstructionInfo::dcmpl }
        InstructionTypeNum::dconst_0 => { InstructionInfo::dconst_0 }
        InstructionTypeNum::dconst_1 => { InstructionInfo::dconst_1 }
        InstructionTypeNum::ddiv => { InstructionInfo::ddiv }
        InstructionTypeNum::dload => { InstructionInfo::dload(read_u8(c).unwrap()) }
        InstructionTypeNum::dload_0 => { InstructionInfo::dload_0 }
        InstructionTypeNum::dload_1 => { InstructionInfo::dload_1 }
        InstructionTypeNum::dload_2 => { InstructionInfo::dload_2 }
        InstructionTypeNum::dload_3 => { InstructionInfo::dload_3 }
        InstructionTypeNum::dmul => { InstructionInfo::dmul }
        InstructionTypeNum::dneg => { InstructionInfo::dneg }
        InstructionTypeNum::drem => { InstructionInfo::drem }
        InstructionTypeNum::dreturn => { InstructionInfo::dreturn }
        InstructionTypeNum::dstore => { InstructionInfo::dstore(read_u8(c).unwrap()) }
        InstructionTypeNum::dstore_0 => { InstructionInfo::dstore_0 }
        InstructionTypeNum::dstore_1 => { InstructionInfo::dstore_1 }
        InstructionTypeNum::dstore_2 => { InstructionInfo::dstore_2 }
        InstructionTypeNum::dstore_3 => { InstructionInfo::dstore_3 }
        InstructionTypeNum::dsub => { InstructionInfo::dsub }
        InstructionTypeNum::dup => { InstructionInfo::dup }
        InstructionTypeNum::dup_x1 => { InstructionInfo::dup_x1 }
        InstructionTypeNum::dup_x2 => { InstructionInfo::dup_x2 }
        InstructionTypeNum::dup2 => { InstructionInfo::dup2 }
        InstructionTypeNum::dup2_x1 => { InstructionInfo::dup2_x1 }
        InstructionTypeNum::dup2_x2 => { InstructionInfo::dup2_x2 }
        InstructionTypeNum::f2d => { InstructionInfo::f2d }
        InstructionTypeNum::f2i => { InstructionInfo::f2i }
        InstructionTypeNum::f2l => { InstructionInfo::f2l }
        InstructionTypeNum::fadd => { InstructionInfo::fadd }
        InstructionTypeNum::faload => { InstructionInfo::faload }
        InstructionTypeNum::fastore => { InstructionInfo::fastore }
        InstructionTypeNum::fcmpg => { InstructionInfo::fcmpg }
        InstructionTypeNum::fcmpl => { InstructionInfo::fcmpl }
        InstructionTypeNum::fconst_0 => { InstructionInfo::fconst_0 }
        InstructionTypeNum::fconst_1 => { InstructionInfo::fconst_1 }
        InstructionTypeNum::fconst_2 => { InstructionInfo::fconst_2 }
        InstructionTypeNum::fdiv => { InstructionInfo::fdiv }
        InstructionTypeNum::fload => { InstructionInfo::fload(read_u8(c).unwrap()) }
        InstructionTypeNum::fload_0 => { InstructionInfo::fload_0 }
        InstructionTypeNum::fload_1 => { InstructionInfo::fload_1 }
        InstructionTypeNum::fload_2 => { InstructionInfo::fload_2 }
        InstructionTypeNum::fload_3 => { InstructionInfo::fload_3 }
        InstructionTypeNum::fmul => { InstructionInfo::fmul }
        InstructionTypeNum::fneg => { InstructionInfo::fneg }
        InstructionTypeNum::frem => { InstructionInfo::frem }
        InstructionTypeNum::freturn => { InstructionInfo::freturn }
        InstructionTypeNum::fstore => { InstructionInfo::fstore(read_u8(c).unwrap()) }
        InstructionTypeNum::fstore_0 => { InstructionInfo::fstore_0 }
        InstructionTypeNum::fstore_1 => { InstructionInfo::fstore_1 }
        InstructionTypeNum::fstore_2 => { InstructionInfo::fstore_2 }
        InstructionTypeNum::fstore_3 => { InstructionInfo::fstore_3 }
        InstructionTypeNum::fsub => { InstructionInfo::fsub }
        InstructionTypeNum::getfield => { InstructionInfo::getfield(read_u16(c).unwrap()) }
        InstructionTypeNum::getstatic => { InstructionInfo::getstatic(read_u16(c).unwrap()) }
        InstructionTypeNum::goto_ => { InstructionInfo::goto_(read_i16(c).unwrap()) }
        InstructionTypeNum::goto_w => { InstructionInfo::goto_w(read_i32(c).unwrap()) }
        InstructionTypeNum::i2b => { InstructionInfo::i2b }
        InstructionTypeNum::i2c => { InstructionInfo::i2c }
        InstructionTypeNum::i2d => { InstructionInfo::i2d }
        InstructionTypeNum::i2f => { InstructionInfo::i2f }
        InstructionTypeNum::i2l => { InstructionInfo::i2l }
        InstructionTypeNum::i2s => { InstructionInfo::i2s }
        InstructionTypeNum::iadd => { InstructionInfo::iadd }
        InstructionTypeNum::iaload => { InstructionInfo::iaload }
        InstructionTypeNum::iand => { InstructionInfo::iand }
        InstructionTypeNum::iastore => { InstructionInfo::iastore }
        InstructionTypeNum::iconst_m1 => { InstructionInfo::iconst_m1 }
        InstructionTypeNum::iconst_0 => { InstructionInfo::iconst_0 }
        InstructionTypeNum::iconst_1 => { InstructionInfo::iconst_1 }
        InstructionTypeNum::iconst_2 => { InstructionInfo::iconst_2 }
        InstructionTypeNum::iconst_3 => { InstructionInfo::iconst_3 }
        InstructionTypeNum::iconst_4 => { InstructionInfo::iconst_4 }
        InstructionTypeNum::iconst_5 => { InstructionInfo::iconst_5 }
        InstructionTypeNum::idiv => { InstructionInfo::idiv }
        InstructionTypeNum::if_acmpeq => { InstructionInfo::if_acmpeq(read_i16(c).unwrap()) }
        InstructionTypeNum::if_acmpne => { InstructionInfo::if_acmpne(read_i16(c).unwrap()) }
        InstructionTypeNum::if_icmpeq => { InstructionInfo::if_icmpeq(read_i16(c).unwrap()) }
        InstructionTypeNum::if_icmpne => { InstructionInfo::if_icmpne(read_i16(c).unwrap()) }
        InstructionTypeNum::if_icmplt => { InstructionInfo::if_icmplt(read_i16(c).unwrap()) }
        InstructionTypeNum::if_icmpge => { InstructionInfo::if_icmpge(read_i16(c).unwrap()) }
        InstructionTypeNum::if_icmpgt => { InstructionInfo::if_icmpgt(read_i16(c).unwrap()) }
        InstructionTypeNum::if_icmple => { InstructionInfo::if_icmple(read_i16(c).unwrap()) }
        InstructionTypeNum::ifeq => { InstructionInfo::ifeq(read_i16(c).unwrap()) }
        InstructionTypeNum::ifne => { InstructionInfo::ifne(read_i16(c).unwrap()) }
        InstructionTypeNum::iflt => { InstructionInfo::iflt(read_i16(c).unwrap()) }
        InstructionTypeNum::ifge => { InstructionInfo::ifge(read_i16(c).unwrap()) }
        InstructionTypeNum::ifgt => { InstructionInfo::ifgt(read_i16(c).unwrap()) }
        InstructionTypeNum::ifle => { InstructionInfo::ifle(read_i16(c).unwrap()) }
        InstructionTypeNum::ifnonnull => { InstructionInfo::ifnonnull(read_i16(c).unwrap()) }
        InstructionTypeNum::ifnull => { InstructionInfo::ifnull(read_i16(c).unwrap()) }
        InstructionTypeNum::iinc => { InstructionInfo::iinc(read_iinc(c).unwrap()) }
        InstructionTypeNum::iload => { InstructionInfo::iload(read_u8(c).unwrap()) }
        InstructionTypeNum::iload_0 => { InstructionInfo::iload_0 }
        InstructionTypeNum::iload_1 => { InstructionInfo::iload_1 }
        InstructionTypeNum::iload_2 => { InstructionInfo::iload_2 }
        InstructionTypeNum::iload_3 => { InstructionInfo::iload_3 }
        InstructionTypeNum::imul => { InstructionInfo::imul }
        InstructionTypeNum::ineg => { InstructionInfo::ineg }
        InstructionTypeNum::instanceof => { InstructionInfo::instanceof(read_u16(c).unwrap()) }
        InstructionTypeNum::invokedynamic => {
            let res = InstructionInfo::invokedynamic(read_u16(c).unwrap());
            let zero = read_u16(c)?;
            assert_eq!(zero, 0);
            res
        }
        InstructionTypeNum::invokeinterface => { InstructionInfo::invokeinterface(read_invoke_interface(c).unwrap()) }
        InstructionTypeNum::invokespecial => { InstructionInfo::invokespecial(read_u16(c).unwrap()) }
        InstructionTypeNum::invokestatic => { InstructionInfo::invokestatic(read_u16(c).unwrap()) }
        InstructionTypeNum::invokevirtual => { InstructionInfo::invokevirtual(read_u16(c).unwrap()) }
        InstructionTypeNum::ior => { InstructionInfo::ior }
        InstructionTypeNum::irem => { InstructionInfo::irem }
        InstructionTypeNum::ireturn => { InstructionInfo::ireturn }
        InstructionTypeNum::ishl => { InstructionInfo::ishl }
        InstructionTypeNum::ishr => { InstructionInfo::ishr }
        InstructionTypeNum::istore => { InstructionInfo::istore(read_u8(c).unwrap()) }
        InstructionTypeNum::istore_0 => { InstructionInfo::istore_0 }
        InstructionTypeNum::istore_1 => { InstructionInfo::istore_1 }
        InstructionTypeNum::istore_2 => { InstructionInfo::istore_2 }
        InstructionTypeNum::istore_3 => { InstructionInfo::istore_3 }
        InstructionTypeNum::isub => { InstructionInfo::isub }
        InstructionTypeNum::iushr => { InstructionInfo::iushr }
        InstructionTypeNum::ixor => { InstructionInfo::ixor }
        InstructionTypeNum::jsr => { InstructionInfo::jsr(read_i16(c).unwrap()) }
        InstructionTypeNum::jsr_w => { InstructionInfo::jsr_w(read_i32(c).unwrap()) }
        InstructionTypeNum::l2d => { InstructionInfo::l2d }
        InstructionTypeNum::l2f => { InstructionInfo::l2f }
        InstructionTypeNum::l2i => { InstructionInfo::l2i }
        InstructionTypeNum::ladd => { InstructionInfo::ladd }
        InstructionTypeNum::laload => { InstructionInfo::laload }
        InstructionTypeNum::land => { InstructionInfo::land }
        InstructionTypeNum::lastore => { InstructionInfo::lastore }
        InstructionTypeNum::lcmp => { InstructionInfo::lcmp }
        InstructionTypeNum::lconst_0 => { InstructionInfo::lconst_0 }
        InstructionTypeNum::lconst_1 => { InstructionInfo::lconst_1 }
        InstructionTypeNum::ldc => { InstructionInfo::ldc(read_u8(c).unwrap()) }
        InstructionTypeNum::ldc_w => { InstructionInfo::ldc_w(read_u16(c).unwrap()) }
        InstructionTypeNum::ldc2_w => { InstructionInfo::ldc2_w(read_u16(c).unwrap()) }
        InstructionTypeNum::ldiv => { InstructionInfo::ldiv }
        InstructionTypeNum::lload => { InstructionInfo::lload(read_u8(c).unwrap()) }
        InstructionTypeNum::lload_0 => { InstructionInfo::lload_0 }
        InstructionTypeNum::lload_1 => { InstructionInfo::lload_1 }
        InstructionTypeNum::lload_2 => { InstructionInfo::lload_2 }
        InstructionTypeNum::lload_3 => { InstructionInfo::lload_3 }
        InstructionTypeNum::lmul => { InstructionInfo::lmul }
        InstructionTypeNum::lneg => { InstructionInfo::lneg }
        InstructionTypeNum::lookupswitch => { InstructionInfo::lookupswitch(read_lookup_switch(c).unwrap()) }
        InstructionTypeNum::lor => { InstructionInfo::lor }
        InstructionTypeNum::lrem => { InstructionInfo::lrem }
        InstructionTypeNum::lreturn => { InstructionInfo::lreturn }
        InstructionTypeNum::lshl => { InstructionInfo::lshl }
        InstructionTypeNum::lshr => { InstructionInfo::lshr }
        InstructionTypeNum::lstore => { InstructionInfo::lstore(read_u8(c).unwrap()) }
        InstructionTypeNum::lstore_0 => { InstructionInfo::lstore_0 }
        InstructionTypeNum::lstore_1 => { InstructionInfo::lstore_1 }
        InstructionTypeNum::lstore_2 => { InstructionInfo::lstore_2 }
        InstructionTypeNum::lstore_3 => { InstructionInfo::lstore_3 }
        InstructionTypeNum::lsub => { InstructionInfo::lsub }
        InstructionTypeNum::lushr => { InstructionInfo::lushr }
        InstructionTypeNum::lxor => { InstructionInfo::lxor }
        InstructionTypeNum::monitorenter => { InstructionInfo::monitorenter }
        InstructionTypeNum::monitorexit => { InstructionInfo::monitorexit }
        InstructionTypeNum::multianewarray => { InstructionInfo::multianewarray(read_multi_new_array(c).unwrap()) }
        InstructionTypeNum::new => { InstructionInfo::new(read_u16(c).unwrap()) }
        InstructionTypeNum::newarray => { InstructionInfo::newarray(read_atype(c).unwrap()) }
        InstructionTypeNum::nop => { InstructionInfo::nop }
        InstructionTypeNum::pop => { InstructionInfo::pop }
        InstructionTypeNum::pop2 => { InstructionInfo::pop2 }
        InstructionTypeNum::putfield => { InstructionInfo::putfield(read_u16(c).unwrap()) }
        InstructionTypeNum::putstatic => { InstructionInfo::putstatic(read_u16(c).unwrap()) }
        InstructionTypeNum::ret => { InstructionInfo::ret(read_u8(c).unwrap()) }
        InstructionTypeNum::return_ => { InstructionInfo::return_ }
        InstructionTypeNum::saload => { InstructionInfo::saload }
        InstructionTypeNum::sastore => { InstructionInfo::sastore }
        InstructionTypeNum::sipush => { InstructionInfo::sipush(read_u16(c).unwrap()) }
        InstructionTypeNum::swap => { InstructionInfo::swap }
        InstructionTypeNum::tableswitch => { InstructionInfo::tableswitch(read_table_switch(c).unwrap()) }
        InstructionTypeNum::wide => { InstructionInfo::wide(read_wide(c).unwrap()) }
    })
}


fn parse_code_impl(c: &mut CodeParserContext) -> Vec<Instruction> {
    let mut res = vec![];
    loop {
        let offset = c.offset;
        let instruction_option = parse_instruction(c);
        match instruction_option {
            None => { break; }
            Some(instruction) => { res.push(Instruction { offset, instruction }) }
        }
    };
    return res;
}