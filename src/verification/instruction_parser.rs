use std::io::{Error, Write, Cursor, Read};

use classfile::attribute_infos::{Code};
use classfile::Classfile;
use classfile::constant_infos::{Class, ConstantKind};
use interpreter::{InstructionType, read_opcode};
use verification::prolog_info_defs::{extract_string_from_utf8, BOOTSTRAP_LOADER_NAME, ExtraDescriptors, class_prolog_name};
use verification::types::{parse_field_descriptor, write_type_prolog};

fn name_and_type_extractor(i: u16, class_file: &Classfile) -> (String, String) {
    let nt;
    match &class_file.constant_pool[i as usize].kind {
        ConstantKind::NameAndType(nt_) => {
            nt = nt_;
        },
        _ => { panic!("Ths a bug.") }
    }
    let descriptor = extract_string_from_utf8(&class_file.constant_pool[nt.descriptor_index as usize]);
    let method_name = extract_string_from_utf8(&class_file.constant_pool[nt.name_index as usize]);
    return (method_name, descriptor)
}

pub fn extract_class_from_constant_pool(i: u16, class_file: &Classfile) -> &Class {
    match &class_file.constant_pool[i as usize].kind {
        ConstantKind::Class(c) => {
            return c;
        },
        _ => {
            panic!();
        }
    }
}

/*
fn extract_field_from_constant_pool(i: u16, class_file: &Classfile) -> &Fieldref {
    match &class_file.constant_pool[i as usize].kind {
        ConstantKind::Fieldref(f) => {
            return f;
        },
        _ => {
            panic!();
        }
    }
}
*/

/*
fn bootstrap_methods(class_file: &Classfile) -> &BootstrapMethods  {
    for attr in class_file.attributes.iter() {
        match &attr.attribute_type {
            AttributeType::BootstrapMethods(bm) => {
                return bm
            },
            _ => {panic!("No bootstrap methods found")}
        }
    }
    panic!("No bootstrap methods found");
}
*/


fn cp_elem_to_string(extra_descriptors: &mut ExtraDescriptors, class_file: &Classfile, cp_index: u16) -> String {
    let mut res = String::new();
    use std::fmt::Write;
    match &class_file.constant_pool[cp_index as usize].kind {
        ConstantKind::InvokeDynamic(i) => {
            let (method_name, descriptor) = name_and_type_extractor(i.name_and_type_index, class_file);
            write!(&mut res, "dmethod('{}', '{}')",method_name,descriptor).unwrap();
            extra_descriptors.extra_method_descriptors.push(descriptor);
        },
        ConstantKind::Methodref(m) => {
            let c = extract_class_from_constant_pool(m.class_index, class_file);
            let class_name = extract_string_from_utf8(&class_file.constant_pool[c.name_index as usize]);
            let (method_name, descriptor) = name_and_type_extractor(m.name_and_type_index, class_file);
            if class_name.chars().nth(0).unwrap() == '[' {
                let parsed_class_descriptor = parse_field_descriptor(class_name.as_str()).expect("Error parsing descriptor").field_type;
                write!(&mut res,"method(").unwrap();
                let mut type_vec = Vec::new();
                write_type_prolog(&parsed_class_descriptor,&mut type_vec).unwrap();
//                write_for_write_type.read_to_string(&mut collected_cursor);
                write!(&mut res, "{}", String::from_utf8(type_vec).unwrap()).unwrap();
//                dbg!( String::from_utf8(collected_cursor));
                write!(&mut res,",'{}','{}')",method_name,descriptor).unwrap();
            }else {
                write!(&mut res, "method('{}', '{}', '{}')", class_name, method_name, descriptor).unwrap();
                extra_descriptors.extra_method_descriptors.push(descriptor);
            }
        },
        ConstantKind::Fieldref(f) => {
            let (field_name, descriptor) = name_and_type_extractor(f.name_and_type_index, class_file);
            let c = extract_class_from_constant_pool(f.class_index, class_file);
            let class_name = extract_string_from_utf8(&class_file.constant_pool[c.name_index as usize]);
            write!(&mut res, "field('{}','{}', '{}')",class_name,field_name,descriptor).unwrap();
            extra_descriptors.extra_field_descriptors.push(descriptor);
        },
        ConstantKind::String(s) => {
            let string = extract_string_from_utf8(&class_file.constant_pool[s.string_index as usize]);
            write!(&mut res, "string('{}')",string.replace("\\","\\\\")).unwrap();
        },
        ConstantKind::Integer(i) => {
            write!(&mut res, "int({})",i.bytes).unwrap();
        },
        ConstantKind::Long(l) => {
            let long = (((l.high_bytes as u64) << 32) | (l.low_bytes as u64)) as i64;
            write!(&mut res, "long({})",long).unwrap();
        },
        ConstantKind::Class(c) => {
            let class_name = extract_string_from_utf8(&class_file.constant_pool[c.name_index as usize]);
            if class_name.chars().nth(0).unwrap() == '[' {
                let parsed_class_descriptor = parse_field_descriptor(class_name.as_str()).expect("Error parsing descriptor").field_type;
                let mut type_vec = Vec::new();
                write_type_prolog(&parsed_class_descriptor, &mut type_vec).unwrap();
                write!(&mut res, "{}", String::from_utf8(type_vec).unwrap()).unwrap();
            }else {
                write!(&mut res, "class('{}',{})", class_name,BOOTSTRAP_LOADER_NAME).unwrap();
            }
        }
        ConstantKind::InterfaceMethodref(im) => {
            let (method_name, descriptor) = name_and_type_extractor(im.nt_index, class_file);
            let c = extract_class_from_constant_pool(im.class_index, class_file);
            let class_name = extract_string_from_utf8(&class_file.constant_pool[c.name_index as usize]);
            write!(&mut res, "imethod('{}', '{}', '{}')", class_name, method_name, descriptor).unwrap();
            extra_descriptors.extra_method_descriptors.push(descriptor);
        }
        a => {
            dbg!(a);
            unimplemented!()
        }
    }
    res
}

