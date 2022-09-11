use std::rc::Rc;

use classfile_view::view::HasAccessFlags;
use classfile_view::view::method_view::MethodView;
use rust_jvm_common::ByteCodeOffset;
use rust_jvm_common::classfile::{AttributeType, Code, SameFrame, StackMapTable};
use rust_jvm_common::compressed_classfile::CPDType;
use rust_jvm_common::compressed_classfile::code::{CompressedAppendFrame, CompressedChopFrame, CompressedFullFrame, CompressedSameFrameExtended, CompressedSameLocals1StackItemFrame, CompressedSameLocals1StackItemFrameExtended, CompressedStackMapFrame};
use rust_jvm_common::loading::*;
use rust_jvm_common::vtype::VType;

use crate::{StackMap, VerifierContext};
use crate::OperandStack;
use crate::verifier::{Frame, InternalFrame};
use crate::verifier::codecorrectness::expand_to_length;

pub fn stack_map_table_attribute(code: &Code) -> Option<&StackMapTable> {
    for attr in code.attributes.iter() {
        if let AttributeType::StackMapTable(table) = &attr.attribute_type {
            return Some(table);
        }
    }
    None
}

pub fn get_stack_map_frames(_vf: &VerifierContext, class: &ClassWithLoader, method_info: &MethodView) -> Vec<StackMap> {
    let mut res = vec![];
    let code = method_info.code_attribute().expect("This method won't be called for a non-code attribute function. If you see this , this is a bug");
    let parsed_descriptor = method_info.desc();
    let stack_map = &code.stack_map_table;
    let this_pointer = if method_info.is_static() { None } else { Some(CPDType::Class(class.class_name)) };
    let mut frame = init_frame(parsed_descriptor.arg_types.clone(), this_pointer, code.max_locals);

    let mut previous_frame_is_first_frame = true;
    for (_, entry) in stack_map.iter().enumerate() {
        match entry {
            CompressedStackMapFrame::SameFrame(s) => handle_same_frame(&mut frame, &s),
            CompressedStackMapFrame::AppendFrame(append_frame) => handle_append_frame(&mut frame, &append_frame),
            CompressedStackMapFrame::SameLocals1StackItemFrame(s) => handle_same_locals_1_stack(&mut frame, &s),
            CompressedStackMapFrame::FullFrame(f) => handle_full_frame(&mut frame, &f),
            CompressedStackMapFrame::ChopFrame(f) => handle_chop_frame(&mut frame, &f),
            CompressedStackMapFrame::SameFrameExtended(f) => handle_same_frame_extended(&mut frame, &f),
            CompressedStackMapFrame::SameLocals1StackItemFrameExtended(f) => handle_same_locals_1_stack_frame_extended(&mut frame, &f),
        }
        if previous_frame_is_first_frame {
            previous_frame_is_first_frame = false;
        } else {
            frame.current_offset.0 += 1;
        }
        res.push(StackMap {
            offset: frame.current_offset,
            map_frame: Frame {
                locals: Rc::new(expand_to_length(frame.locals.clone(), frame.max_locals as usize, VType::TopType).iter().cloned().collect()),
                stack_map: OperandStack::new_prolog_display_order(&frame.stack.iter().cloned().collect::<Vec<_>>()),
                flag_this_uninit: false,
            },
        });
    }

    res
}

pub fn handle_same_locals_1_stack_frame_extended(mut frame: &mut InternalFrame, f: &CompressedSameLocals1StackItemFrameExtended) {
    frame.current_offset.0 += f.offset_delta;
    frame.stack.clear();
    add_verification_type_to_array_convert(&mut frame.stack, &f.stack);
}

pub fn handle_same_frame_extended(mut frame: &mut InternalFrame, f: &CompressedSameFrameExtended) {
    frame.current_offset.0 += f.offset_delta;
    frame.stack.clear();
}

pub fn handle_chop_frame(mut frame: &mut InternalFrame, f: &CompressedChopFrame) {
    frame.current_offset.0 += f.offset_delta;
    frame.stack.clear();
    for _ in 0..f.k_frames_to_chop {
        //so basically what's going on here is we want to remove [Double|Long, top],[any type including top]
        let removed = frame.locals.pop().unwrap();
        match removed {
            VType::DoubleType | VType::LongType => panic!(),
            VType::TopType => {
                let second_removed_maybe = frame.locals.pop();
                if let Some(second_removed) = second_removed_maybe {
                    match second_removed {
                        VType::DoubleType | VType::LongType => {}
                        _ => frame.locals.push(second_removed),
                    }
                }
            }
            _ => {}
        }
    }
}

pub fn handle_full_frame(frame: &mut InternalFrame, f: &CompressedFullFrame) {
    frame.current_offset.0 += f.offset_delta;
    frame.locals.clear();
    for new_local in f.locals.iter() {
        add_verification_type_to_array_convert(&mut frame.locals, &new_local);
    }

    frame.stack.clear();
    for new_stack_member in f.stack.iter() {
        add_verification_type_to_array_convert(&mut frame.stack, &new_stack_member);
    }
}

pub fn handle_same_locals_1_stack(frame: &mut InternalFrame, s: &CompressedSameLocals1StackItemFrame) {
    frame.current_offset.0 += s.offset_delta;
    frame.stack.clear();
    add_verification_type_to_array_convert(&mut frame.stack, &s.stack);
}

pub fn handle_append_frame(frame: &mut InternalFrame, append_frame: &CompressedAppendFrame) {
    frame.current_offset.0 += append_frame.offset_delta;
    for new_local in append_frame.locals.iter() {
        add_verification_type_to_array_convert(&mut frame.locals, &new_local)
    }
    frame.stack.clear();
}

pub fn handle_same_frame(frame: &mut InternalFrame, s: &SameFrame) {
    frame.current_offset.0 += s.offset_delta;
    frame.stack.clear();
}

fn add_verification_type_to_array_convert(locals: &mut Vec<VType>, new_local: &VType) {
    match new_local.clone() {
        VType::DoubleType => {
            locals.push(VType::DoubleType);
            locals.push(VType::TopType);
        }
        VType::LongType => {
            locals.push(VType::LongType);
            locals.push(VType::TopType);
        }
        new => {
            locals.push(new);
        }
    }
}

pub fn init_frame(parameter_types: Vec<CPDType>, this_pointer: Option<CPDType>, max_locals: u16) -> InternalFrame {
    let mut locals = Vec::with_capacity(max_locals as usize);
    match this_pointer {
        None => {} //class is static etc.
        Some(t) => add_verification_type_to_array_convert(&mut locals, &VType::UninitializedThisOrClass(t)),
    }
    //so these parameter types come unconverted and therefore need conversion
    for parameter_type in parameter_types {
        add_verification_type_to_array_convert(&mut locals, &parameter_type.to_verification_type(LoaderName::BootstrapLoader))
        //todo fix bootstrap loader
    }
    InternalFrame { max_locals, locals, stack: Vec::new(), current_offset: ByteCodeOffset(0) }
}