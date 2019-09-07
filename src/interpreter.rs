use std::mem::size_of;

use num::Integer;

#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum InstructionType {
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

pub struct InterpreterState {
    pub local_vars: Vec<u32>,
    pub operand_stack: Vec<u64>,
}

pub fn read_opcode(b: u8) -> InstructionType {
    return unsafe { ::std::mem::transmute(b) };
}

const EXECUTION_ERROR: &str = "Fatal Error, when executing, this is a bug.";

macro_rules! null_pointer_check {
    ($var_name:ident) => {
        if $var_name == 0 {
                unimplemented!("handle null pointers exceptions")
            }
    };
}

macro_rules! array_out_of_bounds_check {
    ($index:ident,$array_length:ident) => {if $index >= $array_length {
            unimplemented!("handle array out of bounds exceptions")
        }};
}

macro_rules! load {
    ($type_:ident,$state:ident) => {
        let index = $state.operand_stack.pop().expect(EXECUTION_ERROR);
        let array_ref = $state.operand_stack.pop().expect(EXECUTION_ERROR);
        null_pointer_check!(array_ref);
        let array_elem:$type_ = unsafe {
            let array_64: *mut u64 = ::std::mem::transmute(array_ref);
            let array_length: u64 = *array_64.offset(-1);
            let array_type:* mut $type_ = array_ref as * mut $type_;
            array_out_of_bounds_check!(index,array_length);
            *(array_type.offset(index as isize)) as $type_
        };
        $state.operand_stack.push(array_elem as u64);
    };
}

macro_rules! store {
    ($type_:ident,$state:ident) => {
        let value : $type_= $state.operand_stack.pop().expect(EXECUTION_ERROR) as $type_;
        let index = $state.operand_stack.pop().expect(EXECUTION_ERROR);
        let array_ref = $state.operand_stack.pop().expect(EXECUTION_ERROR);
        null_pointer_check!(array_ref);
        unsafe {
            let array: *mut u64 = ::std::mem::transmute(array_ref);
            let array_length: u64 = *array.offset(-1);
            array_out_of_bounds_check!(index,array_length);
            let array_type : *mut $type_ = array_ref as *mut $type_;
            *(array_type.offset(index as isize)) = value;
        }
    };
}

