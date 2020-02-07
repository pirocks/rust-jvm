use crate::verifier::codecorrectness::{Environment, MergedCodeInstruction, frame_is_assignable, operand_stack_has_legal_length, valid_type_transition, handler_exception_class, Handler, size_of, push_operand_stack};
use rust_jvm_common::classfile::{InstructionInfo, ConstantKind};
use crate::verifier::{Frame, get_class, standard_exception_frame};
use crate::verifier::instructions::big_match::instruction_is_type_safe;
use crate::verifier::codecorrectness::MergedCodeInstruction::{StackMap, Instruction};
use rust_jvm_common::unified_types::ClassWithLoader;
use rust_jvm_common::classnames::{ClassName, class_name};
use crate::verifier::filecorrectness::is_assignable;
use crate::verifier::TypeSafetyError;
use rust_jvm_common::classfile::CPIndex;
use crate::VerifierContext;
use crate::OperandStack;
use rust_jvm_common::unified_types::VType;

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

pub enum FrameResult<'l> {
    Regular(&'l Frame),
    AfterGoto,
}

//todo how to handle other values here
pub fn merged_code_is_type_safe<'l>(env: &Environment, merged_code: &[MergedCodeInstruction], after_frame: FrameResult<'l>) -> Result<(), TypeSafetyError> {
    let first = &merged_code[0];//infinite recursion will not occur becuase we stop when we reach EndOfCode
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
                InstructionTypeSafe::Safe(s) => {
                    let _exception_stack_frame1 = instruction_satisfies_handlers(env, i.offset, &s.exception_frame)?;
                    merged_code_is_type_safe(env, rest, FrameResult::Regular(&s.next_frame))
                }
                InstructionTypeSafe::AfterGoto(ag) => {
                    let _exception_stack_frame1 = instruction_satisfies_handlers(env, i.offset, &ag.exception_frame)?;
                    merged_code_is_type_safe(env, rest, FrameResult::AfterGoto)
                }
            }
        }
        MergedCodeInstruction::StackMap(s) => {
            match after_frame {
                FrameResult::Regular(f) => {
                    frame_is_assignable(&env.vf, f, &s.map_frame)?;
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
                stack_map: s.map_frame.stack_map.clone(),
                flag_this_uninit: s.map_frame.flag_this_uninit,
            },
        }
    }) {
        None => { Result::Err(unknown_error_verifying!()) }
        Some(f) => Result::Ok(f),
    }
}

fn target_is_type_safe(env: &Environment, stack_frame: &Frame, target: usize) -> Result<(), TypeSafetyError> {
    let frame = offset_stack_frame(env, target)?;
//    dbg!(&env.merged_code);
//    let classfile = get_class(&env.vf, &env.method.class);
//    dbg!(classfile.methods[env.method.method_index as usize].method_name(&classfile));
//    dbg!(&frame);
//    dbg!(target);
//    dbg!(env.merged_code);
//    dbg!(&stack_frame);
//    dbg!(&frame);
    frame_is_assignable(&env.vf, stack_frame, &frame)?;
    Result::Ok(())
}

fn instruction_satisfies_handlers(env: &Environment, offset: usize, exception_stack_frame: &Frame) -> Result<(), TypeSafetyError> {
    let handlers = &env.handlers;
    let applicable_handler = handlers.iter().filter(|h| {
        is_applicable_handler(offset as usize, h)
    });
    let res: Result<Vec<_>, _> = applicable_handler.map(|h| {
        //dbg!(&h);
//        dbg!(offset);
        instruction_satisfies_handler(env, exception_stack_frame, h)
    }).collect();
    res?;
    Result::Ok(())
}

fn is_applicable_handler(offset: usize, handler: &Handler) -> bool {
    offset >= handler.start && offset < handler.end
}

fn class_to_type(vf: &VerifierContext, class: &ClassWithLoader) -> VType {
    let classfile = get_class(vf, class);
    let class_name = class_name(&classfile);
    VType::Class(ClassWithLoader { class_name, loader: class.loader.clone() })
}

