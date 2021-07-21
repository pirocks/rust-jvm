use std::rc::Rc;

use classfile_view::view::constant_info_view::ConstantInfoView;
use rust_jvm_common::compressed_classfile::code::{CInstructionInfo, CompressedLdc2W, CompressedLdcW};
use rust_jvm_common::compressed_classfile::names::CClassName;
use rust_jvm_common::loading::{ClassWithLoader, LoaderName};
use rust_jvm_common::vtype::VType;

use crate::OperandStack;
use crate::verifier::{Frame, get_class, standard_exception_frame};
use crate::verifier::codecorrectness::{Environment, frame_is_assignable, Handler, handler_exception_class, init_handler_is_legal, MergedCodeInstruction, operand_stack_has_legal_length, push_operand_stack, size_of, valid_type_transition};
use crate::verifier::codecorrectness::MergedCodeInstruction::{Instruction, StackMap};
use crate::verifier::filecorrectness::is_assignable;
use crate::verifier::instructions::big_match::instruction_is_type_safe;
use crate::verifier::TypeSafetyError;
use crate::VerifierContext;

pub mod loads;
pub mod consts;
pub mod big_match;
pub mod branches;
pub mod stores;
pub mod special;
pub mod float;

pub struct ResultFrames {
    pub next_frame: Frame,
    pub exception_frame: Frame,
}

pub struct AfterGotoFrames {
    pub exception_frame: Frame,
}

pub enum InstructionTypeSafe {
    Safe(ResultFrames),
    AfterGoto(AfterGotoFrames),
}

#[derive(Debug)]
pub enum FrameResult {
    Regular(Frame),
    AfterGoto,
}

pub fn merged_code_is_type_safe(env: &mut Environment, merged_code: &[MergedCodeInstruction], after_frame: FrameResult) -> Result<(), TypeSafetyError> {
    let first = &merged_code[0];//infinite recursion will not occur becuase we stop when we reach EndOfCode
    let rest = &merged_code[1..merged_code.len()];
    match first {
        MergedCodeInstruction::Instruction(i) => {
            let f = match after_frame {
                FrameResult::Regular(f) => f,
                FrameResult::AfterGoto => {
                    match i.info {
                        CInstructionInfo::EndOfCode => return Result::Ok(()),
                        _ => return Result::Err(TypeSafetyError::NotSafe("No stack frame after unconditional branch".to_string()))
                    }
                }
            };
            match instruction_is_type_safe(&i, env, i.offset, f)? {
                InstructionTypeSafe::Safe(s) => {
                    let ResultFrames { next_frame, exception_frame } = s;
                    let _exception_stack_frame1 = instruction_satisfies_handlers(env, i.offset, &exception_frame)?;
                    merged_code_is_type_safe(env, rest, FrameResult::Regular(next_frame))
                }
                InstructionTypeSafe::AfterGoto(ag) => {
                    let _exception_stack_frame1 = instruction_satisfies_handlers(env, i.offset, &ag.exception_frame)?;
                    merged_code_is_type_safe(env, rest, FrameResult::AfterGoto)
                }
            }
        }
        MergedCodeInstruction::StackMap(s) => {
            let map_frame = Frame {
                locals: s.map_frame.locals.clone(),
                stack_map: s.map_frame.stack_map.clone(),
                flag_this_uninit: s.map_frame.flag_this_uninit,
            };
            match after_frame {
                FrameResult::Regular(f) => {
                    frame_is_assignable(&env.vf, &f, &map_frame)?;
                    merged_code_is_type_safe(env, rest, FrameResult::Regular(map_frame))
                }
                FrameResult::AfterGoto => {
                    merged_code_is_type_safe(env, rest, FrameResult::Regular(map_frame))
                }
            }
        }
    }
}

fn offset_stack_frame(env: &Environment, offset: u16) -> Result<Frame, TypeSafetyError> {
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
                locals: Rc::new(s.map_frame.locals.iter().cloned().collect()),
                stack_map: s.map_frame.stack_map.clone(),
                flag_this_uninit: s.map_frame.flag_this_uninit,
            },
        }
    }) {
        None => { Result::Err(unknown_error_verifying!()) }
        Some(f) => Result::Ok(f),
    }
}

