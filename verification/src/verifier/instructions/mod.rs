use crate::verifier::codecorrectness::{Environment, MergedCodeInstruction, frame_is_assignable, operand_stack_has_legal_length, valid_type_transition,  handler_exception_class, Handler};
use rust_jvm_common::classfile::{InstructionInfo};
use crate::verifier::{Frame, get_class};
use crate::verifier::instructions::big_match::instruction_is_type_safe;
use crate::verifier::codecorrectness::MergedCodeInstruction::{StackMap, Instruction};
use rust_jvm_common::unified_types::{UnifiedType, ClassWithLoader};
use rust_jvm_common::classnames::{ClassName, NameReference, class_name};
use std::sync::Arc;
use crate::verifier::filecorrectness::is_assignable;
use rust_jvm_common::loading::BOOTSTRAP_LOADER;
use crate::verifier::TypeSafetyError;

pub mod loads;
pub mod consts;
pub mod big_match;
pub mod branches;
pub mod stores;
pub mod special;

pub struct ResultFrames {
    pub next_frame: Frame,
    pub exception_frame: Frame,
}

pub struct AfterGotoFrames {
    pub exception_frame: Frame,
}

pub enum InstructionIsTypeSafeResult {
    Safe(ResultFrames),
    AfterGoto(AfterGotoFrames),
}

pub enum FrameResult<'l> {
    Regular(&'l Frame),
    AfterGoto,
}

//todo how to handle other values here
pub fn merged_code_is_type_safe<'l>(env: &Environment, merged_code: &[MergedCodeInstruction], after_frame: FrameResult<'l>) -> Result<(), TypeSafetyError> {
    let first = &merged_code[0];//todo infinite recursion
    let rest = &merged_code[1..merged_code.len()];
    match first {
        MergedCodeInstruction::Instruction(i) => {
            let f = match after_frame {
                FrameResult::Regular(f) => f,
                FrameResult::AfterGoto => {
                    match i.instruction {
                        InstructionInfo::EndOfCode => return Result::Ok(()),
                        _ => return Result::Err(TypeSafetyError::NotSafe("No stack frame after unconditional branch".to_string()))
                    }
                }
            };
            match instruction_is_type_safe(&i, env, i.offset, f)? {
                InstructionIsTypeSafeResult::Safe(s) => {
                    let _exception_stack_frame1 = instruction_satisfies_handlers(env, i.offset, &s.exception_frame)?;
                    merged_code_is_type_safe(env, rest, FrameResult::Regular(&s.next_frame))
                }
                InstructionIsTypeSafeResult::AfterGoto(ag) => {
                    let _exception_stack_frame1 = instruction_satisfies_handlers(env, i.offset, &ag.exception_frame)?;
                    merged_code_is_type_safe(env, rest, FrameResult::AfterGoto)
                }
            }
        }
        MergedCodeInstruction::StackMap(s) => {
            match after_frame {
                FrameResult::Regular(f) => {
                    frame_is_assignable(f, &s.map_frame)?;
                    merged_code_is_type_safe(env, rest, FrameResult::Regular(&s.map_frame))
                }
                FrameResult::AfterGoto => {
                    merged_code_is_type_safe(env, rest, FrameResult::Regular(&s.map_frame))
                }
            }
        }
    }
}

fn offset_stack_frame(env: &Environment, offset: usize) -> Result<Frame, TypeSafetyError> {
    match env.merged_code.unwrap().iter().find(|x| {
        match x {
            Instruction(_) => false,
            StackMap(s) => {
                s.offset == offset
            }
        }
    }).map(|x| {
        match x {
            Instruction(_) => panic!(),
            StackMap(s) => Frame {
                locals: s.map_frame.locals.iter().map(|x| x.clone()).collect(),
                stack_map: s.map_frame.stack_map.iter().map(|x| x.clone()).collect(),
                flag_this_uninit: s.map_frame.flag_this_uninit,
            },
        }
    }) {
        None => { Result::Err(TypeSafetyError::NotSafe(format!("todo message:{}",line!()).to_string())) }//todo msg
        Some(f) => Result::Ok(f),
    }
}

fn target_is_type_safe(env: &Environment, stack_frame: &Frame, target: usize) -> Result<(), TypeSafetyError> {
    let frame = offset_stack_frame(env, target)?;
//        None => { return TypeSafetyResult::NotSafe("No frame fround at target".to_string()); }
    frame_is_assignable(stack_frame, &frame)?;
    Result::Ok(())
}

fn instruction_satisfies_handlers(env: &Environment, offset: usize, exception_stack_frame: &Frame) -> Result<(), TypeSafetyError> {
    let handlers = &env.handlers;
    let applicable_handler = handlers.iter().filter(|h| {
        is_applicable_handler(offset as usize, h)
    });
    let res: Result<Vec<_>, _> = applicable_handler.map(|h| {
        instruction_satisfies_handler(env, exception_stack_frame, h)
    }).collect();
    res?;
    Result::Ok(())
}

fn is_applicable_handler(offset: usize, handler: &Handler) -> bool {
    offset <= handler.start && offset < handler.end
}

