//parseCodeAttribute(Class, Method, FrameSize, MaxStack, ParsedCode, Handlers, StackMap)

use std::io;
use std::io::Write;

use classfile::{ACC_STATIC, Classfile, code_attribute, MethodInfo, stack_map_table_attribute};
use classfile::attribute_infos::{AppendFrame, ArrayVariableInfo, ChopFrame, Code, ExceptionTableElem, FullFrame, ObjectVariableInfo, SameFrame, SameLocals1StackItemFrame, StackMapFrame, StackMapTable, UninitializedVariableInfo, VerificationTypeInfo};
use verification::types;
use verification::instruction_outputer::extract_class_from_constant_pool;
use verification::prolog_info_writer::{BOOTSTRAP_LOADER_NAME, class_prolog_name, extract_string_from_utf8, write_method_prolog_name};
use verification::types::{ArrayReference, Byte, Char, Int, parse_field_descriptor, parse_method_descriptor, Reference, Void, write_type_prolog};
use verification::prolog_info_writer::PrologGenContext;
use verification::prolog_info_writer::class_name;
use classfile::attribute_infos::SameFrameExtended;
use classfile::attribute_infos::SameLocals1StackItemFrameExtended;
use classfile::code::Instruction;
use verification::verifier::Frame;
use verification::verifier::UnifiedType;

pub enum Name{
    String(String)
}

pub struct ParseCodeAttribute{
    pub class_name: Name,
    pub frame_size : u16,
    pub max_stack: u16,
    pub code : Vec<Instruction>,
    pub exception_table : Vec<ExceptionHandler>,//todo
    pub stackmap_frames: Vec<StackMap>//todo
}

pub struct StackMap{
    pub offset: usize,
    pub map_frame: Frame
}


pub fn write_parse_code_attribute(context: &mut PrologGenContext, w: &mut dyn Write) -> Result<(), io::Error> {
    for class_file in context.to_verify.iter() {
        for method_info in class_file.methods.iter() {
            let code = match code_attribute(&method_info){
                None => {continue;},
                Some(c) => {c},
            };
            write!(w, "parseCodeAttribute({},", class_prolog_name(&class_name(&class_file)))?;
            write_method_prolog_name(&class_file, &method_info, w,false)?;

            let max_stack = code.max_stack;
            let frame_size = code.max_locals;
            write!(w, ",{},{},", frame_size, max_stack)?;

            use verification::instruction_outputer::output_instruction_info_for_code;
            output_instruction_info_for_code(&mut context.extra,&class_file,code, w)?;

            write!(w, "[")?;
            for (i, exception_entry) in code.exception_table.iter().enumerate() {
                write_exception_handler(&class_file, exception_entry, w)?;
                if i != code.exception_table.len() - 1 {
                    write!(w, ",")?;
                }
            }
            write!(w, "],")?;
            write_stack_map_frames(&class_file, &method_info, w)?;
            write!(w, ").\n")?;
        }
    }
    Ok(())
}


pub struct ExceptionHandler{
    start_pc:u32,
    end_pc:u32,
    handler_pc:u32
}

fn write_exception_handler(class_file: &Classfile, exception_handler: &ExceptionTableElem, w: &mut dyn Write) -> Result<(), io::Error> {
    if exception_handler.catch_type == 0{
        write!(w, "handler({},{},{},0)", exception_handler.start_pc, exception_handler.end_pc, exception_handler.handler_pc)?;
    }else {
        let c = extract_class_from_constant_pool(exception_handler.catch_type,class_file);
        let class_name = extract_string_from_utf8(&class_file.constant_pool[c.name_index as usize]);
        write!(w, "handler({},{},{},'{}')", exception_handler.start_pc, exception_handler.end_pc, exception_handler.handler_pc, class_name)?;
    }
    Ok(())
}

/*pub struct Frame {
    pub locals: Vec<VerificationTypeInfo>,
    pub stack: Vec<VerificationTypeInfo>,
    pub max_locals: u16,
    pub current_offset: u16,
}*/

