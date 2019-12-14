use std::rc::Rc;

use classfile::{ACC_STATIC, Classfile, MethodInfo, code_attribute, stack_map_table_attribute};
use classfile::attribute_infos::{AppendFrame, ChopFrame, FullFrame, SameFrame, SameFrameExtended, SameLocals1StackItemFrame, SameLocals1StackItemFrameExtended, StackMapFrame, StackMapTable, UninitializedVariableInfo};
use verification::code_writer::{init_frame, StackMap};
use verification::prolog_info_writer::{class_name, ParsedMethodDescriptor, extract_string_from_utf8};
use verification::unified_type::{UnifiedType, ArrayType};
use verification::verifier::{InternalFrame, PrologClass, MethodDescriptor};
use verification::classnames::ClassName;
use verification::types::parse_method_descriptor;

pub fn get_stack_map_frames<'l>(class: &'l PrologClass,method_info:&'l MethodInfo) -> Vec<&'l StackMap<'l>> {
    let res = vec![];
    let code = code_attribute(method_info).expect("This method won't be called for a non-code attribute function. If you see this , this is a bug");
    let descriptor_str = extract_string_from_utf8(&class.class.constant_pool[method_info.descriptor_index as usize]);
    let parsed_descriptor = parse_method_descriptor(descriptor_str.as_str()).expect("Error parsing method descriptor");
    let empty_stack_map = StackMapTable { entries: Vec::new() };
    let stack_map: &StackMapTable = stack_map_table_attribute(code).get_or_insert(&empty_stack_map);
    let this_pointer = if method_info.access_flags & ACC_STATIC > 0 {
        None
    } else {
        Some(UnifiedType::ReferenceType(class_name(&class.class)))
    };
    let mut frame = init_frame(parsed_descriptor.parameter_types, this_pointer, code.max_locals);

    let mut previous_frame_is_first_frame = true;
    for (i, entry) in stack_map.entries.iter().enumerate() {
        match entry {
            StackMapFrame::SameFrame(s) => handle_same_frame(&mut frame, &s),
            StackMapFrame::AppendFrame(append_frame) => handle_append_frame(&class.class, &mut frame, &append_frame),
            StackMapFrame::SameLocals1StackItemFrame(s) => handle_same_locals_1_stack(&class.class, &mut frame, &s),
            StackMapFrame::FullFrame(f) => handle_full_frame(&class.class, &mut frame, &f),
            StackMapFrame::ChopFrame(f) => handle_chop_frame(&mut frame, &f),
            StackMapFrame::SameFrameExtended(f) => handle_same_frame_extended(&mut frame, &f),
            StackMapFrame::SameLocals1StackItemFrameExtended(f) => handle_same_locals_1_stack_frame_extended(&class.class, &mut frame, &f)
        }
        if previous_frame_is_first_frame {
            previous_frame_is_first_frame = false;
        } else {
            frame.current_offset += 1;
        }
//        write_stack_map_frame(&class.class, w, &frame)?;
    }

    return res;
}


pub fn handle_same_locals_1_stack_frame_extended(class_file: &Rc<Classfile>, mut frame: &mut InternalFrame, f: &SameLocals1StackItemFrameExtended) -> () {
    frame.current_offset += f.offset_delta;
    frame.stack.clear();
    push_to_stack(class_file, frame, &f.stack);
}

pub fn handle_same_frame_extended(mut frame: &mut InternalFrame, f: &SameFrameExtended) -> () {
    frame.current_offset += f.offset_delta;
    frame.stack.clear();
}

pub fn handle_chop_frame(mut frame: &mut InternalFrame, f: &ChopFrame) -> () {
    frame.current_offset += f.offset_delta;
    frame.stack.clear();
    for _ in 0..f.k_frames_to_chop {
        frame.locals.remove(frame.locals.len() - 1);
    }
}