fn class_to_type(class: &ClassWithLoader) -> UnifiedType {
    let class_name = ClassName::Ref(NameReference {
        index: get_class(class).this_class,
        class_file: Arc::downgrade(&get_class(class)),
    });
    UnifiedType::Class(ClassWithLoader { class_name, loader: class.loader.clone() })
}

fn instruction_satisfies_handler(env: &Environment, exc_stack_frame: &Frame, handler: &Handler) -> Result<(), TypeSafetyError> {
    let target = handler.target;
    let _class_loader = &env.class_loader;
    let exception_class = handler_exception_class(handler);
    let locals = &exc_stack_frame.locals;
    let flags = exc_stack_frame.flag_this_uninit;
    let locals_copy = locals.iter().map(|x| { x.clone() }).collect();
    let true_exc_stack_frame = Frame { locals: locals_copy, stack_map: vec![class_to_type(&exception_class)], flag_this_uninit: flags };
    if operand_stack_has_legal_length(env, &vec![class_to_type(&exception_class)]) {
        target_is_type_safe(env, &true_exc_stack_frame, target)
    } else {
        Result::Err(TypeSafetyError::NotSafe("operand stack does not have legal length".to_string()))
    }
}

pub fn nth0(index: usize, locals: &Vec<UnifiedType>) -> UnifiedType {
    match locals.get(index) {
        None => unimplemented!(),
        Some(res) => res.clone(),
    }
}


pub fn handers_are_legal(env: &Environment) -> Result<(), TypeSafetyError> {
    let handlers = &env.handlers;
    let res: Result<Vec<_>, _> = handlers.iter().map(|h| {
        handler_is_legal(env, h)
    }).collect();
    res?;
    Result::Ok(())
}

pub fn start_is_member_of(start: usize, merged_instructs: &Vec<MergedCodeInstruction>) -> bool {
    merged_instructs.iter().any(|m| match m {
        Instruction(i) => { i.offset == start }
        StackMap(s) => { s.offset == start }
    })
}

pub fn handler_is_legal(env: &Environment, h: &Handler) -> Result<(), TypeSafetyError> {
    if h.start < h.end {
        if start_is_member_of(h.start, env.merged_code.unwrap()) {
            let _target_stack_frame = offset_stack_frame(env, h.target)?;
            if instructions_include_end(env.merged_code.unwrap(), h.end) {
                let exception_class = handler_exception_class(&h);
                //todo how does bootstrap loader from throwable make its way into this
                let class_name = class_name(&get_class(&exception_class));
                let assignable = is_assignable(&UnifiedType::Class(ClassWithLoader { class_name, loader: env.class_loader.clone() }),
                                               &UnifiedType::Class(ClassWithLoader { class_name: ClassName::Str("java/lang/Throwable".to_string()), loader: BOOTSTRAP_LOADER.clone() }));
                assignable?;
                Result::Ok(())
            } else {
                Result::Err(TypeSafetyError::NotSafe("Instructions do not include handler end".to_string()))
            }
        } else {
            Result::Err(TypeSafetyError::NotSafe("No instruction found at handler start.".to_string()))
        }
    } else {
        Result::Err(TypeSafetyError::NotSafe("Handler start not less than end".to_string()))
    }
}


pub fn instructions_include_end(_instructs: &Vec<MergedCodeInstruction>, _end: usize) -> bool {
    unimplemented!()
}

//
//#[allow(unused)]
//fn instruction_is_type_safe_dup(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_dup_x1(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_dup_x2(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_dup2(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_dup2_x1(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_dup2_x2(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//
//#[allow(unused)]
//fn instruction_is_type_safe_i2d(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_i2f(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_iadd(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//
//#[allow(unused)]
//fn instruction_is_type_safe_iinc(index: usize, value: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//
//#[allow(unused)]
//fn instruction_is_type_safe_ineg(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//
//
//#[allow(unused)]
//fn instruction_is_type_safe_l2d(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_l2f(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_l2i(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_ladd(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
fn instruction_is_type_safe_lcmp(env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionIsTypeSafeResult, TypeSafetyError> {
    let next_frame = valid_type_transition(env, vec![UnifiedType::LongType, UnifiedType::LongType], &UnifiedType::IntType, stack_frame)?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionIsTypeSafeResult::Safe(ResultFrames { next_frame, exception_frame }))
}

//
//#[allow(unused)]
//fn instruction_is_type_safe_ldc(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_ldc2_w(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//
//#[allow(unused)]
//fn instruction_is_type_safe_lneg(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}

//
//#[allow(unused)]
//fn instruction_is_type_safe_lshl(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//
//
//#[allow(unused)]
//fn instruction_is_type_safe_nop(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_pop(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_pop2(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//

//#[allow(unused)]
//fn instruction_is_type_safe_sipush(value: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_swap(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//


#[allow(unused)]
fn different_package_name(class1: &ClassWithLoader, class2: &ClassWithLoader) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn same_package_name(class1: &ClassWithLoader, class2: &ClassWithLoader) -> bool {
    unimplemented!()
}


pub fn exception_stack_frame(f: &Frame) -> Frame {
    Frame { locals: f.locals.iter().map(|x| x.clone()).collect(), stack_map: vec![], flag_this_uninit: f.flag_this_uninit }
}