fn instruction_satisfies_handler(env: &Environment, exc_stack_frame: &Frame, handler: &Handler) -> Result<(), TypeSafetyError> {
    let target = handler.target;
    let _class_loader = &env.class_loader;
    let exception_class = handler_exception_class(&env.vf, handler, env.class_loader.clone());
    let locals = &exc_stack_frame.locals;
    let flags = exc_stack_frame.flag_this_uninit;
    let locals_copy = locals.iter().map(|x| { x.clone() }).collect();
    let stack_map = OperandStack::new_prolog_display_order(&vec![class_to_type(&env.vf, &exception_class)]);
    let true_exc_stack_frame = Frame { locals: locals_copy, stack_map: stack_map.clone(), flag_this_uninit: flags };
    if operand_stack_has_legal_length(env, &stack_map.clone()) {
        target_is_type_safe(env, &true_exc_stack_frame, target)
    } else {
        Result::Err(TypeSafetyError::NotSafe("operand stack does not have legal length".to_string()))
    }
}

pub fn nth0(index: usize, locals: &Vec<VType>) -> VType {
    match locals.get(index) {
        None => unimplemented!(),
        Some(res) => res.clone(),
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
                let exception_class = handler_exception_class(&env.vf, &h, env.class_loader.clone());
                //todo how does bootstrap loader from throwable make its way into this
                //todo why do I take the class name when I already know it
                let class_name = class_name(&get_class(&env.vf, &exception_class));
                is_assignable(&env.vf, &VType::Class(ClassWithLoader { class_name, loader: env.class_loader.clone() }),
                              &VType::Class(ClassWithLoader { class_name: ClassName::throwable(), loader: env.vf.bootstrap_loader.clone() }))
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


pub fn instructions_include_end(instructs: &Vec<MergedCodeInstruction>, end: usize) -> bool {
    instructs.iter().any(|x: &MergedCodeInstruction| {
        match x {
            MergedCodeInstruction::Instruction(i) => {
                i.offset == end
            }
            MergedCodeInstruction::StackMap(_) => false,
        }
    })
}

pub fn instruction_is_type_safe_dup(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let locals = &stack_frame.locals;
    let input_operand_stack = &stack_frame.stack_map;
    let flags = stack_frame.flag_this_uninit;
    let type_ = pop_category1(&env.vf, &mut input_operand_stack.clone())?;
    let output_operand_stack = can_safely_push(env, input_operand_stack, &type_)?;
    let next_frame = Frame {
        locals: locals.clone(),
        stack_map: output_operand_stack,
        flag_this_uninit: flags,
    };
    standard_exception_frame(stack_frame, next_frame)
}

pub fn can_safely_push(env: &Environment, input_operand_stack: &OperandStack, type_: &VType) -> Result<OperandStack, TypeSafetyError> {
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
    } /*else {
        unimplemented!()
    }*/
    Result::Err(unknown_error_verifying!())
}


pub fn pop_category2(vf: &VerifierContext, input: &mut OperandStack) -> Result<VType, TypeSafetyError> {
    let top = input.operand_pop();
    assert_eq!(top, VType::TopType);
    if size_of(vf, &input.peek()) == 2 {
        let type_ = input.operand_pop();
        return Result::Ok(type_);
    } /*else {
        unimplemented!()
    }*/
    Result::Err(unknown_error_verifying!())
}

pub fn instruction_is_type_safe_dup_x1(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let locals = &stack_frame.locals;
    let input_operand_stack = &stack_frame.stack_map;
    let flags = stack_frame.flag_this_uninit;
    let mut stack_1 = input_operand_stack.clone();
    let type_1 = pop_category1(&env.vf, &mut stack_1)?;
    let mut rest = stack_1.clone();
    let type_2 = pop_category1(&env.vf, &mut rest)?;
    let output_stack = can_safely_push_list(env, &rest, vec![type_1.clone(), type_2, type_1])?;
    let next_frame = Frame {
        locals: locals.clone(),
        stack_map: output_stack,
        flag_this_uninit: flags,
    };
    standard_exception_frame(stack_frame, next_frame)
}

pub fn can_safely_push_list(env: &Environment, input_stack: &OperandStack, types: Vec<VType>) -> Result<OperandStack, TypeSafetyError> {
    let output_stack = can_push_list(&env.vf, input_stack, types.as_slice())?;
    if !operand_stack_has_legal_length(env, &output_stack) {
        return Result::Err(unknown_error_verifying!());
    }
    Result::Ok(output_stack)
}

pub fn can_push_list(vf: &VerifierContext, input_stack: &OperandStack, types: &[VType]) -> Result<OperandStack, TypeSafetyError> {
    if types.is_empty() {
        return Result::Ok(input_stack.clone());
    }
    let type_ = &types[0];
    let rest = &types[1..];
    let interim_stack = push_operand_stack(vf, input_stack, type_);
    can_push_list(vf, &interim_stack, rest)
}

pub fn instruction_is_type_safe_dup2(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
//    StackFrame = frame(Locals, InputOperandStack, Flags),
//    dup2FormIsTypeSafe(Environment,InputOperandStack, OutputOperandStack),
//    NextStackFrame = frame(Locals, OutputOperandStack, Flags),
//    exceptionStackFrame(StackFrame, ExceptionStackFrame).
    let locals = &stack_frame.locals;
    let input_stack = &stack_frame.stack_map;
    let flags = stack_frame.flag_this_uninit;
    let output_stack = dup2_form_is_type_safe(env, input_stack)?;
    let next_frame = Frame {
        locals: locals.clone(),
        stack_map: output_stack,
        flag_this_uninit: flags,
    };
    standard_exception_frame(stack_frame, next_frame)
}


pub fn instruction_is_type_safe_dup_x2(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let locals = &stack_frame.locals;
    let input_stack = &stack_frame.stack_map;
    let flags = stack_frame.flag_this_uninit;
    let output_stack = dup_x2_form_is_type_safe(env, input_stack)?;
    let next_frame = Frame {
        locals: locals.clone(),
        stack_map: output_stack,
        flag_this_uninit: flags,
    };
    standard_exception_frame(stack_frame, next_frame)
}

fn dup_x2_form_is_type_safe(env: &Environment, input_stack: &OperandStack) -> Result<OperandStack, TypeSafetyError> {
    match dup_x2_form1_is_type_safe(env, input_stack) {
        Ok(o) => Result::Ok(o),
        Err(_) => match dup_x2_form2_is_type_safe(env, input_stack) {
            Ok(o) => Result::Ok(o),
            Err(e) => Result::Err(e),
        },
    }
}

fn dup2_form_is_type_safe(env: &Environment, input_stack: &OperandStack) -> Result<OperandStack, TypeSafetyError> {
    match dup2_form1_is_type_safe(env, input_stack) {
        Ok(o) => Result::Ok(o),
        Err(_) => match dup2_form2_is_type_safe(env, input_stack) {
            Ok(o) => Result::Ok(o),
            Err(e) => Result::Err(e),
        },
    }
}



fn dup2_form1_is_type_safe(env: &Environment, input_stack: &OperandStack) -> Result<OperandStack, TypeSafetyError> {
//    popCategory1(InputOperandStack, Type1, TempStack),
//    popCategory1(TempStack, Type2, _),
//    canSafelyPushList(Environment, InputOperandStack, [Type2, Type1], OutputOperandStack).
    let mut temp_stack = input_stack.clone();
    let type1 = pop_category1(&env.vf, &mut temp_stack)?;
    let mut stack2 = temp_stack.clone();
    let type2 = pop_category1(&env.vf, &mut stack2)?;
    can_safely_push_list(env, input_stack, vec![type2,type1])
}

fn dup2_form2_is_type_safe(env: &Environment, input_stack: &OperandStack) -> Result<OperandStack, TypeSafetyError> {
    //popCategory2(InputOperandStack, Type, _),
    //    canSafelyPush(Environment, InputOperandStack, Type, OutputOperandStack).
    let mut stack1 = input_stack.clone();
//    dbg!(&input_stack);
    let type_ = pop_category2(&env.vf, &mut stack1)?;
    can_safely_push_list(env, input_stack, vec![type_.clone()])
}



fn dup_x2_form1_is_type_safe(env: &Environment, input_stack: &OperandStack) -> Result<OperandStack, TypeSafetyError> {
    let mut stack1 = input_stack.clone();
    let type1 = pop_category1(&env.vf, &mut stack1)?;
    let mut stack2 = stack1.clone();
    let type2 = pop_category1(&env.vf, &mut stack2)?;
    let mut rest = stack2.clone();
    let type3 = pop_category1(&env.vf, &mut rest)?;
    can_safely_push_list(env, &rest, vec![type1.clone(), type3, type2, type1])
}

fn dup_x2_form2_is_type_safe(env: &Environment, input_stack: &OperandStack) -> Result<OperandStack, TypeSafetyError> {
    let mut stack1 = input_stack.clone();
    let type1 = pop_category1(&env.vf, &mut stack1)?;
    let mut rest = stack1.clone();
    let type2 = pop_category1(&env.vf, &mut rest)?;
    can_safely_push_list(env, input_stack, vec![type1.clone(), type2, type1])
}


//#[allow(unused)]
//pub fn instruction_is_type_safe_dup2(env: &Environment, offset: usize, stack_frame: &Frame)  -> Result<InstructionTypeSafe, TypeSafetyError> {
//    unimplemented!()
//}
//
//#[allow(unused)]
//pub fn instruction_is_type_safe_dup2_x1(env: &Environment, offset: usize, stack_frame: &Frame)  -> Result<InstructionTypeSafe, TypeSafetyError> {
//    unimplemented!()
//}
//
//#[allow(unused)]
//pub fn instruction_is_type_safe_dup2_x2(env: &Environment, offset: usize, stack_frame: &Frame)  -> Result<InstructionTypeSafe, TypeSafetyError> {
//    unimplemented!()
//}
//
//

pub fn instruction_is_type_safe_i2d(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env, stack_frame, vec![VType::IntType], VType::DoubleType)
}

