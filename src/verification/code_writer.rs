//parseCodeAttribute(Class, Method, FrameSize, MaxStack, ParsedCode, Handlers, StackMap)

use std::io;
use std::io::Write;

use classfile::{ACC_STATIC, Classfile, code_attribute, MethodInfo, stack_map_table_attribute};
use classfile::attribute_infos::{AppendFrame, ChopFrame, Code, ExceptionTableElem, FullFrame, ObjectVariableInfo, SameFrame, SameLocals1StackItemFrame, StackMapFrame, StackMapTable};
use classfile::attribute_infos::SameFrameExtended;
use classfile::attribute_infos::SameLocals1StackItemFrameExtended;
use classfile::attribute_infos::UninitializedVariableInfo;
use classfile::code::Instruction;
use verification::instruction_outputer::extract_class_from_constant_pool;
use verification::prolog_info_writer::{BOOTSTRAP_LOADER_NAME, class_prolog_name, extract_string_from_utf8, write_method_prolog_name};
use verification::prolog_info_writer::class_name;
use verification::prolog_info_writer::PrologGenContext;
use verification::types::parse_field_descriptor;
use verification::types::parse_method_descriptor;
use verification::types::write_type_prolog;
use verification::unified_type::ArrayType;
use verification::unified_type::ClassNameReference;
use verification::unified_type::get_referred_name;
use verification::unified_type::UnifiedType;
use verification::verifier::Frame;
use verification::verifier::Handler;
use verification::verifier::InternalFrame;

pub enum Name{
    String(String)
}

pub struct ParseCodeAttribute<'l>{
    pub class_name: Name,
    pub frame_size : u16,
    pub max_stack: u16,
    pub code : Vec<Instruction>,
    pub exception_table : Vec<Handler>,//todo
    pub stackmap_frames: Vec<StackMap<'l>>//todo
}

pub struct StackMap<'l>{
    pub offset: usize,
    pub map_frame: Frame<'l>
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
    handler_pc:u32,
    catch_type : u32
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


pub fn init_frame<'l>(parameter_types: Vec<UnifiedType<'l>>, this_pointer: Option<UnifiedType<'l>>, max_locals: u16) -> InternalFrame<'l> {
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
    InternalFrame { max_locals, locals, stack: Vec::new(), current_offset: 0 }
}

fn locals_push_convert_type<'l>(res: &mut Vec<UnifiedType<'l>>, type_: UnifiedType<'l>) -> () {
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
                    sub_type: &UnifiedType::ArrayReferenceType(art)
                }));
        }
        UnifiedType::VoidType => { panic!() }
        _ => {panic!("Case wasn't coverred with non-unified types")}
    }
}

fn write_locals(classfile: &Classfile, frame: &InternalFrame, w: &mut dyn Write) -> Result<(), io::Error> {
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
fn verification_type_as_string(classfile: &Classfile, verification_type: &UnifiedType, w: &mut dyn Write) -> Result<(), io::Error> {
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
                let parsed_descriptor = parse_field_descriptor(class_name).expect("Error parsing a descriptor").field_type;
                write_type_prolog(&parsed_descriptor, w)?;
                return Ok(())
            }
            write!(w, "class('{}',{})", class_name, BOOTSTRAP_LOADER_NAME)?;
        }
        UnifiedType::Uninitialized(_) => { unimplemented!() }
        UnifiedType::ArrayReferenceType(a) => {
//            write!(w,"arrayOf(")?;
            write_type_prolog(&a.sub_type, w)?;
//            write!(w,")")?;
        }
        _ => {panic!("Case wasn't coverred with non-unified types")}
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

fn write_operand_stack(classfile: &Classfile, frame: &InternalFrame, w: &mut dyn Write) -> Result<(), io::Error> {
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
        Some(UnifiedType::ReferenceType(ClassNameReference::Str(class_name(class_file))))
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

fn handle_same_locals_1_stack_frame_extended<'l>(class_file: &Classfile, mut frame: &'l mut InternalFrame<'l>, f: &SameLocals1StackItemFrameExtended<'l>) -> (){
    frame.current_offset  += f.offset_delta;
    frame.stack.clear();
    push_to_stack(class_file, frame, &f.stack);
}

fn handle_same_frame_extended<'l>(mut frame: &mut InternalFrame<'l>, f: &SameFrameExtended) -> (){
    frame.current_offset += f.offset_delta;
    frame.stack.clear();
}

fn handle_chop_frame(mut frame: &mut InternalFrame, f: &ChopFrame) -> () {
    frame.current_offset += f.offset_delta;
    frame.stack.clear();
    for _ in 0..f.k_frames_to_chop {
        frame.locals.remove(frame.locals.len() - 1);
    }
}

fn handle_full_frame<'l>(class_file: &Classfile, frame: &'l mut InternalFrame<'l>, f: &FullFrame<'l>) -> () {
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

fn handle_same_locals_1_stack<'l>(class_file: &Classfile, frame: &'l mut InternalFrame<'l>, s: &SameLocals1StackItemFrame<'l>) -> () {
    frame.current_offset += s.offset_delta;
    frame.stack.clear();
    push_to_stack(class_file, frame, &s.stack);
}