pub fn do_instruction(code: &[u8], state: &mut InterpreterState) {
    let opcode = read_opcode(code[0]);
    match opcode {
        InstructionType::aaload => { load!(u64,state); }/*do_aaload(state)*/
        InstructionType::aastore => { store!(u64,state); }/*do_aastore(state)*/
        InstructionType::aconst_null => { state.operand_stack.push(0 as u64) }
        InstructionType::aload => {
            let var_index = code[1];
            load_n_64(state,var_index as u64);
            unimplemented!("Need to increase pc by 2")
        }
        InstructionType::aload_0 => { load_n_64(state, 0) }
        InstructionType::aload_1 => { load_n_64(state, 1) }
        InstructionType::aload_2 => { load_n_64(state, 2) }
        InstructionType::aload_3 => { load_n_64(state, 3) }
        InstructionType::anewarray => {
            let indexbyte1 = code[1] as u16;
            let indexbyte2 = code[2] as u16;
            (indexbyte1 << 8) | indexbyte2;
            let count = state.operand_stack.pop().expect(EXECUTION_ERROR);
            unimplemented!("Need to figure out how to get the constant pool in here.");
            unimplemented!("Need to increase pc by 3");
        }
        InstructionType::areturn => {
            unimplemented!("Need to figure out function calls/returning from functions.");
        }
        InstructionType::arraylength => do_arraylength(state),
        InstructionType::astore => {
            let index = code[1];
            store_n_32(state,index as u64);
            unimplemented!("Need to increase pc by 2");
        }
        InstructionType::astore_0 => { store_n_64(state, 0) }
        InstructionType::astore_1 => { store_n_64(state, 1) }
        InstructionType::astore_2 => { store_n_64(state, 2) }
        InstructionType::astore_3 => { store_n_64(state, 3) }
        InstructionType::athrow => { unimplemented!("Need to pass in  exception handlers somehow"); }
        InstructionType::baload => { load!(u8,state); }
        InstructionType::bastore => { store!(u8,state); }
        InstructionType::bipush => {
            let byte = state.operand_stack.pop().expect(EXECUTION_ERROR) as i8;
            state.operand_stack.push(byte as i64 as u64);
        }
        InstructionType::caload => { load!(u16,state); }
        InstructionType::castore => { store!(u16,state); }
        InstructionType::checkcast => {
            unimplemented!("Need to increase pc by 3 and get constant pool involved");
        }
        InstructionType::d2f => {
            let double = pop_as_double(state);
            let converted_to_float = unsafe {
                let converted: u64 = ::std::mem::transmute(double as f32 as f64);
                converted
            };
            state.operand_stack.push(converted_to_float);
        }
        InstructionType::d2i => {
            let double = pop_as_double(state);
            state.operand_stack.push(double as u32 as u64)
        }
        InstructionType::d2l => {
            let double = pop_as_double(state);
            state.operand_stack.push(double as u64)
        }
        InstructionType::dadd => {
            let a = pop_as_double(state);
            let b = pop_as_double(state);
            let sum = a + b;
            state.operand_stack.push(unsafe { ::std::mem::transmute(sum) })
        }
        InstructionType::daload => {
            load!(f64,state);
        }
        InstructionType::dastore => {
            store!(f64,state);
        }
        InstructionType::dcmpg => {
            unimplemented!("This one is kinda annoying to implement for now")
        }
        InstructionType::dcmpl => {
            unimplemented!("This one is also kinda annoying to implement for now")
        }
        InstructionType::dconst_0 => {
            push_double(0.0, state)
        }
        InstructionType::dconst_1 => {
            push_double(1.0, state)
        }
        InstructionType::ddiv => {
            let bottom = pop_as_double(state);
            let top = pop_as_double(state);
            push_double(bottom / top, state)
        }
        InstructionType::dload => {
            let var_index = code[1];
            load_n_64(state,var_index as u64);
            unimplemented!("Need to increase pc by 2")
        }
        InstructionType::dload_0 => {
            load_n_64(state,0);
        }
        InstructionType::dload_1 => {
            load_n_64(state,1);
        }
        InstructionType::dload_2 => {
            load_n_64(state,2);
        }
        InstructionType::dload_3 => {
            load_n_64(state,3);
        }
        InstructionType::dmul => {
            let a = pop_as_double(state);
            let b = pop_as_double(state);
            push_double(a * b, state);
        }
        InstructionType::dneg => {
            let a = pop_as_double(state);
            push_double(-1.0 * a, state);
        }
        InstructionType::drem => {
            let a = pop_as_double(state);
            let b = pop_as_double(state);
            push_double(a % b, state);//todo not sure if that is correct since rem is non-standard in java
        }
        InstructionType::dreturn => {
            unimplemented!("need to figure out functions")
        }
        InstructionType::dstore => {
            let var_index = code[1];
            store_n_64(state,var_index as u64);
            unimplemented!("Need to increase pc by 2")
        }
        InstructionType::dstore_0 => {
            store_n_64(state,0);
        }
        InstructionType::dstore_1 => {
            store_n_64(state,1);
        }
        InstructionType::dstore_2 => {
            store_n_64(state,2);
        }
        InstructionType::dstore_3 => {
            store_n_64(state,3);
        }
        InstructionType::dsub => {
            let value2 = pop_as_double(state);
            let value1 = pop_as_double(state);
            push_double(value1 - value2, state);
        }
        InstructionType::dup => {
            let to_dup = state.operand_stack.pop().expect(EXECUTION_ERROR);
            state.operand_stack.push(to_dup);
            state.operand_stack.push(to_dup);
        }
        InstructionType::dup_x1 => {
            let value1 = state.operand_stack.pop().expect(EXECUTION_ERROR);
            let value2 = state.operand_stack.pop().expect(EXECUTION_ERROR);
            state.operand_stack.push(value1);
            state.operand_stack.push(value2);
            state.operand_stack.push(value1);
        }
        InstructionType::dup_x2 => {
            unimplemented!("Need to get typeinfo in here, to determine which type of operation to do and/or refactor the operand stack")
        }
        InstructionType::dup2 => {
            unimplemented!("Need to get typeinfo in here, to determine which type of operation to do and/or refactor the operand stack")
        }
        InstructionType::dup2_x1 => {
            unimplemented!("Need to get typeinfo in here, to determine which type of operation to do and/or refactor the operand stack")
        }
        InstructionType::dup2_x2 => {
            unimplemented!("Need to get typeinfo in here, to determine which type of operation to do and/or refactor the operand stack")
        }
        InstructionType::f2d => {
            let float = pop_as_float(state);
            push_double(float as f64,state);
        }
        InstructionType::f2i => {
            let float = pop_as_float(state);
            state.operand_stack.push(float as u32 as u64);
        }
        InstructionType::f2l => {
            let float = pop_as_float(state);
            state.operand_stack.push(float as u64);
        }
        InstructionType::fadd => {}
        InstructionType::faload => {}
        InstructionType::fastore => {}
        InstructionType::fcmpg => {
            unimplemented!("This one is kinda annoying to implement for now")
        }
        InstructionType::fcmpl => {
            unimplemented!("This one is kinda annoying to implement for now")
        }
        InstructionType::fconst_0 => {}
        InstructionType::fconst_1 => {}
        InstructionType::fconst_2 => {}
        InstructionType::fdiv => {}
        InstructionType::fload => {}
        InstructionType::fload_0 => {}
        InstructionType::fload_1 => {}
        InstructionType::fload_2 => {}
        InstructionType::fload_3 => {}
        InstructionType::fmul => {}
        InstructionType::fneg => {}
        InstructionType::frem => {}
        InstructionType::freturn => {}
        InstructionType::fstore => {}
        InstructionType::fstore_0 => {}
        InstructionType::fstore_1 => {}
        InstructionType::fstore_2 => {}
        InstructionType::fstore_3 => {}
        InstructionType::fsub => {}
        InstructionType::getfield => {}
        InstructionType::getstatic => {}
        InstructionType::goto_ => {}
        InstructionType::goto_w => {}
        InstructionType::i2b => {}
        InstructionType::i2c => {}
        InstructionType::i2d => {}
        InstructionType::i2f => {}
        InstructionType::i2l => {}
        InstructionType::i2s => {}
        InstructionType::iadd => {}
        InstructionType::iaload => {}
        InstructionType::iand => {}
        InstructionType::iastore => {}
        InstructionType::iconst_m1 => {}
        InstructionType::iconst_0 => {}
        InstructionType::iconst_1 => {}
        InstructionType::iconst_2 => {}
        InstructionType::iconst_3 => {}
        InstructionType::iconst_4 => {}
        InstructionType::iconst_5 => {}
        InstructionType::idiv => {}
        InstructionType::if_acmpeq => {}
        InstructionType::if_acmpne => {}
        InstructionType::if_icmpeq => {}
        InstructionType::if_icmpne => {}
        InstructionType::if_icmplt => {}
        InstructionType::if_icmpge => {}
        InstructionType::if_icmpgt => {}
        InstructionType::if_icmple => {}
        InstructionType::ifeq => {}
        InstructionType::ifne => {}
        InstructionType::iflt => {}
        InstructionType::ifge => {}
        InstructionType::ifgt => {}
        InstructionType::ifle => {}
        InstructionType::ifnonnull => {}
        InstructionType::ifnull => {}
        InstructionType::iinc => {}
        InstructionType::iload => {}
        InstructionType::iload_0 => {}
        InstructionType::iload_1 => {}
        InstructionType::iload_2 => {}
        InstructionType::iload_3 => {}
        InstructionType::imul => {}
        InstructionType::ineg => {}
        InstructionType::instanceof => {}
        InstructionType::invokedynamic => {}
        InstructionType::invokeinterface => {}
        InstructionType::invokespecial => {}
        InstructionType::invokestatic => {}
        InstructionType::invokevirtual => {}
        InstructionType::ior => {}
        InstructionType::irem => {}
        InstructionType::ireturn => {}
        InstructionType::ishl => {}
        InstructionType::ishr => {}
        InstructionType::istore => {}
        InstructionType::istore_0 => {}
        InstructionType::istore_1 => {}
        InstructionType::istore_2 => {}
        InstructionType::istore_3 => {}
        InstructionType::isub => {}
        InstructionType::iushr => {}
        InstructionType::ixor => {}
        InstructionType::jsr => {}
        InstructionType::jsr_w => {}
        InstructionType::l2d => {}
        InstructionType::l2f => {}
        InstructionType::l2i => {}
        InstructionType::ladd => {}
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
        InstructionType::wide => {}
    }
}


