use crate::verifier::{InternalFrame, get_class};
use crate::verifier::Frame;
use rust_jvm_common::classfile::{MethodInfo, StackMapTable, ACC_STATIC, StackMapFrame, SameFrameExtended, ChopFrame, SameLocals1StackItemFrameExtended, AppendFrame, SameFrame, SameLocals1StackItemFrame, FullFrame};
use rust_jvm_common::unified_types::ClassWithLoader;
use classfile_parser::stack_map_table_attribute;
use crate::{StackMap, VerifierContext};
use crate::OperandStack;
use crate::verifier::codecorrectness::expand_to_length;
use classfile_parser::types::MethodDescriptor;
use rust_jvm_common::unified_types::ParsedType;

pub fn get_stack_map_frames(vf: &VerifierContext, class: &ClassWithLoader, method_info: &MethodInfo) -> Vec<StackMap> {
    let mut res = vec![];
    let code = method_info
        .code_attribute()
        .expect("This method won't be called for a non-code attribute function. If you see this , this is a bug");
    let parsed_descriptor = MethodDescriptor::from(method_info,&get_class(vf, class),&class.loader);
    let empty_stack_map = StackMapTable { entries: Vec::new() };
    let stack_map: &StackMapTable = stack_map_table_attribute(code).get_or_insert(&empty_stack_map);
    let this_pointer = if method_info.access_flags & ACC_STATIC > 0 {
        None
    } else {
        Some(ParsedType::Class(ClassWithLoader { class_name: class.class_name.clone(), loader: class.loader.clone() }))
    };
    let mut frame = init_frame(parsed_descriptor.parameter_types, this_pointer, code.max_locals);

    let mut previous_frame_is_first_frame = true;
    for (_, entry) in stack_map.entries.iter().enumerate() {
        match entry {
            StackMapFrame::SameFrame(s) => handle_same_frame(&mut frame, &s),
            StackMapFrame::AppendFrame(append_frame) => handle_append_frame(&mut frame, &append_frame),
            StackMapFrame::SameLocals1StackItemFrame(s) => handle_same_locals_1_stack(&mut frame, &s),
            StackMapFrame::FullFrame(f) => handle_full_frame(&mut frame, &f),
            StackMapFrame::ChopFrame(f) => handle_chop_frame(&mut frame, &f),
            StackMapFrame::SameFrameExtended(f) => handle_same_frame_extended(&mut frame, &f),
            StackMapFrame::SameLocals1StackItemFrameExtended(f) => handle_same_locals_1_stack_frame_extended(&mut frame, &f)
        }
        if previous_frame_is_first_frame {
            previous_frame_is_first_frame = false;
        } else {
            frame.current_offset += 1;
        }
        res.push(StackMap {
            offset: frame.current_offset as usize,
            map_frame: Frame {
                locals: expand_to_length(frame.locals.clone(), frame.max_locals as usize, ParsedType::TopType)
                    .iter()
                    .map(ParsedType::to_verification_type)
                    .collect(),
                stack_map: OperandStack::new_prolog_display_order(&frame.stack.iter()
                    .map(ParsedType::to_verification_type)
                    .collect()),
                flag_this_uninit: false,
            },
        });
    }

    return res;
}


pub fn handle_same_locals_1_stack_frame_extended(mut frame: &mut InternalFrame, f: &SameLocals1StackItemFrameExtended) -> () {
    frame.current_offset += f.offset_delta;
    frame.stack.clear();
    add_verification_type_to_array_convert(&mut frame.stack, &f.stack);
}

pub fn handle_same_frame_extended(mut frame: &mut InternalFrame, f: &SameFrameExtended) -> () {
    frame.current_offset += f.offset_delta;
    frame.stack.clear();
}

pub fn handle_chop_frame(mut frame: &mut InternalFrame, f: &ChopFrame) -> () {
    frame.current_offset += f.offset_delta;
    frame.stack.clear();
    for _ in 0..f.k_frames_to_chop {
        //so basically what's going on here is we want to remove [Double|Long, top],[any type including top]
        let removed = frame.locals.pop().unwrap();
        match removed {
            ParsedType::DoubleType | ParsedType::LongType => panic!(),
            ParsedType::TopType => {
                let second_removed = frame.locals.pop().unwrap();
                match second_removed {
                    ParsedType::DoubleType | ParsedType::LongType => {}
                    _ => {
                        frame.locals.push(second_removed);
                    }
                }
            }
            _ => {}
        }
    }
}

pub fn handle_full_frame(frame: &mut InternalFrame, f: &FullFrame) -> () {
    frame.current_offset += f.offset_delta;
    frame.locals.clear();
    for new_local in f.locals.iter() {
        add_verification_type_to_array_convert(&mut frame.locals, new_local);
    }

    frame.stack.clear();
    for new_stack_member in f.stack.iter() {
        add_verification_type_to_array_convert(&mut frame.stack, new_stack_member);
    }
}

pub fn handle_same_locals_1_stack(frame: &mut InternalFrame, s: &SameLocals1StackItemFrame) -> () {
    frame.current_offset += s.offset_delta;
    frame.stack.clear();
    add_verification_type_to_array_convert(&mut frame.stack, &s.stack);
}

pub fn handle_append_frame(frame: &mut InternalFrame, append_frame: &AppendFrame) -> () {
    frame.current_offset += append_frame.offset_delta;
    for new_local in append_frame.locals.iter() {
        add_verification_type_to_array_convert(&mut frame.locals, new_local)
    }
    frame.stack.clear();
}

pub fn handle_same_frame(frame: &mut InternalFrame, s: &SameFrame) {
    frame.current_offset += s.offset_delta;
    frame.stack.clear();
}


fn add_verification_type_to_array_convert(locals: &mut Vec<ParsedType>, new_local: &ParsedType) -> () {
    match new_local.clone() {
        ParsedType::DoubleType => {
            locals.push(ParsedType::DoubleType);
            locals.push(ParsedType::TopType);
        }
        ParsedType::LongType => {
            locals.push(ParsedType::LongType);
            locals.push(ParsedType::TopType);
        }
        new => { locals.push(new); }
    }
}

pub fn init_frame(parameter_types: Vec<ParsedType>, this_pointer: Option<ParsedType>, max_locals: u16) -> InternalFrame {
    let mut locals = Vec::with_capacity(max_locals as usize);
    match this_pointer {
        None => {}//class is static etc.
        Some(t) => {
//            add_verification_type_to_array_convert(&mut locals, &t)
            add_verification_type_to_array_convert(&mut locals, &ParsedType::UninitializedThisOrClass(t.clone().into()))
        }
    }
    //so these parameter types come unconverted and therefore need conversion
    for parameter_type in parameter_types {
        add_verification_type_to_array_convert(&mut locals, &parameter_type)
    }
    InternalFrame { max_locals, locals, stack: Vec::new(), current_offset: 0 }
}