fn target_is_type_safe(env: &Environment, stack_frame: &Frame, target: u16) -> Result<(), TypeSafetyError> {
    let frame = offset_stack_frame(env, target)?;
    frame_is_assignable(&env.vf, stack_frame, &frame)?;
    Result::Ok(())
}

fn instruction_satisfies_handlers(env: &Environment, offset: u16, exception_stack_frame: &Frame) -> Result<(), TypeSafetyError> {
    let handlers = &env.handlers;
    let applicable_handler = handlers.iter().filter(|h| {
        is_applicable_handler(offset, h)
    });
    let res: Result<Vec<_>, _> = applicable_handler.map(|h| {
        instruction_satisfies_handler(env, exception_stack_frame, h)
    }).collect();
    res?;
    Result::Ok(())
}

fn is_applicable_handler(offset: u16, handler: &Handler) -> bool {
    offset >= handler.start && offset < handler.end
}

fn class_to_type(vf: &VerifierContext, class: &ClassWithLoader) -> VType {
    let class_view = get_class(vf, class);
    let class_name = class_view.name();
    class_name.to_verification_type(class.loader)
}

fn instruction_satisfies_handler(env: &Environment, exc_stack_frame: &Frame, handler: &Handler) -> Result<(), TypeSafetyError> {
    let target = handler.target;
    let _class_loader = &env.class_loader;
    let exception_class = handler_exception_class(&env.vf, handler, env.class_loader.clone());
    let locals = &exc_stack_frame.locals;
    let flags = exc_stack_frame.flag_this_uninit;
    let locals_copy = locals.clone();
    let stack_map = OperandStack::new_prolog_display_order(&[class_to_type(&env.vf, &exception_class)]);
    let true_exc_stack_frame = Frame { locals: locals_copy, stack_map: stack_map.clone(), flag_this_uninit: flags };
    if operand_stack_has_legal_length(env, &stack_map) {
        target_is_type_safe(env, &true_exc_stack_frame, target)
    } else {
        Result::Err(TypeSafetyError::NotSafe("operand stack does not have legal length".to_string()))
    }
}

pub fn nth0(index: u16, locals: &[VType]) -> Result<VType, TypeSafetyError> {
    match locals.get(index as usize) {
        None => Err(unknown_error_verifying!()),
        Some(res) => Ok(res.clone()),
    }
}


pub fn handlers_are_legal(env: &Environment) -> Result<(), TypeSafetyError> {
    let handlers = &env.handlers;
    let res: Result<Vec<_>, _> = handlers.iter().map(|h| {
        handler_is_legal(env, h)
    }).collect();
    res?;
    Result::Ok(())
}