fn push_float(to_push: f32, state: &mut InterpreterState) {
    state.operand_stack.push(unsafe { ::std::mem::transmute(to_push as u64) })
}

fn pop_as_float(state: &mut InterpreterState) -> f32 {
    let value = state.operand_stack.pop().expect(EXECUTION_ERROR) as u32;
    return unsafe {
        ::std::mem::transmute(value)
    }
}


fn push_double(to_push: f64, state: &mut InterpreterState) {
    state.operand_stack.push(unsafe { ::std::mem::transmute(to_push) })
}

fn pop_as_double(state: &mut InterpreterState) -> f64 {
    let value = state.operand_stack.pop().expect(EXECUTION_ERROR);
    return unsafe {
        ::std::mem::transmute(value)
    }
}

fn store_n_32(state: &mut InterpreterState, n: u64) {
    let reference = state.operand_stack.pop().expect(EXECUTION_ERROR);
    state.local_vars[n as usize] = reference as u32;
}


fn store_n_64(state: &mut InterpreterState, n: u64) {
    let reference = state.operand_stack.pop().expect(EXECUTION_ERROR);
    state.local_vars[n as usize] = reference as u32;
    state.local_vars[(n + 1) as usize] = (reference >> 32) as u32;
}

fn load_n_32(state: &mut InterpreterState, n: u64) {
    let reference = state.local_vars[n as usize];
    state.operand_stack.push(reference as u64)
}