pub fn instruction_is_type_safe_i2f(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env, stack_frame, vec![VType::IntType], VType::FloatType)
}

pub fn instruction_is_type_safe_i2l(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env, stack_frame, vec![VType::IntType], VType::LongType)
}


pub fn type_transition(env: &Environment, stack_frame: &Frame, expected_types: Vec<VType>, res_type: VType) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let next_frame = valid_type_transition(env, expected_types, &res_type, stack_frame)?;
    standard_exception_frame(stack_frame, next_frame)
}

pub fn instruction_is_type_safe_iadd(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env, stack_frame, vec![VType::IntType, VType::IntType], VType::IntType)
}


pub fn instruction_is_type_safe_iinc(index: usize, _env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let locals = &stack_frame.locals;
    let should_be_int = nth0(index, locals);
    match should_be_int {
        VType::IntType => {
            Result::Ok(InstructionTypeSafe::Safe(ResultFrames {
                next_frame: Frame {
                    locals: stack_frame.locals.clone(),
                    stack_map: stack_frame.stack_map.clone(),
                    flag_this_uninit: stack_frame.flag_this_uninit,
                },
                exception_frame: exception_stack_frame(stack_frame),
            }))
        }
        _ => {
            Result::Err(unknown_error_verifying!())
        }
    }
}