pub fn start_is_member_of(start: u16, merged_instructs: &[MergedCodeInstruction]) -> bool {
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
                let exception_class = handler_exception_class(&env.vf, &h, env.class_loader.clone());
                is_assignable(&env.vf, &VType::Class(ClassWithLoader { class_name: exception_class.class_name, loader: env.class_loader.clone() }),
                              &VType::Class(ClassWithLoader { class_name: CClassName::throwable(), loader: LoaderName::BootstrapLoader }))?;
                init_handler_is_legal(env, h)
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


pub fn instructions_include_end(instructs: &[MergedCodeInstruction], end: u16) -> bool {
    instructs.iter().any(|x: &MergedCodeInstruction| {
        match x {
            MergedCodeInstruction::Instruction(i) => {
                i.offset == end
            }
            MergedCodeInstruction::StackMap(_) => false,
        }
    })
}

pub fn instruction_is_type_safe_dup(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let Frame { locals, stack_map: input_operand_stack, flag_this_uninit: flags } = stack_frame;
    let type_ = peek_category1(&env.vf, &input_operand_stack)?;
    let output_operand_stack = can_safely_push(env, input_operand_stack, &type_)?;
    let next_frame = Frame {
        locals: locals.clone(),
        stack_map: output_operand_stack,
        flag_this_uninit: flags,
    };
    standard_exception_frame(locals, flags, next_frame)
}

pub fn can_safely_push(env: &Environment, input_operand_stack: OperandStack, type_: &VType) -> Result<OperandStack, TypeSafetyError> {
    let output_operand_stack = push_operand_stack(&env.vf, input_operand_stack, type_);
    if operand_stack_has_legal_length(env, &output_operand_stack) {
        Result::Ok(output_operand_stack)
    } else {
        Result::Err(unknown_error_verifying!())
    }
}

pub fn pop_category1(vf: &VerifierContext, input: &mut OperandStack) -> Result<VType, TypeSafetyError> {
    if size_of(vf, &input.peek()) == 1 {
        let type_ = input.operand_pop();
        return Result::Ok(type_);
    }
    Result::Err(unknown_error_verifying!())
}

pub fn peek_category1(vf: &VerifierContext, input: &OperandStack) -> Result<VType, TypeSafetyError> {
    if size_of(vf, &input.peek()) == 1 {
        let type_ = input.peek();
        return Result::Ok(type_);
    }
    Result::Err(unknown_error_verifying!())
}


pub fn pop_category2(vf: &VerifierContext, input: &mut OperandStack) -> Result<VType, TypeSafetyError> {
    let top = input.operand_pop();
    assert_eq!(top, VType::TopType);
    if size_of(vf, &input.peek()) == 2 {
        let type_ = input.operand_pop();
        return Result::Ok(type_);
    }
    Result::Err(unknown_error_verifying!())
}

pub fn peek_category2(vf: &VerifierContext, input: &mut OperandStack) -> Result<VType, TypeSafetyError> {
    let top = input.operand_pop();
    // assert_eq!(top, VType::TopType);
    let valid_size = size_of(vf, &input.peek()) == 2;
    let type_ = input.peek();
    input.operand_push(top.clone());
    if valid_size && top == VType::TopType {
        return Result::Ok(type_);
    }
    Result::Err(unknown_error_verifying!())
}

pub fn instruction_is_type_safe_dup_x1(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let Frame { locals, stack_map: input_operand_stack, flag_this_uninit: flags } = stack_frame;
    let mut stack_1 = input_operand_stack;
    let type_1 = pop_category1(&env.vf, &mut stack_1)?;
    let mut rest = stack_1;
    let type_2 = pop_category1(&env.vf, &mut rest)?;
    let output_stack = can_safely_push_list(env, rest, vec![type_1.clone(), type_2, type_1])?;
    let next_frame = Frame {
        locals: locals.clone(),
        stack_map: output_stack,
        flag_this_uninit: flags,
    };
    standard_exception_frame(locals, flags, next_frame)
}

pub fn can_safely_push_list(env: &Environment, input_stack: OperandStack, types: Vec<VType>) -> Result<OperandStack, TypeSafetyError> {
    let output_stack = can_push_list(&env.vf, input_stack, types.as_slice())?;
    if !operand_stack_has_legal_length(env, &output_stack) {
        return Result::Err(unknown_error_verifying!());
    }
    Result::Ok(output_stack)
}

pub fn can_push_list(vf: &VerifierContext, input_stack: OperandStack, types: &[VType]) -> Result<OperandStack, TypeSafetyError> {
    if types.is_empty() {
        return Result::Ok(input_stack);
    }
    let type_ = &types[0];
    let rest = &types[1..];
    let interim_stack = push_operand_stack(vf, input_stack, type_);
    can_push_list(vf, interim_stack, rest)
}

pub fn instruction_is_type_safe_dup2(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let Frame { locals, stack_map: input_stack, flag_this_uninit: flags } = stack_frame;
    let output_stack = dup2_form_is_type_safe(env, input_stack)?;
    let next_frame = Frame {
        locals: locals.clone(),
        stack_map: output_stack,
        flag_this_uninit: flags,
    };
    standard_exception_frame(locals, flags, next_frame)
}


pub fn instruction_is_type_safe_dup_x2(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let Frame { locals, stack_map: input_stack, flag_this_uninit: flags } = stack_frame;
    let output_stack = dup_x2_form_is_type_safe(env, input_stack)?;
    let next_frame = Frame {
        locals: locals.clone(),
        stack_map: output_stack,
        flag_this_uninit: flags,
    };
    standard_exception_frame(locals, flags, next_frame)
}

fn dup_x2_form_is_type_safe(env: &Environment, mut input_stack: OperandStack) -> Result<OperandStack, TypeSafetyError> {
    let temp = input_stack.operand_pop();
    let is_form1 = peek_category2(&env.vf, &mut input_stack).is_err();
    input_stack.operand_push(temp);
    if is_form1 {
        dup_x2_form1_is_type_safe(env, input_stack)
    } else {
        dup_x2_form2_is_type_safe(env, input_stack)
    }
}

fn dup2_form_is_type_safe(env: &Environment, mut input_stack: OperandStack) -> Result<OperandStack, TypeSafetyError> {
    let top = input_stack.operand_pop();
    let is_form2 = size_of(&env.vf, &input_stack.peek()) == 2;
    input_stack.operand_push(top);
    if is_form2 {
        dup2_form2_is_type_safe(env, input_stack)
    } else {
        dup2_form1_is_type_safe(env, input_stack)
    }
}

fn dup2_form1_is_type_safe(env: &Environment, input_stack: OperandStack) -> Result<OperandStack, TypeSafetyError> {
    let mut temp_stack = input_stack;
    let type1 = pop_category1(&env.vf, &mut temp_stack)?;
    let mut stack2 = temp_stack;
    let type2 = pop_category1(&env.vf, &mut stack2)?;
    stack2.operand_push(type2.clone());
    stack2.operand_push(type1.clone());
    let original_stack = stack2;
    can_safely_push_list(env, original_stack, vec![type2, type1])
}

fn dup2_form2_is_type_safe(env: &Environment, input_stack: OperandStack) -> Result<OperandStack, TypeSafetyError> {
    let mut stack1 = input_stack.clone();
    let type_ = pop_category2(&env.vf, &mut stack1)?;
    can_safely_push_list(env, input_stack, vec![type_])
}

fn dup_x2_form1_is_type_safe(env: &Environment, input_stack: OperandStack) -> Result<OperandStack, TypeSafetyError> {
    let mut stack1 = input_stack;
    let type1 = pop_category1(&env.vf, &mut stack1)?;
    let mut stack2 = stack1;
    let type2 = pop_category1(&env.vf, &mut stack2)?;
    let mut rest = stack2;
    let type3 = pop_category1(&env.vf, &mut rest)?;
    can_safely_push_list(env, rest, vec![type1.clone(), type3, type2, type1])
}

fn dup_x2_form2_is_type_safe(env: &Environment, input_stack: OperandStack) -> Result<OperandStack, TypeSafetyError> {
    let mut stack1 = input_stack;
    let type1 = pop_category1(&env.vf, &mut stack1)?;
    let mut rest = stack1;
    let type2 = pop_category2(&env.vf, &mut rest)?;
    can_safely_push_list(env, rest, vec![type1.clone(), type2, type1])
}


//
//instructionIsTypeSafe(dup2_x1, Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
//StackFrame = frame(Locals, InputOperandStack, Flags),
//dup2_x1FormIsTypeSafe(Environment, InputOperandStack, OutputOperandStack),
//NextStackFrame = frame(Locals, OutputOperandStack, Flags),
//exceptionStackFrame(StackFrame, ExceptionStackFrame).
pub fn instruction_is_type_safe_dup2_x1(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let Frame { locals, stack_map: input_stack, flag_this_uninit: flags } = stack_frame;
    let output = dup2_x1form_is_type_safe(env, input_stack)?;
    let next_frame = Frame {
        locals: locals.clone(),
        stack_map: output,
        flag_this_uninit: flags,
    };
    standard_exception_frame(locals, flags, next_frame)
}

pub fn dup2_x1form_is_type_safe(env: &Environment, input_frame: OperandStack) -> Result<OperandStack, TypeSafetyError> {
    dup2_x1form1_is_type_safe(env, input_frame.clone()).or_else(|_| {
        dup2_x1form2_is_type_safe(env, input_frame)
    })
}

//dup2_x1Form1IsTypeSafe(Environment, InputOperandStack, OutputOperandStack) :-
//popCategory1(InputOperandStack, Type1, Stack1),
//popCategory1(Stack1, Type2, Stack2),
//popCategory1(Stack2, Type3, Rest),
//canSafelyPushList(Environment, Rest, [Type2, Type1, Type3, Type2, Type1],OutputOperandStack).
pub fn dup2_x1form1_is_type_safe(env: &Environment, input_frame: OperandStack) -> Result<OperandStack, TypeSafetyError> {
    let mut stack1 = input_frame;
    let type1 = pop_category1(&env.vf, &mut stack1)?;
    let mut stack2 = stack1;
    let type2 = pop_category1(&env.vf, &mut stack2)?;
    let mut rest = stack2;
    let type3 = pop_category1(&env.vf, &mut rest)?;
    can_safely_push_list(env, rest, vec![type2.clone(), type1.clone(), type3, type2, type1])
}

//dup2_x1Form2IsTypeSafe(Environment, InputOperandStack, OutputOperandStack) :-
//popCategory2(InputOperandStack, Type1, Stack1),
//popCategory1(Stack1, Type2, Rest),
//canSafelyPushList(Environment, Rest, [Type1, Type2, Type1],OutputOperandStack).
pub fn dup2_x1form2_is_type_safe(env: &Environment, input_frame: OperandStack) -> Result<OperandStack, TypeSafetyError> {
    let mut stack1 = input_frame;
    let type1 = pop_category2(&env.vf, &mut stack1)?;
    let mut rest = stack1;
    let type2 = pop_category1(&env.vf, &mut rest)?;
    can_safely_push_list(env, rest, vec![type1.clone(), type2, type1])
}


//
//dup2_x1FormIsTypeSafe(Environment, InputOperandStack, OutputOperandStack) :-
//dup2_x1Form1IsTypeSafe(Environment, InputOperandStack, OutputOperandStack).
//
//dup2_x1FormIsTypeSafe(Environment, InputOperandStack, OutputOperandStack) :-
//dup2_x1Form2IsTypeSafe(Environment, InputOperandStack, OutputOperandStack).
//

//


// instructionIsTypeSafe(dup2_x2, Environment, _Offset, StackFrame,NextStackFrame, ExceptionStackFrame) :-
// StackFrame = frame(Locals, InputOperandStack, Flags),
// dup2_x2FormIsTypeSafe(Environment, InputOperandStack, OutputOperandStack),
// NextStackFrame = frame(Locals, OutputOperandStack, Flags),
// exceptionStackFrame(StackFrame, ExceptionStackFrame).
//


pub fn instruction_is_type_safe_dup2_x2(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let Frame { locals, stack_map: input_stack, flag_this_uninit: flags } = stack_frame;
    let output = dup2_x2form_is_type_safe(env, input_stack)?;
    let next_frame = Frame {
        locals: locals.clone(),
        stack_map: output,
        flag_this_uninit: flags,
    };
    standard_exception_frame(locals, flags, next_frame)
}

// dup2_x2FormIsTypeSafe(Environment, InputOperandStack, OutputOperandStack) :-
// dup2_x2Form1IsTypeSafe(Environment, InputOperandStack, OutputOperandStack).
//
// dup2_x2FormIsTypeSafe(Environment, InputOperandStack, OutputOperandStack) :-
// dup2_x2Form2IsTypeSafe(Environment, InputOperandStack, OutputOperandStack).
//
// dup2_x2FormIsTypeSafe(Environment, InputOperandStack, OutputOperandStack) :-
// dup2_x2Form3IsTypeSafe(Environment, InputOperandStack, OutputOperandStack).
//
// dup2_x2FormIsTypeSafe(Environment, InputOperandStack, OutputOperandStack) :-
// dup2_x2Form4IsTypeSafe(Environment, InputOperandStack, OutputOperandStack).
//

pub fn dup2_x2form_is_type_safe(env: &Environment, input_frame: OperandStack) -> Result<OperandStack, TypeSafetyError> {
    dup2_x2form1_is_type_safe(env, input_frame.clone()).or_else(|_| {
        dup2_x2form2_is_type_safe(env, input_frame.clone()).or_else(|_| {
            dup2_x2form3_is_type_safe(env, input_frame.clone()).or_else(|_| {
                dup2_x2form4_is_type_safe(env, input_frame)
            })
        })
    })
}

// dup2_x2Form1IsTypeSafe(Environment, InputOperandStack, OutputOperandStack) :-
// popCategory1(InputOperandStack, Type1, Stack1),
// popCategory1(Stack1, Type2, Stack2),
// popCategory1(Stack2, Type3, Stack3),
// popCategory1(Stack3, Type4, Rest),
// canSafelyPushList(Environment, Rest,[Type2, Type1, Type4, Type3, Type2, Type1],OutputOperandStack).
pub fn dup2_x2form1_is_type_safe(env: &Environment, mut input_stack: OperandStack) -> Result<OperandStack, TypeSafetyError> {
    let type_1 = pop_category1(&env.vf, &mut input_stack)?;
    let type_2 = pop_category1(&env.vf, &mut input_stack)?;
    let type_3 = pop_category1(&env.vf, &mut input_stack)?;
    let type_4 = pop_category1(&env.vf, &mut input_stack)?;
    can_safely_push_list(env, input_stack, vec![type_2.clone(), type_1.clone(), type_4, type_3, type_2, type_1])
}


// dup2_x2Form2IsTypeSafe(Environment, InputOperandStack, OutputOperandStack) :-
// popCategory2(InputOperandStack, Type1, Stack1),
// popCategory1(Stack1, Type2, Stack2),
// popCategory1(Stack2, Type3, Rest),
// canSafelyPushList(Environment, Rest,[Type1, Type3, Type2, Type1],OutputOperandStack).

pub fn dup2_x2form2_is_type_safe(env: &Environment, mut input_stack: OperandStack) -> Result<OperandStack, TypeSafetyError> {
    let type_1 = pop_category2(&env.vf, &mut input_stack)?;
    let type_2 = pop_category1(&env.vf, &mut input_stack)?;
    let type_3 = pop_category1(&env.vf, &mut input_stack)?;
    can_safely_push_list(env, input_stack, vec![type_1.clone(), type_3, type_2, type_1])
}

// dup2_x2Form3IsTypeSafe(Environment, InputOperandStack, OutputOperandStack) :-
// popCategory1(InputOperandStack, Type1, Stack1),
// popCategory1(Stack1, Type2, Stack2),
// popCategory2(Stack2, Type3, Rest),
// canSafelyPushList(Environment, Rest,[Type2, Type1, Type3, Type2, Type1],OutputOperandStack).

pub fn dup2_x2form3_is_type_safe(env: &Environment, mut input_stack: OperandStack) -> Result<OperandStack, TypeSafetyError> {
    let type_1 = pop_category1(&env.vf, &mut input_stack)?;
    let type_2 = pop_category1(&env.vf, &mut input_stack)?;
    let type_3 = pop_category2(&env.vf, &mut input_stack)?;
    can_safely_push_list(env, input_stack, vec![type_2.clone(), type_1.clone(), type_3, type_2, type_1])
}

// dup2_x2Form4IsTypeSafe(Environment, InputOperandStack, OutputOperandStack) :-
// popCategory2(InputOperandStack, Type1, Stack1),
// popCategory2(Stack1, Type2, Rest),
// canSafelyPushList(Environment, Rest, [Type1, Type2, Type1],OutputOperandStack).

pub fn dup2_x2form4_is_type_safe(env: &Environment, mut input_stack: OperandStack) -> Result<OperandStack, TypeSafetyError> {
    let type_1 = pop_category2(&env.vf, &mut input_stack)?;
    let type_2 = pop_category2(&env.vf, &mut input_stack)?;
    can_safely_push_list(env, input_stack, vec![type_1.clone(), type_2, type_1])
}


pub fn instruction_is_type_safe_i2d(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env, stack_frame, vec![VType::IntType], VType::DoubleType)
}

