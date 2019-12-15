//parseCodeAttribute(Class, Method, FrameSize, MaxStack, ParsedCode, Handlers, StackMap)

use std::io;
use std::io::Write;
use verification::unified_type::{UnifiedType, ArrayType};
use classfile::code::Instruction;
use verification::verifier::codecorrectness::Handler;
use verification::prolog_info_writer::{PrologGenContext, write_method_prolog_name, class_prolog_name, class_name_legacy, class_name, extract_string_from_utf8, BOOTSTRAP_LOADER_NAME};
use classfile::{code_attribute, stack_map_table_attribute, Classfile, MethodInfo, ACC_STATIC};
use classfile::attribute_infos::{StackMapFrame, Code, SameLocals1StackItemFrameExtended, ChopFrame, FullFrame, UninitializedVariableInfo, StackMapTable, SameFrameExtended, ObjectVariableInfo, ExceptionTableElem, AppendFrame, SameFrame, SameLocals1StackItemFrame};
use verification::types::{parse_method_descriptor, parse_field_descriptor, write_type_prolog};
use verification::verifier::{InternalFrame, Frame};
use verification::instruction_outputer::extract_class_from_constant_pool;
use verification::classnames::{ClassName, NameReference, get_referred_name};
use verification::verifier::codecorrectness::stackmapframes::{handle_same_frame, handle_append_frame, handle_same_locals_1_stack, handle_full_frame, handle_chop_frame, handle_same_frame_extended, handle_same_locals_1_stack_frame_extended};
use verification::verifier::PrologClassMethod;
use std::sync::Arc;

#[derive(Debug)]
pub struct StackMap<'l> {
    pub offset: usize,
    pub map_frame: Frame<'l>,
}


pub fn write_parse_code_attribute(context: &mut PrologGenContext, w: &mut dyn Write) -> Result<(), io::Error> {
    for classfile in context.to_verify.iter() {
        for method_info in classfile.methods.iter() {
            let code = match code_attribute(&method_info) {
                None => { continue; }
                Some(c) => { c }
            };
            write!(w, "parseCodeAttribute({},", get_referred_name(&class_name(&classfile)))?;
            write_method_prolog_name(&classfile, &method_info, w, false)?;

            let max_stack = code.max_stack;
            let frame_size = code.max_locals;
            write!(w, ",{},{},", frame_size, max_stack)?;

            use verification::instruction_outputer::output_instruction_info_for_code;
            output_instruction_info_for_code(&mut context.extra, &classfile, code, w)?;

            write!(w, "[")?;
            for (i, exception_entry) in code.exception_table.iter().enumerate() {
                write_exception_handler(&classfile, exception_entry, w)?;
                if i != code.exception_table.len() - 1 {
                    write!(w, ",")?;
                }
            }
            write!(w, "],")?;
            write_stack_map_frames(&classfile, &method_info, w)?;
            write!(w, ").\n")?;
        }
    }
    Ok(())
}


#[allow(dead_code)]
pub struct ExceptionHandler {
    start_pc: u32,
    end_pc: u32,
    handler_pc: u32,
    catch_type: u32,
}

fn write_exception_handler(class_file: &Arc<Classfile>, exception_handler: &ExceptionTableElem, w: &mut dyn Write) -> Result<(), io::Error> {
    if exception_handler.catch_type == 0 {
        write!(w, "handler({},{},{},0)", exception_handler.start_pc, exception_handler.end_pc, exception_handler.handler_pc)?;
    } else {
        let c = extract_class_from_constant_pool(exception_handler.catch_type, class_file);
        let class_name = extract_string_from_utf8(&class_file.constant_pool[c.name_index as usize]);
        write!(w, "handler({},{},{},'{}')", exception_handler.start_pc, exception_handler.end_pc, exception_handler.handler_pc, class_name)?;
    }
    Ok(())
}