fn instruction_to_string(prolog_context: &mut ExtraDescriptors,class_file: &Classfile, i: usize, whole_code: &Vec<u8>) -> (String, u64) {
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
            let cp_index = (indexbyte1 << 8) | indexbyte2;
            (format!("anewarray({})", cp_elem_to_string(prolog_context,class_file, cp_index)), 2)
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
            let cp_index = (indexbyte1 << 8) | indexbyte2;
            (format!("checkcast({})", cp_elem_to_string(prolog_context,class_file, cp_index)), 2)
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
            let cp_index = (indexbyte1 << 8) | indexbyte2;
            (format!("getfield({})", cp_elem_to_string(prolog_context,class_file, cp_index)), 2)
        },
        InstructionType::getstatic => {
            let indexbyte1 = code[1] as u16;
            let indexbyte2 = code[2] as u16;
            let cp_index = (indexbyte1 << 8) | indexbyte2;
            (format!("getstatic({})", cp_elem_to_string(prolog_context,class_file, cp_index)), 2)
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
            let cp_index = ((indexbyte1 << 8) | indexbyte2) as u16;
            (format!("instanceof({})", cp_elem_to_string(prolog_context,class_file, cp_index)), 2)
        },
        InstructionType::invokedynamic => {
            let indexbyte1 = code[1] as u16;
            let indexbyte2 = code[2] as u16;
            let cp_index = ((indexbyte1 << 8) | indexbyte2) as u16;
            (format!("invokedynamic({},0,0)", cp_elem_to_string(prolog_context,class_file, cp_index)), 4)
        },
        InstructionType::invokeinterface => {
            let indexbyte1 = code[1] as u16;
            let indexbyte2 = code[2] as u16;
            let cp_index = ((indexbyte1 << 8) | indexbyte2) as u16;
            let count = code[3];
            (format!("invokeinterface({}, {}, 0)", cp_elem_to_string(prolog_context,class_file, cp_index), count), 4)
        },
        InstructionType::invokespecial => {
            let indexbyte1 = code[1] as u16;
            let indexbyte2 = code[2] as u16;
            let cp_index = ((indexbyte1 << 8) | indexbyte2) as u16;
            (format!("invokespecial({})", cp_elem_to_string(prolog_context,class_file, cp_index)), 2)
        },
        InstructionType::invokestatic => {
            let indexbyte1 = code[1] as u16;
            let indexbyte2 = code[2] as u16;
            let cp_index = ((indexbyte1 << 8) | indexbyte2) as u16;
            (format!("invokestatic({})", cp_elem_to_string(prolog_context,class_file, cp_index)), 2)
        },
        InstructionType::invokevirtual => {
            let indexbyte1 = code[1] as u16;
            let indexbyte2 = code[2] as u16;
            let cp_index = ((indexbyte1 << 8) | indexbyte2) as u16;
            (format!("invokevirtual({})", cp_elem_to_string(prolog_context,class_file, cp_index)), 2)
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
            (format!("ldc({})", cp_elem_to_string(prolog_context,class_file, code[1] as u16)), 1)
        },
        InstructionType::ldc_w => {
            let indexbyte1 = code[1] as u16;
            let indexbyte2 = code[2] as u16;
            let cp_index = ((indexbyte1 << 8) | indexbyte2) as u16;
            (format!("ldc_w({})", cp_elem_to_string(prolog_context,class_file, cp_index)), 2)
        },
        InstructionType::ldc2_w => {
            let indexbyte1 = code[1] as u16;
            let indexbyte2 = code[2] as u16;
            let cp_index = ((indexbyte1 << 8) | indexbyte2) as u16;
            (format!("ldc2_w({})", cp_elem_to_string(prolog_context,class_file, cp_index)), 2)
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
        /*InstructionType::lookupswitch => {
            ("lookupswitch(Targets, Keys)".to_string(), unimplemented!())
        },*/
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
            let cp_index = ((indexbyte1 << 8) | indexbyte2) as u16;
            let dimensions = code[3];
            (format!("multianewarray({}, {})",cp_index,dimensions), 3)
        },
        InstructionType::new => {
            let indexbyte1 = code[1] as u16;
            let indexbyte2 = code[2] as u16;
            let cp_index = ((indexbyte1 << 8) | indexbyte2) as u16;
            (format!("new({})",cp_elem_to_string(prolog_context,class_file, cp_index)), 2)
        },
        InstructionType::newarray => {
            let typecode = code[1];
            (format!("newarray({})", typecode), 1)
        },
        InstructionType::nop => { ("nop".to_string(), 0) },
        InstructionType::pop => { ("pop".to_string(), 0) },
        InstructionType::pop2 => { ("pop2".to_string(), 0) },
        InstructionType::putfield => {
            let indexbyte1 = code[1] as u16;
            let indexbyte2 = code[2] as u16;
            let cp_index = ((indexbyte1 << 8) | indexbyte2) as u16;
            (format!("putfield({})",cp_elem_to_string(prolog_context,class_file, cp_index)), 2)
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
        InstructionType::tableswitch => parse_table_switch(i, whole_code),
        InstructionType::wide => {
            ("wide(WidenedInstruction)".to_string(), unimplemented!())
        },
        _ => unimplemented!()
    }
}