pub fn instruction_is_type_safe_i2f(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env, stack_frame, vec![VType::IntType], VType::FloatType)
}

pub fn instruction_is_type_safe_i2l(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env, stack_frame, vec![VType::IntType], VType::LongType)
}

pub fn type_transition(env: &Environment, stack_frame: Frame, expected_types: Vec<VType>, res_type: VType) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let locals = stack_frame.locals.clone();
    let flag = stack_frame.flag_this_uninit;
    let next_frame = valid_type_transition(env, expected_types, res_type, stack_frame)?;
    standard_exception_frame(locals, flag, next_frame)
}

pub fn instruction_is_type_safe_iadd(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env, stack_frame, vec![VType::IntType, VType::IntType], VType::IntType)
}

pub fn instruction_is_type_safe_iinc(index: u16, _env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let locals = &stack_frame.locals;
    let should_be_int = nth0(index, locals)?;
    match should_be_int {
        VType::IntType => {
            Result::Ok(InstructionTypeSafe::Safe(ResultFrames {
                next_frame: Frame {
                    locals: stack_frame.locals.clone(),
                    stack_map: stack_frame.stack_map.clone(),
                    flag_this_uninit: stack_frame.flag_this_uninit,
                },
                exception_frame: exception_stack_frame(locals.clone(), stack_frame.flag_this_uninit),
            }))
        }
        _ => {
            Result::Err(unknown_error_verifying!())
        }
    }
}

