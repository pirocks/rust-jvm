use crate::verifier::{InternalFrame, get_class};
use crate::verifier::Frame;
use rust_jvm_common::classfile::{MethodInfo, StackMapTable, ACC_STATIC, StackMapFrame, SameFrameExtended, ChopFrame, SameLocals1StackItemFrameExtended, AppendFrame, SameFrame, SameLocals1StackItemFrame, FullFrame};
use rust_jvm_common::utils::extract_string_from_utf8;
use rust_jvm_common::unified_types::{UnifiedType, ClassWithLoader};
use classfile_parser::{code_attribute, stack_map_table_attribute};
use crate::{init_frame, VerifierContext};
use crate::StackMap;
use crate::OperandStack;
use crate::verifier::codecorrectness::expand_to_length;
use classfile_parser::types::parse_method_descriptor;

pub fn get_stack_map_frames(vf: &VerifierContext,class: &ClassWithLoader, method_info: &MethodInfo) -> Vec<StackMap> {
    let mut res = vec![];
    let code = code_attribute(method_info).expect("This method won't be called for a non-code attribute function. If you see this , this is a bug");
    let descriptor_str = extract_string_from_utf8(&get_class(vf,class).constant_pool[method_info.descriptor_index as usize]);
    let parsed_descriptor = parse_method_descriptor(&class.loader, descriptor_str.as_str()).expect("Error parsing method descriptor");
    let empty_stack_map = StackMapTable { entries: Vec::new() };
    let stack_map: &StackMapTable = stack_map_table_attribute(code).get_or_insert(&empty_stack_map);
    let this_pointer = if method_info.access_flags & ACC_STATIC > 0 {
        None
    } else {
        Some(UnifiedType::Class(ClassWithLoader { class_name: class.class_name.clone(), loader: class.loader.clone() }))
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
                locals: expand_to_length(frame.locals.clone(),frame.max_locals as usize,UnifiedType::TopType),
                stack_map: OperandStack::new_prolog_display_order(&frame.stack),
                flag_this_uninit: false,
            },
        });
    }

    return res;
}


pub fn handle_same_locals_1_stack_frame_extended(mut frame: &mut InternalFrame, f: &SameLocals1StackItemFrameExtended) -> () {
    frame.current_offset += f.offset_delta;
    frame.stack.clear();
    push_to_stack(frame, &f.stack);
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

pub fn handle_full_frame(frame: &mut InternalFrame, f: &FullFrame) -> () {
    frame.current_offset += f.offset_delta;
    frame.locals.clear();
    for new_local in f.locals.iter() {
        add_new_local(frame, new_local);
    }

    frame.stack.clear();
    for new_stack_member in f.stack.iter() {
        push_to_stack(frame, new_stack_member);
    }
}

pub fn handle_same_locals_1_stack(frame: &mut InternalFrame, s: &SameLocals1StackItemFrame) -> () {
    frame.current_offset += s.offset_delta;
    frame.stack.clear();
    push_to_stack(frame, &s.stack);
}

pub fn handle_append_frame(frame: &mut InternalFrame, append_frame: &AppendFrame) -> () {
    frame.current_offset += append_frame.offset_delta;
    for new_local in append_frame.locals.iter() {
        add_new_local(frame, new_local)
    }
    frame.stack.clear();
}

pub fn handle_same_frame(frame: &mut InternalFrame, s: &SameFrame) {
    frame.current_offset += s.offset_delta;
    frame.stack.clear();
}


fn push_to_stack(frame: &mut InternalFrame, new_local: &UnifiedType) {
    add_verification_type_to_array(&mut frame.stack, new_local)
}

fn add_new_local(frame: &mut InternalFrame, new_local: &UnifiedType) {
    add_verification_type_to_array(&mut frame.locals, new_local)
}

fn add_verification_type_to_array(locals: &mut Vec<UnifiedType>, new_local: &UnifiedType) -> () {
    match new_local.clone() {
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

