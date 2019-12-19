use std::io::{Error, Write};

use classfile::attribute_infos::Code;
use classfile::Classfile;
use classfile::constant_infos::{Class, ConstantKind};
use verification::prolog_info_writer::{extract_string_from_utf8, BOOTSTRAP_LOADER_NAME, ExtraDescriptors};
use verification::types::{parse_field_descriptor, write_type_prolog};
use classfile::code::Instruction;
use classfile::code::InstructionInfo;
use std::sync::Arc;

pub fn name_and_type_extractor(i: u16, class_file: &Arc<Classfile>) -> (String, String) {
    let nt;
    match &class_file.constant_pool[i as usize].kind {
        ConstantKind::NameAndType(nt_) => {
            nt = nt_;
        }
        _ => { panic!("Ths a bug.") }
    }
    let descriptor = extract_string_from_utf8(&class_file.constant_pool[nt.descriptor_index as usize]);
    let method_name = extract_string_from_utf8(&class_file.constant_pool[nt.name_index as usize]);
    return (method_name, descriptor);
}

pub fn extract_class_from_constant_pool(i: u16, classfile: &Arc<Classfile>) -> &Class {
    match &classfile.constant_pool[i as usize].kind {
        ConstantKind::Class(c) => {
            return c;
        }
        _ => {
            panic!();
        }
    }
}