pub fn init_frame(parameter_types: Vec<UnifiedType>, this_pointer: Option<UnifiedType>, max_locals: u16) -> InternalFrame {
    let mut locals = Vec::with_capacity(max_locals as usize);
    match this_pointer {
        None => {}//class is static etc.
        Some(t) => {
            locals_push_convert_type(&mut locals, t)
        }
    }
    for parameter_type in parameter_types {
        locals_push_convert_type(&mut locals, parameter_type)
    }
    InternalFrame { max_locals, locals, stack: Vec::new(), current_offset: 0 }
}

fn locals_push_convert_type(res: &mut Vec<UnifiedType>, type_: UnifiedType) -> () {
    match type_ {
        UnifiedType::ByteType => {
            res.push(UnifiedType::IntType);
        }
        UnifiedType::CharType => {
            res.push(UnifiedType::IntType);
        }
        UnifiedType::DoubleType => {
            res.push(UnifiedType::DoubleType);
            res.push(UnifiedType::TopType);
        }
        UnifiedType::FloatType => {
            res.push(UnifiedType::FloatType);
        }
        UnifiedType::IntType => {
            res.push(UnifiedType::IntType);
        }
        UnifiedType::LongType => {
            res.push(UnifiedType::LongType);
            res.push(UnifiedType::TopType);
        }
        UnifiedType::ReferenceType(r) => {
            assert_ne!(get_referred_name(&r).chars().nth(0).unwrap(), '[');
            res.push(UnifiedType::ReferenceType(r));
        }
        UnifiedType::ShortType => {
            res.push(UnifiedType::IntType);
        }
        UnifiedType::BooleanType => {
            res.push(UnifiedType::IntType);
        }
        UnifiedType::ArrayReferenceType(art) => {
            res.push(UnifiedType::ArrayReferenceType(
                ArrayType {
                    sub_type: Box::from(UnifiedType::ArrayReferenceType(art))
                }));
        }
        UnifiedType::VoidType => { panic!() }
        _ => { panic!("Case wasn't coverred with non-unified types") }
    }
}

fn write_locals(classfile: &Arc<Classfile>, frame: &InternalFrame, w: &mut dyn Write) -> Result<(), io::Error> {
    write!(w, "[")?;
    for (i, local) in frame.locals.iter().enumerate() {
        verification_type_as_string(classfile, local, w)?;
        if i != frame.locals.len() - 1 {
            write!(w, ",")?;
        }
    }
    for _ in 0..(frame.max_locals - frame.locals.len() as u16) {
        if !frame.locals.is_empty() {
            write!(w, ",")?;
        }
        write!(w, "top")?;
    }
    write!(w, "]")?;
    Ok(())
}

//todo this should really be a write function
#[allow(unused)]
fn verification_type_as_string(classfile: &Arc<Classfile>, verification_type: &UnifiedType, w: &mut dyn Write) -> Result<(), io::Error> {
    match verification_type {
        UnifiedType::TopType => { write!(w, "top")?; }
        UnifiedType::IntType => { write!(w, "int")?; }
        UnifiedType::FloatType => { write!(w, "float")?; }
        UnifiedType::LongType => { write!(w, "long")?; }
        UnifiedType::DoubleType => { write!(w, "double")?; }
        UnifiedType::NullType => { write!(w, "null")?; }
        UnifiedType::UninitializedThis => { unimplemented!() }
        UnifiedType::ReferenceType(o) => {
            let class_name = get_referred_name(o);
            if class_name.chars().nth(0).unwrap() == '[' {
                let parsed_descriptor = parse_field_descriptor(class_name.as_str()).expect("Error parsing a descriptor").field_type;
                write_type_prolog(&parsed_descriptor, w)?;
                return Ok(());
            }
            write!(w, "class('{}',{})", class_name, BOOTSTRAP_LOADER_NAME)?;
        }
        UnifiedType::Uninitialized(_) => { unimplemented!() }
        UnifiedType::ArrayReferenceType(a) => {
//            write!(w,"arrayOf(")?;
            write_type_prolog(&a.sub_type, w)?;
//            write!(w,")")?;
        }
        _ => { panic!("Case wasn't coverred with non-unified types") }
    }
    Ok(())
}