pub fn instruction_is_type_safe_ineg(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env, stack_frame, vec![VType::IntType], VType::IntType)
}

pub fn instruction_is_type_safe_l2d(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env, stack_frame, vec![VType::LongType], VType::DoubleType)
}

pub fn instruction_is_type_safe_l2f(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env, stack_frame, vec![VType::LongType], VType::FloatType)
}

pub fn instruction_is_type_safe_l2i(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env, stack_frame, vec![VType::LongType], VType::IntType)
}

pub fn instruction_is_type_safe_ladd(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env, stack_frame, vec![VType::LongType, VType::LongType], VType::LongType)
}

fn instruction_is_type_safe_lcmp(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env, stack_frame, vec![VType::LongType, VType::LongType], VType::IntType)
}


pub fn loadable_constant(vf: &VerifierContext, c: &CompressedLdcW) -> VType {
    match c {
        CompressedLdcW::Integer { .. } => VType::IntType,
        CompressedLdcW::Float { .. } => VType::FloatType,
        CompressedLdcW::Class { .. } => {
            let class_name = CClassName::class();
            VType::Class(ClassWithLoader { class_name, loader: vf.current_loader.clone() })
        }
        CompressedLdcW::String { .. } => {
            let class_name = CClassName::string();
            VType::Class(ClassWithLoader { class_name, loader: vf.current_loader.clone() })
        }
        CompressedLdcW::MethodHandle {} => VType::Class(ClassWithLoader { class_name: CClassName::method_handle(), loader: vf.current_loader.clone() }),
        CompressedLdcW::MethodType {} => VType::Class(ClassWithLoader { class_name: CClassName::method_type(), loader: vf.current_loader.clone() }),
        CompressedLdcW::LiveObject(idx) => {
            vf.live_pool_getter.elem_type(*idx).to_verification_type(vf.current_loader)
        }
    }
}