fn parse_table_switch(mut i: usize, whole_code: &Vec<u8>) -> (String, u64) {
    let opcode_i = i;
    loop {
        if i % 4 == 0 {
            break;
        }
        i += 1;
    }
    let defaultbyte0 = whole_code[i] as u32;
    let defaultbyte1 = whole_code[i + 1] as u32;
    let defaultbyte2 = whole_code[i + 2] as u32;
    let defaultbyte3 = whole_code[i + 3] as u32;
    let defaultbyte = (defaultbyte0 << 24) | (defaultbyte1 << 16) | (defaultbyte2 << 8) | defaultbyte3;
    i += 4;
    let lowbyte0 = whole_code[i] as u32;
    let lowbyte1 = whole_code[i + 1] as u32;
    let lowbyte2 = whole_code[i + 2] as u32;
    let lowbyte3 = whole_code[i + 3] as u32;
    let lowbyte = (lowbyte0 << 24) | (lowbyte1 << 16) | (lowbyte2 << 8) | lowbyte3;
    i += 4;
    let highbyte0 = whole_code[i] as u32;
    let highbyte1 = whole_code[i + 1] as u32;
    let highbyte2 = whole_code[i + 2] as u32;
    let highbyte3 = whole_code[i + 3] as u32;
    let highbyte = (highbyte0 << 24) | (highbyte1 << 16) | (highbyte2 << 8) | highbyte3;
    let mut targets = Vec::new();
    targets.push(defaultbyte);

    ("tableswitch(Targets, [])".to_string(), unimplemented!())//keys do not matter as long as they are sortable
}

pub fn output_instruction_info_for_code(prolog_context: &mut ExtraDescriptors, class_file: &Classfile, code: &Code, w: &mut dyn Write) -> Result<(), Error> {
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
        let (string, skip_copy) = instruction_to_string(prolog_context,class_file, i, &code.code);
        skip = skip_copy;
        write!(w, ", {})", string)?;
        final_offset = i;
    }
    write!(w,",endOfCode({})",final_offset)?;
    write!(w,"],")?;
    Ok(())
}