fn load_n_64(state: &mut InterpreterState, n: u64) {
    let least_significant = state.local_vars[n as usize] as u64;
    let most_significant = state.local_vars[(n + 1) as usize] as u64;
    state.operand_stack.push((most_significant << 32) | least_significant)
}


fn do_arraylength(state: &mut InterpreterState) -> () {
    let array_ref = state.operand_stack.pop().expect(EXECUTION_ERROR);
    let length = unsafe {
        let array: *mut u64 = ::std::mem::transmute(array_ref);
        *(array.offset(-1 as isize)) as u64
    };
    state.operand_stack.push(length)
}

fn do_aastore(state: &mut InterpreterState) -> () {
    let value = state.operand_stack.pop().expect(EXECUTION_ERROR);
    let index = state.operand_stack.pop().expect(EXECUTION_ERROR);
    let array_ref = state.operand_stack.pop().expect(EXECUTION_ERROR);
    null_pointer_check!(array_ref);
    unsafe {
        let array: *mut u64 = ::std::mem::transmute(array_ref);
        let array_length: u64 = *array.offset(-1);
        array_out_of_bounds_check!(index,array_length);
        *(array.offset(index as isize)) = value;
    }
}

//fn load<Type>(state: &mut InterpreterState) -> () where Type : Integer{
//    let index = state.operand_stack.pop().expect(EXECUTION_ERROR);
//    let array_ref = state.operand_stack.pop().expect(EXECUTION_ERROR);
//    null_pointer_check!(array_ref);
//    let array_elem:Type = unsafe {
//        let array_64: *mut u64 = ::std::mem::transmute(array_ref);
//        let array_length: u64 = *array_64.offset(-1);
//        let array_type:* mut Type = array_ref as * mut Type;
//        array_out_of_bounds_check!(index,array_length);
//        *(array_type.offset(index as isize)) as Type
//    };
//    state.operand_stack.push(array_elem as u64);
//}

fn do_aaload(state: &mut InterpreterState) -> () {
    let index = state.operand_stack.pop().expect(EXECUTION_ERROR);
    let array_ref = state.operand_stack.pop().expect(EXECUTION_ERROR);
    null_pointer_check!(array_ref);
    let array_elem = unsafe {
        let array: *mut u64 = ::std::mem::transmute(array_ref);
        let array_length: u64 = *array.offset(-1);
        array_out_of_bounds_check!(index,array_length);
        *(array.offset(index as isize)) as u64
    };
    state.operand_stack.push(array_elem);
}