pub fn loadable_constant_w(vf: &VerifierContext, c: &CompressedLdc2W) -> VType {
    match c {
        CompressedLdc2W::Long(_) => VType::LongType,
        CompressedLdc2W::Double(_) => VType::DoubleType,

        _ => {
            panic!()
        }
    }
}

// pub fn instruction_is_type_safe_ldc(const_: &CompressedLdc2W, env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
//     let view = get_class(&env.vf, &env.method.class);
//     let type_: VType = loadable_constant_w(&env.vf, const_);
//     match type_ {
//         VType::DoubleType => { return Result::Err(unknown_error_verifying!()); }
//         VType::LongType => { return Result::Err(unknown_error_verifying!()); }
//         _ => todo!()
//     };
//     type_transition(env, stack_frame, vec![], type_)
// }

pub fn instruction_is_type_safe_ldc_w(const_: &CompressedLdcW, env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let view = get_class(&env.vf, &env.method.class);
    let type_ = match const_ {
        CompressedLdcW::Integer { .. } => VType::IntType,
        CompressedLdcW::Float { .. } => VType::FloatType,
        CompressedLdcW::Class { .. } => VType::Class(ClassWithLoader { class_name: CClassName::class(), loader: env.vf.current_loader.clone() }),
        CompressedLdcW::String { .. } => VType::Class(ClassWithLoader { class_name: CClassName::string(), loader: env.vf.current_loader.clone() }),
        CompressedLdcW::MethodType {} => VType::Class(ClassWithLoader { class_name: CClassName::method_type(), loader: env.vf.current_loader.clone() }),
        _ => panic!()
    };
    type_transition(env, stack_frame, vec![], type_)
}