pub fn init_frame(parameter_types: Vec<UnifiedType>, this_pointer: Option<UnifiedType>, max_locals: u16) -> Frame {
    let mut locals = Vec::with_capacity(max_locals as usize);
    match this_pointer {
        None => {},//class is static etc.
        Some(t) => {
            locals_push_convert_type(&mut locals, t)
        },
    }
    for parameter_type in parameter_types {
        locals_push_convert_type(&mut locals,parameter_type)
    }
    Frame { max_locals, locals, stack: Vec::new(), current_offset: 0 }
}

fn locals_push_convert_type(res: &mut Vec<VerificationTypeInfo>, type_: Type) -> () {
    match type_ {
        UnifiedType::ByteType => {
            res.push(VerificationTypeInfo::Integer);
        }
        UnifiedType::CharType => {
            res.push(VerificationTypeInfo::Integer);
        }
        UnifiedType::DoubleType => {
            res.push(VerificationTypeInfo::Double);
            res.push(VerificationTypeInfo::Top);
        }
        UnifiedType::FloatType => {
            res.push(VerificationTypeInfo::Float);
        }
        UnifiedType::IntType => {
            res.push(VerificationTypeInfo::Integer);
        }
        UnifiedType::LongType => {
            res.push(VerificationTypeInfo::Long);
            res.push(VerificationTypeInfo::Top);
        }
        UnifiedType::ReferenceType(r) => {
            assert_ne!(r.class_name.chars().nth(0).unwrap(),'[');
            res.push(VerificationTypeInfo::Object(ObjectVariableInfo { cpool_index: None.clone(), class_name: r.class_name.to_string() }));
        }
        UnifiedType::ShortType => {
            res.push(VerificationTypeInfo::Integer);
        }
        UnifiedType::BooleanType => {
            res.push(VerificationTypeInfo::Integer);
        }
        UnifiedType::ArrayReferenceType(art) => {
            res.push(VerificationTypeInfo::Array(ArrayVariableInfo { array_type: UnifiedType::ArrayReferenceType(art) }));
        }
        UnifiedType::VoidType => { panic!() }
    }
}

fn write_locals(classfile: &Classfile, frame: &Frame, w: &mut dyn Write) -> Result<(), io::Error> {
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
fn verification_type_as_string(classfile: &Classfile, verification_type: &VerificationTypeInfo, w: &mut dyn Write) -> Result<(), io::Error> {
    match verification_type {
        VerificationTypeInfo::Top => { write!(w, "top")?; }
        VerificationTypeInfo::Integer => { write!(w, "int")?; }
        VerificationTypeInfo::Float => { write!(w, "float")?; }
        VerificationTypeInfo::Long => { write!(w, "long")?; }
        VerificationTypeInfo::Double => { write!(w, "double")?; }
        VerificationTypeInfo::Null => { write!(w, "null")?; }
        VerificationTypeInfo::UninitializedThis => { unimplemented!() }
        VerificationTypeInfo::Object(o) => {
            let class_name = object_get_class_name(&classfile, o);
            if class_name.chars().nth(0).unwrap() == '[' {
                let parsed_descriptor = parse_field_descriptor(class_name.as_str()).expect("Error parsing a descriptor").field_type;
                write_type_prolog(&parsed_descriptor, w)?;
                return Ok(())
            }
            write!(w, "class('{}',{})", class_name, BOOTSTRAP_LOADER_NAME)?;
        }
        VerificationTypeInfo::Uninitialized => { unimplemented!() }
        VerificationTypeInfo::Array(a) => {
//            write!(w,"arrayOf(")?;
            write_type_prolog(&a.array_type, w)?;
//            write!(w,")")?;
        }
    }
    Ok(())
}

fn object_get_class_name(classfile: &Classfile, o: &ObjectVariableInfo) -> String {
    let class_name = match o.cpool_index {
        None => { o.class_name.clone() },
        Some(i) => {
            let c = extract_class_from_constant_pool(i, classfile);
            extract_string_from_utf8(&classfile.constant_pool[c.name_index as usize])
        },
    };
    assert_ne!(class_name, "".to_string());
    class_name
}

fn write_operand_stack(classfile: &Classfile, frame: &Frame, w: &mut dyn Write) -> Result<(), io::Error> {
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

fn write_stack_map_frames(class_file: &Classfile, method_info: &MethodInfo, w: &mut dyn Write) -> Result<(), io::Error> {
    let code: &Code = code_attribute(method_info).expect("This method won't be called for a non-code attribute function. If you see this , this is a bug");
    let empty_stack_map = StackMapTable { entries: Vec::new() };
    let stack_map: &StackMapTable = stack_map_table_attribute(code).get_or_insert(&empty_stack_map);
    let descriptor_str = extract_string_from_utf8(&class_file.constant_pool[method_info.descriptor_index as usize]);
    let parsed_descriptor = parse_method_descriptor(descriptor_str.as_str()).expect("Error parsing method descriptor");

    let this_pointer = if method_info.access_flags & ACC_STATIC > 0{
        None
    }else {
        Some(UnifiedType::ReferenceType(Reference {class_name:class_name(class_file) }))
    };

    let mut frame = init_frame(parsed_descriptor.parameter_types, this_pointer, code.max_locals);

    write!(w,"[")?;
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
            StackMapFrame::SameLocals1StackItemFrameExtended(f) => handle_same_locals_1_stack_frame_extended(class_file,&mut frame, f)
        }
        if previous_frame_is_first_frame {
            previous_frame_is_first_frame = false;
        }else{
            frame.current_offset += 1;
        }
        write_stack_map_frame(class_file, w, &frame)?;


        if i != stack_map.entries.len() - 1 {
            write!(w, ",")?;
        }
    }
    write!(w,"]")?;
    Ok(())
}