#[allow(unused)]
fn object_get_class_name(classfile: &Arc<Classfile>, o: &ObjectVariableInfo) -> String {
    let class_name = match o.cpool_index {
        None => { o.class_name.clone() }
        Some(i) => {
            let c = extract_class_from_constant_pool(i, classfile);
            extract_string_from_utf8(&classfile.constant_pool[c.name_index as usize])
        }
    };
    assert_ne!(class_name, "".to_string());
    class_name
}

fn write_operand_stack(classfile: &Arc<Classfile>, frame: &InternalFrame, w: &mut dyn Write) -> Result<(), io::Error> {
    write!(w, "[")?;
    for (i, local) in frame.stack.iter().rev().enumerate() {
        verification_type_as_string(classfile, local, w)?;
        if i != frame.stack.len() - 1 {
            write!(w, ",")?;
        }
    }
    write!(w, "]")?;
    Ok(())
}

fn write_stack_map_frames(class_file: &Arc<Classfile>, method_info: &MethodInfo, w: &mut dyn Write) -> Result<(), io::Error> {
    let code: &Code = code_attribute(method_info).expect("This method won't be called for a non-code attribute function. If you see this , this is a bug");
    let empty_stack_map = StackMapTable { entries: Vec::new() };
    let stack_map: &StackMapTable = stack_map_table_attribute(code).get_or_insert(&empty_stack_map);
    let descriptor_str = extract_string_from_utf8(&class_file.constant_pool[method_info.descriptor_index as usize]);
    let parsed_descriptor = parse_method_descriptor(descriptor_str.as_str()).expect("Error parsing method descriptor");

    let this_pointer = if method_info.access_flags & ACC_STATIC > 0 {
        None
    } else {
        Some(UnifiedType::ReferenceType(class_name(class_file)))
    };

    let mut frame = init_frame(parsed_descriptor.parameter_types, this_pointer, code.max_locals);

    write!(w, "[")?;
    //the fact that this variable needs to exist is dumb, but the spec says so
    let mut previous_frame_is_first_frame = true;
    for (i, entry) in stack_map.entries.iter().enumerate() {
        match entry {
            StackMapFrame::SameFrame(s) => handle_same_frame(&mut frame, s),
            StackMapFrame::AppendFrame(append_frame) => handle_append_frame(class_file, &mut frame, append_frame),
            StackMapFrame::SameLocals1StackItemFrame(s) => handle_same_locals_1_stack(class_file, &mut frame, s),
            StackMapFrame::FullFrame(f) => handle_full_frame(class_file, &mut frame, f),
            StackMapFrame::ChopFrame(f) => handle_chop_frame(&mut frame, f),
            StackMapFrame::SameFrameExtended(f) => handle_same_frame_extended(&mut frame, f),
            StackMapFrame::SameLocals1StackItemFrameExtended(f) => handle_same_locals_1_stack_frame_extended(class_file, &mut frame, f)
        }
        if previous_frame_is_first_frame {
            previous_frame_is_first_frame = false;
        } else {
            frame.current_offset += 1;
        }
        write_stack_map_frame(class_file, w, &frame)?;


        if i != stack_map.entries.len() - 1 {
            write!(w, ",")?;
        }
    }
    write!(w, "]")?;
    Ok(())
}


fn write_stack_map_frame(class_file: &Arc<Classfile>, w: &mut dyn Write, frame: &InternalFrame) -> Result<(), io::Error> {
    write!(w, "stackMap({},frame(", frame.current_offset)?;
    write_locals(class_file, frame, w)?;
    write!(w, ",")?;
    write_operand_stack(class_file, frame, w)?;
    write!(w, ",[]))")?;
    Ok(())
//todo check if flags needed and then write
}

//todo there should really be two lifetimes here, the verifier lifetime and the classfile lifetime