pub fn instruction_is_type_safe_ldc2_w(const_: &CompressedLdc2W, env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let type_: VType = loadable_constant_w(&env.vf, const_);
    match type_ {
        VType::DoubleType => {}
        VType::LongType => {}
        _ => { return Result::Err(unknown_error_verifying!()); }
    };
    type_transition(env, stack_frame, vec![], type_)
}

pub fn instruction_is_type_safe_lneg(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env, stack_frame, vec![VType::LongType], VType::LongType)
}

pub fn instruction_is_type_safe_lshl(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env, stack_frame, vec![VType::IntType, VType::LongType], VType::LongType)
}

pub fn instruction_is_type_safe_pop(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let Frame { locals, stack_map: mut rest, flag_this_uninit: flags } = stack_frame;
    let _type_ = pop_category1(&env.vf, &mut rest)?;
    let next_frame = Frame {
        locals: locals.clone(),
        stack_map: rest,
        flag_this_uninit: flags,
    };
    standard_exception_frame(locals, flags, next_frame)
}

pub fn instruction_is_type_safe_pop2(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let Frame { locals, stack_map: operand_stack, flag_this_uninit: flags } = stack_frame;
    let out = pop2form_is_type_safe(env, operand_stack)?;
    let next_frame = Frame {
        locals: locals.clone(),
        stack_map: out,
        flag_this_uninit: flags,
    };
    standard_exception_frame(locals, flags, next_frame)
}