fn handle_same_locals_1_stack_frame_extended(class_file: &Classfile, mut frame: &mut Frame, f: &SameLocals1StackItemFrameExtended) -> (){
    frame.current_offset  += f.offset_delta;
    frame.stack.clear();
    push_to_stack(class_file, frame, &f.stack);
}

fn handle_same_frame_extended(mut frame: &mut Frame, f: &SameFrameExtended) -> (){
    frame.current_offset += f.offset_delta;
    frame.stack.clear();
}

fn handle_chop_frame(mut frame: &mut Frame, f: &ChopFrame) -> () {
    frame.current_offset += f.offset_delta;
    frame.stack.clear();
    for _ in 0..f.k_frames_to_chop {
        frame.locals.remove(frame.locals.len() - 1);
    }
}

fn handle_full_frame(class_file: &Classfile, frame: &mut Frame, f: &FullFrame) -> () {
    frame.current_offset += f.offset_delta;
    frame.locals.clear();
    for new_local in f.locals.iter() {
        add_new_local(class_file, frame, new_local);
    }

    frame.stack.clear();
    for new_stack_member in f.stack.iter() {
        push_to_stack(class_file, frame, new_stack_member);
    }
}

fn handle_same_locals_1_stack(class_file: &Classfile, frame: &mut Frame, s: &SameLocals1StackItemFrame) -> () {
    frame.current_offset += s.offset_delta;
    frame.stack.clear();
    push_to_stack(class_file, frame, &s.stack);
}

fn handle_append_frame(class_file: &Classfile, frame: &mut Frame, append_frame: &AppendFrame) -> () {
    frame.current_offset += append_frame.offset_delta;
    for new_local in append_frame.locals.iter() {
        add_new_local(class_file, frame, new_local)
    }
}

fn handle_same_frame(frame: &mut Frame, s: &SameFrame) {
    frame.current_offset += s.offset_delta;
    frame.stack.clear();
}

fn write_stack_map_frame(class_file: &Classfile, w: &mut dyn Write, frame: &Frame) -> Result<(),io::Error>{
    write!(w, "stackMap({},frame(", frame.current_offset)?;
    write_locals(class_file, frame, w)?;
    write!(w, ",")?;
    write_operand_stack(class_file, frame, w)?;
    write!(w, ",[]))")?;
    Ok(())
//todo check if flags needed and then write
}