fn cp_elem_to_string(extra_descriptors: &mut ExtraDescriptors, classfile: &Arc<Classfile>, cp_index: u16, is_ldc: bool) -> String {
    let mut res = String::new();
    use std::fmt::Write;
    match &classfile.constant_pool[cp_index as usize].kind {
        ConstantKind::InvokeDynamic(i) => {
            let (method_name, descriptor) = name_and_type_extractor(i.name_and_type_index, classfile);
            write!(&mut res, "dmethod('{}', '{}')", method_name, descriptor).unwrap();
            extra_descriptors.extra_method_descriptors.push(descriptor);
        }
        ConstantKind::Methodref(m) => {
            let c = extract_class_from_constant_pool(m.class_index, &classfile);
            let class_name = extract_string_from_utf8(&classfile.constant_pool[c.name_index as usize]);
            let (method_name, descriptor) = name_and_type_extractor(m.name_and_type_index, classfile);
            if class_name.chars().nth(0).unwrap() == '[' {
                let parsed_class_descriptor = parse_field_descriptor(class_name.as_str()).expect("Error parsing descriptor").field_type;
                write!(&mut res, "method(").unwrap();
                let mut type_vec = Vec::new();
                write_type_prolog(&parsed_class_descriptor, &mut type_vec).unwrap();
                write!(&mut res, "{}", String::from_utf8(type_vec).unwrap()).unwrap();
                write!(&mut res, ",'{}','{}')", method_name, descriptor).unwrap();
            } else {
                write!(&mut res, "method('{}', '{}', '{}')", class_name, method_name, descriptor).unwrap();
                extra_descriptors.extra_method_descriptors.push(descriptor);
            }
        }
        ConstantKind::Fieldref(f) => {
            let (field_name, descriptor) = name_and_type_extractor(f.name_and_type_index, classfile);
            let c = extract_class_from_constant_pool(f.class_index, &classfile);
            let class_name = extract_string_from_utf8(&classfile.constant_pool[c.name_index as usize]);
            write!(&mut res, "field('{}','{}', '{}')", class_name, field_name, descriptor).unwrap();
            extra_descriptors.extra_field_descriptors.push(descriptor);
        }
        ConstantKind::String(s) => {
            let string = extract_string_from_utf8(&classfile.constant_pool[s.string_index as usize]);
            write!(&mut res, "string('{}')", string.replace("\\", "\\\\").replace("'", "\\'")).unwrap();
        }
        ConstantKind::Integer(i) => {
            write!(&mut res, "int({})", i.bytes).unwrap();
        }
        ConstantKind::Float(f) => {
            write!(&mut res, "float({})", f.bytes).unwrap();
        }
        ConstantKind::Long(l) => {
            let long = (((l.high_bytes as u64) << 32) | (l.low_bytes as u64)) as i64;
            write!(&mut res, "long({})", long).unwrap();
        }
        ConstantKind::Class(c) => {
            let class_name = extract_string_from_utf8(&classfile.constant_pool[c.name_index as usize]);
            if class_name.chars().nth(0).unwrap() == '[' {
                let parsed_class_descriptor = parse_field_descriptor(class_name.as_str()).expect("Error parsing descriptor").field_type;
                let mut type_vec = Vec::new();
                write_type_prolog(&parsed_class_descriptor, &mut type_vec).unwrap();
                write!(&mut res, "{}", String::from_utf8(type_vec).unwrap()).unwrap();
            } else {
                if is_ldc {
                    write!(&mut res, "class('{}')", class_name).unwrap();
                } else {
                    write!(&mut res, "class('{}',{})", class_name, BOOTSTRAP_LOADER_NAME).unwrap();
                }
            }
        }
        ConstantKind::InterfaceMethodref(im) => {
            let (method_name, descriptor) = name_and_type_extractor(im.nt_index, classfile);
            let c = extract_class_from_constant_pool(im.class_index, &classfile);
            let class_name = extract_string_from_utf8(&classfile.constant_pool[c.name_index as usize]);
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

fn instruction_to_string(prolog_context: &mut ExtraDescriptors, class_file: &Arc<Classfile>, instruction: &Instruction) -> String {
    format!("instruction({},{})", instruction.offset, match &instruction.instruction {
        InstructionInfo::aaload => { "aaload".to_string() }
        InstructionInfo::aastore => { "aastore".to_string() }
        InstructionInfo::aconst_null => { "aconst_null".to_string() }
        InstructionInfo::aload(index) => { format!("aload({})", index) }
        InstructionInfo::aload_0 => { "aload_0".to_string() }
        InstructionInfo::aload_1 => { "aload_1".to_string() }
        InstructionInfo::aload_2 => { "aload_2".to_string() }
        InstructionInfo::aload_3 => { "aload_3".to_string() }
        InstructionInfo::anewarray(cp_index) => { format!("anewarray({})", cp_elem_to_string(prolog_context, class_file, *cp_index, false)) }
        InstructionInfo::areturn => { "areturn".to_string() }
        InstructionInfo::arraylength => { "arraylength".to_string() }
        InstructionInfo::astore(index) => { format!("astore({})", index) }
        InstructionInfo::astore_0 => { "astore_0".to_string() }
        InstructionInfo::astore_1 => { "astore_1".to_string() }
        InstructionInfo::astore_2 => { "astore_2".to_string() }
        InstructionInfo::astore_3 => { "astore_3".to_string() }
        InstructionInfo::athrow => { "athrow".to_string() }
        InstructionInfo::baload => { "baload".to_string() }
        InstructionInfo::bastore => { "bastore".to_string() }
        InstructionInfo::bipush(byte) => { format!("bipush({})", byte) }
        InstructionInfo::caload => { "caload".to_string() }
        InstructionInfo::castore => { "castore".to_string() }
        InstructionInfo::checkcast(cp_index) => { format!("checkcast({})", cp_elem_to_string(prolog_context, class_file, *cp_index, false)) }
        InstructionInfo::d2f => { "d2f".to_string() }
        InstructionInfo::d2i => { "d2i".to_string() }
        InstructionInfo::d2l => { "d2l".to_string() }
        InstructionInfo::dadd => { "dadd".to_string() }
        InstructionInfo::daload => { "daload".to_string() }
        InstructionInfo::dastore => { "dastore".to_string() }
        InstructionInfo::dcmpg => { "dcmpg".to_string() }
        InstructionInfo::dcmpl => { "dcmpl".to_string() }
        InstructionInfo::dconst_0 => { "dconst_0".to_string() }
        InstructionInfo::dconst_1 => { "dconst_1".to_string() }
        InstructionInfo::ddiv => { "ddiv".to_string() }
        InstructionInfo::dload(index) => { format!("dload({})", index) }
        InstructionInfo::dload_0 => { "dload_0".to_string() }
        InstructionInfo::dload_1 => { "dload_1".to_string() }
        InstructionInfo::dload_2 => { "dload_2".to_string() }
        InstructionInfo::dload_3 => { "dload_3".to_string() }
        InstructionInfo::dmul => { "dmul".to_string() }
        InstructionInfo::dneg => { "dneg".to_string() }
        InstructionInfo::drem => { "drem".to_string() }
        InstructionInfo::dreturn => { "dreturn".to_string() }
        InstructionInfo::dstore(index) => { format!("dstore({})", index) }
        InstructionInfo::dstore_0 => { "dstore_0".to_string() }
        InstructionInfo::dstore_1 => { "dstore_1".to_string() }
        InstructionInfo::dstore_2 => { "dstore_2".to_string() }
        InstructionInfo::dstore_3 => { "dstore_3".to_string() }
        InstructionInfo::dsub => { "dsub".to_string() }
        InstructionInfo::dup => { "dup".to_string() }
        InstructionInfo::dup_x1 => { "dup_x1".to_string() }
        InstructionInfo::dup_x2 => { "dup_x2".to_string() }
        InstructionInfo::dup2 => { "dup2".to_string() }
        InstructionInfo::dup2_x1 => { "dup2_x1".to_string() }
        InstructionInfo::dup2_x2 => { "dup2_x2".to_string() }
        InstructionInfo::f2d => { "f2d".to_string() }
        InstructionInfo::f2i => { "f2i".to_string() }
        InstructionInfo::f2l => { "f2l".to_string() }
        InstructionInfo::fadd => { "fadd".to_string() }
        InstructionInfo::faload => { "faload".to_string() }
        InstructionInfo::fastore => { "fastore".to_string() }
        InstructionInfo::fcmpg => { "fcmpg".to_string() }
        InstructionInfo::fcmpl => { "fcmpl".to_string() }
        InstructionInfo::fconst_0 => { "fconst_0".to_string() }
        InstructionInfo::fconst_1 => { "fconst_1".to_string() }
        InstructionInfo::fconst_2 => { "fconst_2".to_string() }
        InstructionInfo::fdiv => { "fdiv".to_string() }
        InstructionInfo::fload(index) => { format!("fload({})", index) }
        InstructionInfo::fload_0 => { "fload_0".to_string() }
        InstructionInfo::fload_1 => { "fload_1".to_string() }
        InstructionInfo::fload_2 => { "fload_2".to_string() }
        InstructionInfo::fload_3 => { "fload_3".to_string() }
        InstructionInfo::fmul => { "fmul".to_string() }
        InstructionInfo::fneg => { "fneg".to_string() }
        InstructionInfo::frem => { "frem".to_string() }
        InstructionInfo::freturn => { "freturn".to_string() }
        InstructionInfo::fstore(index) => { format!("fstore({})", index) }
        InstructionInfo::fstore_0 => { "fstore_0".to_string() }
        InstructionInfo::fstore_1 => { "fstore_1".to_string() }
        InstructionInfo::fstore_2 => { "fstore_2".to_string() }
        InstructionInfo::fstore_3 => { "fstore_3".to_string() }
        InstructionInfo::fsub => { "fsub".to_string() }
        InstructionInfo::getfield(cp_index) => { format!("getfield({})", cp_elem_to_string(prolog_context, class_file, *cp_index, false)) }
        InstructionInfo::getstatic(cp_index) => { format!("getstatic({})", cp_elem_to_string(prolog_context, class_file, *cp_index, false)) }
        InstructionInfo::goto_(goto_offset) => { format!("goto({})", (*goto_offset as isize + instruction.offset as isize)) }
        InstructionInfo::goto_w(goto_w_offset) => { format!("goto_w({})", *goto_w_offset as isize + instruction.offset as isize) }
        InstructionInfo::i2b => { "i2b".to_string() }
        InstructionInfo::i2c => { "i2c".to_string() }
        InstructionInfo::i2d => { "i2d".to_string() }
        InstructionInfo::i2f => { "i2f".to_string() }
        InstructionInfo::i2l => { "i2l".to_string() }
        InstructionInfo::i2s => { "i2s".to_string() }
        InstructionInfo::iadd => { "iadd".to_string() }
        InstructionInfo::iaload => { "iaload".to_string() }
        InstructionInfo::iand => { "iand".to_string() }
        InstructionInfo::iastore => { "iastore".to_string() }
        InstructionInfo::iconst_m1 => { "iconst_m1".to_string() }
        InstructionInfo::iconst_0 => { "iconst_0".to_string() }
        InstructionInfo::iconst_1 => { "iconst_1".to_string() }
        InstructionInfo::iconst_2 => { "iconst_2".to_string() }
        InstructionInfo::iconst_3 => { "iconst_3".to_string() }
        InstructionInfo::iconst_4 => { "iconst_4".to_string() }
        InstructionInfo::iconst_5 => { "iconst_5".to_string() }
        InstructionInfo::idiv => { "idiv".to_string() }
        InstructionInfo::if_acmpeq(index) => {
            format!("if_acmpeq({})", *index as isize + instruction.offset as isize)//todo duplication
        }
        InstructionInfo::if_acmpne(index) => {
            format!("if_acmpne({})", *index as isize + instruction.offset as isize)//todo duplication
        }
        InstructionInfo::if_icmpeq(index) => {
            format!("if_icmpeq({})", *index as isize + instruction.offset as isize)//todo duplication
        }
        InstructionInfo::if_icmpne(index) => {
            format!("if_icmpne({})", *index as isize + instruction.offset as isize)//todo duplication
        }
        InstructionInfo::if_icmplt(index) => {
            format!("if_icmplt({})", *index as isize + instruction.offset as isize)//todo duplication
        }
        InstructionInfo::if_icmpge(index) => {
            format!("if_icmpge({})", *index as isize + instruction.offset as isize)//todo duplication
        }
        InstructionInfo::if_icmpgt(index) => {
            format!("if_icmpgt({})", *index as isize + instruction.offset as isize)//todo duplication
        }
        InstructionInfo::if_icmple(index) => {
            format!("if_icmple({})", *index as isize + instruction.offset as isize)//todo duplication
        }
        InstructionInfo::ifeq(index) => {
            format!("ifeq({})", *index as isize + instruction.offset as isize)//todo duplication
        }
        InstructionInfo::ifne(index) => {
            format!("ifne({})", *index as isize + instruction.offset as isize)//todo duplication
        }
        InstructionInfo::iflt(index) => {
            format!("iflt({})", *index as isize + instruction.offset as isize)//todo duplication
        }
        InstructionInfo::ifge(index) => {
            format!("ifge({})", *index as isize + instruction.offset as isize)//todo duplication
        }
        InstructionInfo::ifgt(index) => {
            format!("ifgt({})", *index as isize + instruction.offset as isize)//todo duplication
        }
        InstructionInfo::ifle(index) => {
            format!("ifle({})", *index as isize + instruction.offset as isize)//todo duplication
        }
        InstructionInfo::ifnonnull(index) => {
            format!("ifnonnull({})", *index as isize + instruction.offset as isize)//todo duplication
        }
        InstructionInfo::ifnull(index) => {
            format!("ifnull({})", *index as isize + instruction.offset as isize)//todo duplication
        }
        InstructionInfo::iinc(iinc) => {
            let index = iinc.index;
            let const_ = iinc.const_;
            format!("iinc({},{})", index, const_)
        }
        InstructionInfo::iload(index) => {
            format!("iload({})", index)
        }
        InstructionInfo::iload_0 => { "iload_0".to_string() }
        InstructionInfo::iload_1 => { "iload_1".to_string() }
        InstructionInfo::iload_2 => { "iload_2".to_string() }
        InstructionInfo::iload_3 => { "iload_3".to_string() }
        InstructionInfo::imul => { "imul".to_string() }
        InstructionInfo::ineg => { "ineg".to_string() }
        InstructionInfo::instanceof(cp_index) => {
            format!("instanceof({})", cp_elem_to_string(prolog_context, class_file, *cp_index, false))
        }
        InstructionInfo::invokedynamic(cp_index) => {
            format!("invokedynamic({},0,0)", cp_elem_to_string(prolog_context, class_file, *cp_index, false))
        }
        InstructionInfo::invokeinterface(interface) => {
            format!("invokeinterface({}, {}, 0)", cp_elem_to_string(prolog_context, class_file, interface.index, false), interface.count)
        }
        InstructionInfo::invokespecial(cp_index) => {
            format!("invokespecial({})", cp_elem_to_string(prolog_context, class_file, *cp_index, false))
        }
        InstructionInfo::invokestatic(cp_index) => {
            format!("invokestatic({})", cp_elem_to_string(prolog_context, class_file, *cp_index, false))
        }
        InstructionInfo::invokevirtual(cp_index) => {
            format!("invokevirtual({})", cp_elem_to_string(prolog_context, class_file, *cp_index, false))
        }
        InstructionInfo::ior => { "ior".to_string() }
        InstructionInfo::irem => { "irem".to_string() }
        InstructionInfo::ireturn => { "ireturn".to_string() }
        InstructionInfo::ishl => { "ishl".to_string() }
        InstructionInfo::ishr => { "ishr".to_string() }
        InstructionInfo::istore(index) => {
            format!("istore({})", index)
        }
        InstructionInfo::istore_0 => { "istore_0".to_string() }
        InstructionInfo::istore_1 => { "istore_1".to_string() }
        InstructionInfo::istore_2 => { "istore_2".to_string() }
        InstructionInfo::istore_3 => { "istore_3".to_string() }
        InstructionInfo::isub => { "isub".to_string() }
        InstructionInfo::iushr => { "iushr".to_string() }
        InstructionInfo::ixor => { "ixor".to_string() }
        InstructionInfo::jsr(branch) => { format!("jsr({})", branch) }
        InstructionInfo::jsr_w(branch) => { format!("jsr_w({})", branch) }
        InstructionInfo::l2d => { "l2d".to_string() }
        InstructionInfo::l2f => { "l2f".to_string() }
        InstructionInfo::l2i => { "l2i".to_string() }
        InstructionInfo::ladd => { "ladd".to_string() }
        InstructionInfo::laload => { "laload".to_string() }
        InstructionInfo::land => { "land".to_string() }
        InstructionInfo::lastore => { "lastore".to_string() }
        InstructionInfo::lcmp => { "lcmp".to_string() }
        InstructionInfo::lconst_0 => { "lconst_0".to_string() }
        InstructionInfo::lconst_1 => { "lconst_1".to_string() }
        InstructionInfo::ldc(index) => {
            format!("ldc({})", cp_elem_to_string(prolog_context, class_file, *index as u16, true))
        }
        InstructionInfo::ldc_w(cp_index) => {
            format!("ldc_w({})", cp_elem_to_string(prolog_context, class_file, *cp_index, true))
        }
        InstructionInfo::ldc2_w(cp_index) => {
            format!("ldc2_w({})", cp_elem_to_string(prolog_context, class_file, *cp_index, true))
        }
        InstructionInfo::ldiv => { "ldiv".to_string() }
        InstructionInfo::lload(index) => {
            format!("lload({})", index)
        }
        InstructionInfo::lload_0 => { "lload_0".to_string() }
        InstructionInfo::lload_1 => { "lload_1".to_string() }
        InstructionInfo::lload_2 => { "lload_2".to_string() }
        InstructionInfo::lload_3 => { "lload_3".to_string() }
        InstructionInfo::lmul => { "lmul".to_string() }
        InstructionInfo::lneg => { "lneg".to_string() }
        InstructionInfo::lookupswitch(l) => {
            let (_, targets): (Vec<i32>, Vec<i32>) = l.pairs.iter().cloned().unzip();
            let mut res = "lookupswitch([".to_string();
            for target in targets {
                res.push_str(format!("{},", target).as_str());
            }
            res.push_str(format!("{}],[key_unimplemented])", l.default).as_str());
            res
        }
        InstructionInfo::tableswitch(t) => {
            let mut res = "tableswitch([".to_string();
            t.offsets.iter().for_each(|i| {
                let jump_target = *i as isize + instruction.offset as isize;
                res.push_str(format!("{},", jump_target).as_str());
            });
            res.push_str("{}],[keys_unimplmented])");
            res
        }
        InstructionInfo::lor => { "lor".to_string() }
        InstructionInfo::lrem => { "lrem".to_string() }
        InstructionInfo::lreturn => { "lreturn".to_string() }
        InstructionInfo::lshl => { "lshl".to_string() }
        InstructionInfo::lshr => { "lshr".to_string() }
        InstructionInfo::lstore(index) => {
            format!("lstore({})", index)
        }
        InstructionInfo::lstore_0 => { "lstore_0".to_string() }
        InstructionInfo::lstore_1 => { "lstore_1".to_string() }
        InstructionInfo::lstore_2 => { "lstore_2".to_string() }
        InstructionInfo::lstore_3 => { "lstore_3".to_string() }
        InstructionInfo::lsub => { "lsub".to_string() }
        InstructionInfo::lushr => { "lushr".to_string() }
        InstructionInfo::lxor => { "lxor".to_string() }
        InstructionInfo::monitorenter => { "monitorenter".to_string() }
        InstructionInfo::monitorexit => { "monitorexit".to_string() }
        InstructionInfo::multianewarray(_m) => {
            unimplemented!();
//            let indexbyte1 = code[1] as u16;
//            let indexbyte2 = code[2] as u16;
//            let cp_index = ((indexbyte1 << 8) | indexbyte2) as u16;
//            let dimensions = code[3];
//            (format!("multianewarray({}, {})", cp_index, dimensions), 3)
        }
        InstructionInfo::new(cp_index) => {
            format!("new({})", cp_elem_to_string(prolog_context, class_file, *cp_index, false))
        }
        InstructionInfo::newarray(typecode) => {
            format!("newarray({})", *typecode as u8)
        }
        InstructionInfo::nop => { "nop".to_string() }
        InstructionInfo::pop => { "pop".to_string() }
        InstructionInfo::pop2 => { "pop2".to_string() }
        InstructionInfo::putfield(cp_index) => {
            format!("putfield({})", cp_elem_to_string(prolog_context, class_file, *cp_index, false))
        }
        InstructionInfo::putstatic(cp_index) => {
            format!("putstatic({})", cp_elem_to_string(prolog_context, class_file, *cp_index, false))
        }
        InstructionInfo::ret(index) => { format!("ret({})", index) }
        InstructionInfo::return_ => { "return".to_string() }
        InstructionInfo::saload => { "saload".to_string() }
        InstructionInfo::sastore => { "sastore".to_string() }
        InstructionInfo::sipush(value) => {
            format!("sipush({})", value)
        }
        InstructionInfo::swap => { "swap".to_string() }
        InstructionInfo::wide(_wide) => {
            unimplemented!();
//            ("wide(WidenedInstruction)".to_string(), unimplemented!())
        },
        _ => unimplemented!()
    })
}

pub fn output_instruction_info_for_code(prolog_context: &mut ExtraDescriptors, class_file: &Arc<Classfile>, code: &Code, w: &mut dyn Write) -> Result<(), Error> {
    write!(w, "[")?;
    for instruction in code.code.iter() {
        write!(w, "{},", instruction_to_string(prolog_context, class_file, instruction))?;
    }
    let last_offset = match code.code.last() {
        None => { 0 }
        Some(s) => { s.offset }
    };
    write!(w, "endOfCode({})", last_offset)?;
    write!(w, "],")?;
    Ok(())
}