fn pop2form_is_type_safe(env: &Environment, mut input: OperandStack) -> Result<OperandStack, TypeSafetyError> {
    let first_pop = input.operand_pop();
    let second_pop = input.operand_pop();
    let succeds = match first_pop {
        VType::TopType => {
            size_of(&env.vf, &second_pop) == 2
        }
        _ => {
            size_of(&env.vf, &first_pop) == 1 &&
                size_of(&env.vf, &second_pop) == 1
        }
    };
    if succeds {
        Result::Ok(input)
    } else {
        Result::Err(unknown_error_verifying!())
    }
}

pub fn instruction_is_type_safe_sipush(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env, stack_frame, vec![], VType::IntType)
}


pub fn instruction_is_type_safe_nop(stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let Frame { locals, stack_map, flag_this_uninit } = stack_frame;
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames {
        next_frame: Frame {
            locals: locals.clone(),
            stack_map,
            flag_this_uninit,
        },
        exception_frame: exception_stack_frame(locals.clone(), flag_this_uninit),
    }))
}

pub fn instruction_is_type_safe_swap(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let locals = &stack_frame.locals;
    let flags = stack_frame.flag_this_uninit;
    let mut initial_stack_map = stack_frame.stack_map.clone();
    let type_1 = initial_stack_map.operand_pop();
    let type_2 = initial_stack_map.operand_pop();
    if size_of(&env.vf, &type_1) == 1 && size_of(&env.vf, &type_2) == 1 {
        initial_stack_map.operand_push(type_1);
        initial_stack_map.operand_push(type_2);
        Result::Ok(InstructionTypeSafe::Safe(ResultFrames {
            next_frame: Frame {
                locals: locals.clone(),
                stack_map: initial_stack_map,
                flag_this_uninit: flags,
            },
            exception_frame: exception_stack_frame(locals.clone(), flags),
        }))
    } else {
        Result::Err(unknown_error_verifying!())
    }
}

pub fn exception_stack_frame(stack_frame_locals: Rc<Vec<VType>>, stack_frame_flag: bool) -> Frame {
    Frame { locals: stack_frame_locals, stack_map: OperandStack::empty(), flag_this_uninit: stack_frame_flag }
}