pub fn instruction_is_type_safe_ineg(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env, stack_frame, vec![VType::IntType], VType::IntType)
}




pub fn instruction_is_type_safe_l2d(env: &Environment, stack_frame: &Frame)  -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env, stack_frame, vec![VType::LongType], VType::DoubleType)
}

pub fn instruction_is_type_safe_l2f(env: &Environment, stack_frame: &Frame)  -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env, stack_frame, vec![VType::LongType], VType::FloatType)
}


pub fn instruction_is_type_safe_l2i(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env, stack_frame, vec![VType::LongType], VType::IntType)
}

pub fn instruction_is_type_safe_ladd(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let next_frame = valid_type_transition(env, vec![VType::LongType, VType::LongType], &VType::LongType, stack_frame)?;
    standard_exception_frame(stack_frame, next_frame)
}

fn instruction_is_type_safe_lcmp(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    //todo dup with other arithmetic
    let next_frame = valid_type_transition(env, vec![VType::LongType, VType::LongType], &VType::IntType, stack_frame)?;
    standard_exception_frame(stack_frame, next_frame)
}

pub fn loadable_constant(vf: &VerifierContext, c: &ConstantKind) -> VType {
    match c {
        ConstantKind::Integer(_) => VType::IntType,
        ConstantKind::Float(_) => VType::FloatType,
        ConstantKind::Long(_) => VType::LongType,
        ConstantKind::Double(_) => VType::DoubleType,
        ConstantKind::Class(_c) => {
            let class_name = ClassName::class();
            VType::Class(ClassWithLoader { class_name, loader: vf.bootstrap_loader.clone() })
        }
        ConstantKind::String(_) => {
            let class_name = ClassName::string();
            VType::Class(ClassWithLoader { class_name, loader: vf.bootstrap_loader.clone() })
        }
        ConstantKind::MethodHandle(_) => unimplemented!(),
        ConstantKind::MethodType(_) => unimplemented!(),
        ConstantKind::Dynamic(_) => unimplemented!(),
        ConstantKind::InvokeDynamic(_) => unimplemented!(),
        _ => panic!()
    }
}