pub fn handle_full_frame(class_file: &Rc<Classfile>, frame: &mut InternalFrame, f: &FullFrame) -> () {
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

pub fn handle_same_locals_1_stack(class_file: &Rc<Classfile>, frame: &mut InternalFrame, s: &SameLocals1StackItemFrame) -> () {
    frame.current_offset += s.offset_delta;
    frame.stack.clear();
    push_to_stack(class_file, frame, &s.stack);
}

pub fn handle_append_frame(class_file: &Rc<Classfile>, frame: &mut InternalFrame, append_frame: &AppendFrame) -> () {
    frame.current_offset += append_frame.offset_delta;
    for new_local in append_frame.locals.iter() {
        add_new_local(class_file, frame, new_local)
    }
}

pub fn handle_same_frame(frame: &mut InternalFrame, s: &SameFrame) {
    frame.current_offset += s.offset_delta;
    frame.stack.clear();
}


fn push_to_stack(classfile: &Rc<Classfile>, frame: &mut InternalFrame, new_local: &UnifiedType) {
    add_verification_type_to_array(classfile, &mut frame.stack, new_local)
}

fn add_new_local(classfile: &Rc<Classfile>, frame: &mut InternalFrame, new_local: &UnifiedType) {
    add_verification_type_to_array(classfile, &mut frame.locals, new_local)
}

fn add_verification_type_to_array(classfile: &Rc<Classfile>, locals: &mut Vec<UnifiedType>, new_local: &UnifiedType) -> () {
    match copy_recurse(classfile, new_local) {
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

fn copy_recurse(classfile: &Rc<Classfile>, to_copy: &UnifiedType) -> UnifiedType {
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
                ClassName::Ref(r) => { unimplemented!() }
                ClassName::Str(s) => { ClassName::Str(s.clone()) }
            })
        }
        UnifiedType::Uninitialized(u) => {
            UnifiedType::Uninitialized(UninitializedVariableInfo { offset: u.offset })
        }
        UnifiedType::ArrayReferenceType(a) => {
            UnifiedType::ArrayReferenceType(ArrayType { sub_type: Box::from(copy_type_recurse(&a.sub_type)) })
        }

        UnifiedType::TopType => { UnifiedType::TopType }
        UnifiedType::IntType => { UnifiedType::IntType }
        UnifiedType::FloatType => { UnifiedType::FloatType }
        UnifiedType::LongType => { UnifiedType::LongType }
        UnifiedType::DoubleType => { UnifiedType::DoubleType }
        UnifiedType::NullType => { UnifiedType::NullType }
        UnifiedType::UninitializedThis => { UnifiedType::UninitializedThis }
        _ => { panic!("Case wasn't covered with non-unified types") }
    }
}

fn copy_type_recurse(type_: &UnifiedType) -> UnifiedType {
    match type_ {
        UnifiedType::ByteType => { UnifiedType::ByteType }
        UnifiedType::CharType => { UnifiedType::CharType }
        UnifiedType::DoubleType => { UnifiedType::DoubleType }
        UnifiedType::FloatType => { UnifiedType::FloatType }
        UnifiedType::IntType => { UnifiedType::IntType }
        UnifiedType::LongType => { UnifiedType::LongType }
        UnifiedType::ShortType => { UnifiedType::ShortType }
        UnifiedType::ReferenceType(t) => {
            UnifiedType::ReferenceType(match t {
                ClassName::Ref(_) => { unimplemented!() }
                ClassName::Str(s) => { ClassName::Str(s.clone()) }
            })
        }
        UnifiedType::BooleanType => { UnifiedType::BooleanType }
        UnifiedType::ArrayReferenceType(t) => {
            UnifiedType::ArrayReferenceType(ArrayType { sub_type: Box::from(copy_type_recurse(&t.sub_type)) })
        }
        UnifiedType::VoidType => {
            UnifiedType::VoidType
        }
        _ => { panic!("Case wasn't coverred with non-unified types") }
    }
}