fn push_to_stack(classfile: &Classfile,frame: &mut Frame, new_local: &VerificationTypeInfo){
    add_verification_type_to_array(classfile,&mut frame.stack, new_local)
}

fn add_new_local(classfile: &Classfile,frame: &mut Frame, new_local: &VerificationTypeInfo){
    add_verification_type_to_array(classfile,&mut frame.locals, new_local)
}

fn add_verification_type_to_array(classfile: &Classfile,locals: &mut Vec<VerificationTypeInfo>, new_local: &VerificationTypeInfo) -> () {
    match copy_recurse(classfile,new_local) {
        VerificationTypeInfo::Double => {
            locals.push(VerificationTypeInfo::Double);
            locals.push(VerificationTypeInfo::Top);
        }
        VerificationTypeInfo::Long => {
            locals.push(VerificationTypeInfo::Long);
            locals.push(VerificationTypeInfo::Top);
        }
        new => { locals.push(new); }
    }
}

fn copy_recurse(classfile:&Classfile,to_copy : &VerificationTypeInfo)-> VerificationTypeInfo{
    match to_copy {
        VerificationTypeInfo::Object(o) => {
            let class_name = object_get_class_name(classfile,o);
            /*if class_name.chars().nth(0).unwrap() == '[' {
                let type_ = parse_field_descriptor(class_name.as_str()).expect("Error parsing descriptor").field_type;
                let mut temp  = Vec::with_capacity(1);
                locals_push_convert_type(&mut temp,&type_);
                return copy_recurse(classfile,&temp[0]);
            }*/

            VerificationTypeInfo::Object(ObjectVariableInfo { class_name, cpool_index: o.cpool_index })
        },
        VerificationTypeInfo::Uninitialized(u) => {
            VerificationTypeInfo::Uninitialized(UninitializedVariableInfo { offset: u.offset })
        },
        VerificationTypeInfo::Array(a) => {
            VerificationTypeInfo::Array(ArrayVariableInfo { array_type: copy_type_recurse(&a.array_type) })
        },

        VerificationTypeInfo::Top => {VerificationTypeInfo::Top}
        VerificationTypeInfo::Integer => {VerificationTypeInfo::Integer}
        VerificationTypeInfo::Float => {VerificationTypeInfo::Float}
        VerificationTypeInfo::Long => {VerificationTypeInfo::Long}
        VerificationTypeInfo::Double => {VerificationTypeInfo::Double}
        VerificationTypeInfo::Null => {VerificationTypeInfo::Null}
        VerificationTypeInfo::UninitializedThis => {VerificationTypeInfo::UninitializedThis}
    }
}

fn copy_type_recurse(type_: &UnifiedType) -> UnifiedType {
    match type_ {
        UnifiedType::ByteType => { UnifiedType::ByteType(Byte {}) },
        UnifiedType::CharType => { UnifiedType::CharType(Char {}) },
        UnifiedType::DoubleType => { UnifiedType::DoubleType(types::Double {}) },
        UnifiedType::FloatType => { UnifiedType::FloatType(types::Float {}) },
        UnifiedType::IntType => { UnifiedType::IntType(Int {}) },
        UnifiedType::LongType => { UnifiedType::LongType(types::Long {}) },
        UnifiedType::ShortType => { UnifiedType::ShortType(types::Short {}) },
        UnifiedType::ReferenceType(t) => {
            UnifiedType::ReferenceType(types::Reference { class_name: t.class_name.clone() })
        },
        UnifiedType::BooleanType => { UnifiedType::BooleanType(types::Boolean {}) },
        UnifiedType::ArrayReferenceType(t) => {
            UnifiedType::ArrayReferenceType(ArrayReference { sub_type: Box::new(copy_type_recurse(&t.sub_type)) })
        },
        UnifiedType::VoidType => {
            UnifiedType::VoidType(Void{})
        },
    }
}