pub fn instruction_is_type_safe_ldc(cp: u8, env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let const_ = &get_class(&env.vf, env.method.class).constant_pool[cp as usize].kind;
    let type_: VType = loadable_constant(&env.vf, const_);
    match type_ {
        VType::DoubleType => { return Result::Err(unknown_error_verifying!()); }
        VType::LongType => { return Result::Err(unknown_error_verifying!()); }
        _ => {}
    };
    let next_frame = valid_type_transition(env, vec![], &type_, stack_frame)?;
    standard_exception_frame(stack_frame, next_frame)
}

pub fn instruction_is_type_safe_ldc_w(cp: CPIndex, env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let const_ = &get_class(&env.vf, env.method.class).constant_pool[cp as usize].kind;
    let type_ = match const_ {
        ConstantKind::Integer(_) => VType::IntType,
        ConstantKind::Float(_) => VType::FloatType,
        ConstantKind::Class(_) => VType::Class(ClassWithLoader { class_name: ClassName::class(), loader: env.vf.bootstrap_loader.clone() }),
        ConstantKind::String(_) => VType::Class(ClassWithLoader { class_name: ClassName::string(), loader: env.vf.bootstrap_loader.clone() }),
        ConstantKind::MethodType(_) => VType::Class(ClassWithLoader { class_name: ClassName::new("java/lang/invoke/MethodType"), loader: env.vf.bootstrap_loader.clone() }),
        _ => panic!()
    };
    type_transition(env, stack_frame, vec![], type_)
}

pub fn instruction_is_type_safe_ldc2_w(cp: CPIndex, env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let const_ = &get_class(&env.vf, env.method.class).constant_pool[cp as usize].kind;
    let type_: VType = loadable_constant(&env.vf, const_);//todo dup
    match type_ {
        VType::DoubleType => {}
        VType::LongType => {}
        _ => { return Result::Err(unknown_error_verifying!()); }
    };
    let next_frame = valid_type_transition(env, vec![], &type_, stack_frame)?;
    standard_exception_frame(stack_frame, next_frame)
}

pub fn instruction_is_type_safe_lneg(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env, stack_frame, vec![VType::LongType], VType::LongType)
}

pub fn instruction_is_type_safe_lshl(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env, stack_frame, vec![VType::IntType, VType::LongType], VType::LongType)
}


//
//#[allow(unused)]
//pub fn instruction_is_type_safe_nop(env: &Environment, offset: usize, stack_frame: &Frame)  -> Result<InstructionTypeSafe, TypeSafetyError> {
//    unimplemented!()
//}
//

pub fn instruction_is_type_safe_pop(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let locals = &stack_frame.locals;
    let flags = stack_frame.flag_this_uninit;
    let mut rest = stack_frame.stack_map.clone();
    let _type_ = pop_category1(&env.vf, &mut rest)?;
    let next_frame = Frame {
        locals: locals.clone(),
        stack_map: rest,
        flag_this_uninit: flags,
    };
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames { next_frame, exception_frame }))
}

//#[allow(unused)]
//pub fn instruction_is_type_safe_pop2(env: &Environment, offset: usize, stack_frame: &Frame)  -> Result<InstructionTypeSafe, TypeSafetyError> {
//    unimplemented!()
//}
//

pub fn instruction_is_type_safe_sipush(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env, stack_frame, vec![], VType::IntType)
}

//#[allow(unused)]
//pub fn instruction_is_type_safe_swap(env: &Environment, offset: usize, stack_frame: &Frame)  -> Result<InstructionTypeSafe, TypeSafetyError> {
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
    Frame { locals: f.locals.iter().map(|x| x.clone()).collect(), stack_map: OperandStack::empty(), flag_this_uninit: f.flag_this_uninit }
}