fn handle_append_frame<'l>(class_file: &Classfile, frame: &'l mut InternalFrame<'l>, append_frame: &AppendFrame<'l>) -> () {
    frame.current_offset += append_frame.offset_delta;
    for new_local in append_frame.locals.iter() {
        add_new_local(class_file, frame, new_local)
    }
}

fn handle_same_frame(frame: &mut InternalFrame, s: &SameFrame) {
    frame.current_offset += s.offset_delta;
    frame.stack.clear();
}

fn write_stack_map_frame(class_file: &Classfile, w: &mut dyn Write, frame: &InternalFrame) -> Result<(),io::Error>{
    write!(w, "stackMap({},frame(", frame.current_offset)?;
    write_locals(class_file, frame, w)?;
    write!(w, ",")?;
    write_operand_stack(class_file, frame, w)?;
    write!(w, ",[]))")?;
    Ok(())
//todo check if flags needed and then write
}

//todo there should really be two lifetimes here, the verifier lifetime and the classfile lifetime

fn push_to_stack<'l>(classfile: & Classfile,frame: &'l mut InternalFrame<'l>, new_local: &UnifiedType<'l>){
    add_verification_type_to_array(classfile,&mut frame.stack, new_local)
}

fn add_new_local<'l>(classfile: & Classfile,frame: &'l mut InternalFrame<'l>, new_local: &UnifiedType<'l>){
    add_verification_type_to_array(classfile,&mut frame.locals, new_local)
}

fn add_verification_type_to_array<'l>(classfile: &Classfile,locals: &'l mut Vec<UnifiedType<'l>>, new_local: &UnifiedType<'l>) -> () {
    match copy_recurse(classfile,new_local) {
        UnifiedType::DoubleType => {
            locals.push(UnifiedType::DoubleType);
            locals.push(UnifiedType::TopType);
        }
        UnifiedType::LongType => {
            locals.push(UnifiedType::LongType);
            locals.push(UnifiedType::TopType);
        }
        new => { locals.push(new); }
    }
}

fn copy_recurse<'l>(classfile:&Classfile,to_copy : &UnifiedType<'l>)-> UnifiedType<'l>{
    match to_copy {
        UnifiedType::ReferenceType(o) => {
//            let class_name = object_get_class_name(classfile,o);
            /*if class_name.chars().nth(0).unwrap() == '[' {
                let type_ = parse_field_descriptor(class_name.as_str()).expect("Error parsing descriptor").field_type;
                let mut temp  = Vec::with_capacity(1);
                locals_push_convert_type(&mut temp,&type_);
                return copy_recurse(classfile,&temp[0]);
            }*/

            UnifiedType::ReferenceType(match o {
                ClassNameReference::Ref(r) => { unimplemented!() },
                ClassNameReference::Str(s) => { ClassNameReference::Str(s.clone()) },
            })
        },
        UnifiedType::Uninitialized(u) => {
            UnifiedType::Uninitialized(UninitializedVariableInfo { offset: u.offset})
        },
        UnifiedType::ArrayReferenceType(a) => {
            UnifiedType::ArrayReferenceType(ArrayType { sub_type: &copy_type_recurse(&a.sub_type) })
        },

        UnifiedType::TopType => {UnifiedType::TopType}
        UnifiedType::IntType => {UnifiedType::IntType}
        UnifiedType::FloatType => {UnifiedType::FloatType}
        UnifiedType::LongType => {UnifiedType::LongType}
        UnifiedType::DoubleType => {UnifiedType::DoubleType}
        UnifiedType::NullType => {UnifiedType::NullType}
        UnifiedType::UninitializedThis => {UnifiedType::UninitializedThis}
        _ => {panic!("Case wasn't coverred with non-unified types")}
    }
}

fn copy_type_recurse<'l>(type_: &'l UnifiedType<'l>) -> UnifiedType<'l> {
    match type_ {
        UnifiedType::ByteType => { UnifiedType::ByteType },
        UnifiedType::CharType => { UnifiedType::CharType },
        UnifiedType::DoubleType => { UnifiedType::DoubleType },
        UnifiedType::FloatType => { UnifiedType::FloatType },
        UnifiedType::IntType => { UnifiedType::IntType },
        UnifiedType::LongType => { UnifiedType::LongType },
        UnifiedType::ShortType => { UnifiedType::ShortType },
        UnifiedType::ReferenceType(t) => {
            UnifiedType::ReferenceType(match t {
                ClassNameReference::Ref(_) => { unimplemented!() },
                ClassNameReference::Str(s) => { ClassNameReference::Str(s.clone()) },
            })
        },
        UnifiedType::BooleanType => { UnifiedType::BooleanType },
        UnifiedType::ArrayReferenceType(t) => {
            UnifiedType::ArrayReferenceType(ArrayType { sub_type: &copy_type_recurse(&t.sub_type) })
        },
        UnifiedType::VoidType => {
            UnifiedType::VoidType
        },
        _ => {panic!("Case wasn't coverred with non-unified types")}
    }
}