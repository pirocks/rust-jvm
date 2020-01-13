use crate::verifier::codecorrectness::{Environment, MergedCodeInstruction, frame_is_assignable, operand_stack_has_legal_length, valid_type_transition, handler_exception_class, Handler, size_of, push_operand_stack};
use rust_jvm_common::classfile::{InstructionInfo, ConstantKind};
use crate::verifier::{Frame, get_class};
use crate::verifier::instructions::big_match::instruction_is_type_safe;
use crate::verifier::codecorrectness::MergedCodeInstruction::{StackMap, Instruction};
use rust_jvm_common::unified_types::ClassWithLoader;
use rust_jvm_common::classnames::{ClassName, class_name};
use crate::verifier::filecorrectness::is_assignable;
use crate::verifier::TypeSafetyError;
use rust_jvm_common::classfile::CPIndex;
use crate::VerifierContext;
use crate::OperandStack;
use rust_jvm_common::utils::method_name;
use rust_jvm_common::unified_types::VerificationType;

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
    dbg!(&env.merged_code);
    let classfile = get_class(&env.vf, &env.method.class);
    dbg!(method_name(&classfile, &classfile.methods[env.method.method_index as usize]));
    dbg!(&frame);
    dbg!(target);
//    dbg!(env.merged_code);
    frame_is_assignable(&env.vf, stack_frame, &frame)?;
    Result::Ok(())
}

fn instruction_satisfies_handlers(env: &Environment, offset: usize, exception_stack_frame: &Frame) -> Result<(), TypeSafetyError> {
    let handlers = &env.handlers;
    let applicable_handler = handlers.iter().filter(|h| {
        is_applicable_handler(offset as usize, h)
    });
    let res: Result<Vec<_>, _> = applicable_handler.map(|h| {
        dbg!(&h);
        dbg!(offset);
        instruction_satisfies_handler(env, exception_stack_frame, h)
    }).collect();
    res?;
    Result::Ok(())
}

fn is_applicable_handler(offset: usize, handler: &Handler) -> bool {
    offset >= handler.start && offset < handler.end
}

fn class_to_type(vf: &VerifierContext, class: &ClassWithLoader) -> VerificationType {
    let classfile = get_class(vf, class);
    let class_name = class_name(&classfile);
    VerificationType::Class(ClassWithLoader { class_name, loader: class.loader.clone() })
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

pub fn nth0(index: usize, locals: &Vec<VerificationType>) -> VerificationType {
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
                let exception_class = handler_exception_class(&env.vf, &h, env.class_loader.clone());
                //todo how does bootstrap loader from throwable make its way into this
                let class_name = class_name(&get_class(&env.vf, &exception_class));
                let assignable = is_assignable(&env.vf, &VerificationType::Class(ClassWithLoader { class_name, loader: env.class_loader.clone() }),
                                               &VerificationType::Class(ClassWithLoader { class_name: ClassName::Str("java/lang/Throwable".to_string()), loader: env.vf.bootstrap_loader.clone() }));
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

pub fn instruction_is_type_safe_dup(env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
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
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames { next_frame, exception_frame }))
}

pub fn can_safely_push(env: &Environment, input_operand_stack: &OperandStack, type_: &VerificationType) -> Result<OperandStack, TypeSafetyError> {
    let output_operand_stack = push_operand_stack(&env.vf, input_operand_stack, type_);
    if operand_stack_has_legal_length(env, &output_operand_stack) {
        Result::Ok(output_operand_stack)
    } else {
        Result::Err(unknown_error_verifying!())
    }
}

pub fn pop_category1(vf: &VerifierContext, input: &mut OperandStack) -> Result<VerificationType, TypeSafetyError> {
    if size_of(vf, &input.peek()) == 1 {
        let type_ = input.operand_pop();
        return Result::Ok(type_);
    } else {
        unimplemented!()
    }
}

pub fn instruction_is_type_safe_dup_x1(env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
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
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames { next_frame, exception_frame }))
}

pub fn can_safely_push_list(env: &Environment, input_stack: &OperandStack, types: Vec<VerificationType>) -> Result<OperandStack, TypeSafetyError> {
    let output_stack = can_push_list(&env.vf, input_stack, types.as_slice())?;
    if !operand_stack_has_legal_length(env, &output_stack) {
        return Result::Err(unknown_error_verifying!());
    }
    Result::Ok(output_stack)
}

pub fn can_push_list(vf: &VerifierContext, input_stack: &OperandStack, types: &[VerificationType]) -> Result<OperandStack, TypeSafetyError> {
    if types.is_empty() {
        return Result::Ok(input_stack.clone());
    }
    let type_ = &types[0];
    let rest = &types[1..];
    let interim_stack = push_operand_stack(vf, input_stack, type_);
    can_push_list(vf, &interim_stack, rest)
}

pub fn instruction_is_type_safe_dup_x2(env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let locals = &stack_frame.locals;
    let input_stack = &stack_frame.stack_map;
    let flags = stack_frame.flag_this_uninit;
    let output_stack = dup_x2_form_is_type_safe(env, input_stack)?;
    let next_frame = Frame {
        locals: locals.clone(),
        stack_map: output_stack,
        flag_this_uninit: flags,
    };
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames { next_frame, exception_frame }))
}

fn dup_x2_form_is_type_safe(env: &Environment, input_stack: &OperandStack) -> Result<OperandStack, TypeSafetyError> {
    match dup_x2_form1_is_type_safe(env,input_stack) {
        Ok(o) => Result::Ok(o),
        Err(_) => match dup_x2_form2_is_type_safe(env, input_stack){
            Ok(o) => Result::Ok(o),
            Err(e) => Result::Err(e),
        },
    }
}


fn dup_x2_form1_is_type_safe(env: &Environment, input_stack: &OperandStack) -> Result<OperandStack, TypeSafetyError> {
    let mut stack1 = input_stack.clone();
    let type1 = pop_category1(&env.vf,& mut stack1)?;
    let mut stack2 = stack1.clone();
    let type2 = pop_category1(&env.vf, &mut stack2)?;
    let mut rest = stack2.clone();
    let type3 = pop_category1(&env.vf, &mut rest)?;
    can_safely_push_list(env,&rest,vec![type1.clone(),type3,type2,type1])
}

fn dup_x2_form2_is_type_safe(env: &Environment, input_stack: &OperandStack) -> Result<OperandStack, TypeSafetyError> {
    let mut stack1 = input_stack.clone();
    let type1 = pop_category1(&env.vf,&mut stack1)?;
    let mut rest = stack1.clone();
    let type2 = pop_category1(&env.vf,&mut rest)?;
    can_safely_push_list(env,input_stack,vec![type1.clone(),type2,type1])
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

pub fn instruction_is_type_safe_i2d(env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env, stack_frame, vec![VerificationType::IntType], VerificationType::DoubleType)
}

pub fn instruction_is_type_safe_i2f(env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env, stack_frame, vec![VerificationType::IntType], VerificationType::FloatType)
}

pub fn instruction_is_type_safe_i2l(env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env, stack_frame, vec![VerificationType::IntType], VerificationType::LongType)
}


pub fn type_transition(env: &Environment, stack_frame: &Frame, expected_types: Vec<VerificationType>, res_type: VerificationType) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let next_frame = valid_type_transition(env, expected_types, &res_type, stack_frame)?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames { next_frame, exception_frame }))
}

pub fn instruction_is_type_safe_iadd(env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env, stack_frame, vec![VerificationType::IntType, VerificationType::IntType], VerificationType::IntType)
}


pub fn instruction_is_type_safe_iinc(index: usize, _env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let locals = &stack_frame.locals;
    let should_be_int = nth0(index, locals);
    match should_be_int {
        VerificationType::IntType => {
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

pub fn instruction_is_type_safe_ineg(env: &Environment, _offset: usize, stack_frame: &Frame)  -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env,stack_frame,vec![VerificationType::IntType],VerificationType::IntType)
}

//
//
//#[allow(unused)]
//pub fn instruction_is_type_safe_l2d(env: &Environment, offset: usize, stack_frame: &Frame)  -> Result<InstructionTypeSafe, TypeSafetyError> {
//    unimplemented!()
//}
//
//#[allow(unused)]
//pub fn instruction_is_type_safe_l2f(env: &Environment, offset: usize, stack_frame: &Frame)  -> Result<InstructionTypeSafe, TypeSafetyError> {
//    unimplemented!()
//}
//

pub fn instruction_is_type_safe_l2i(env: &Environment, _offset: usize, stack_frame: &Frame)  -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env,stack_frame,vec![VerificationType::LongType],VerificationType::IntType)
}

pub fn instruction_is_type_safe_ladd(env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let next_frame = valid_type_transition(env, vec![VerificationType::LongType, VerificationType::LongType], &VerificationType::LongType, stack_frame)?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames { next_frame, exception_frame }))
}

fn instruction_is_type_safe_lcmp(env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    //todo dup with other arithmetic
    let next_frame = valid_type_transition(env, vec![VerificationType::LongType, VerificationType::LongType], &VerificationType::IntType, stack_frame)?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames { next_frame, exception_frame }))
}

pub fn loadable_constant(vf: &VerifierContext, c: &ConstantKind) -> VerificationType {
    match c {
        ConstantKind::Integer(_) => VerificationType::IntType,
        ConstantKind::Float(_) => VerificationType::FloatType,
        ConstantKind::Long(_) => VerificationType::LongType,
        ConstantKind::Double(_) => VerificationType::DoubleType,
        ConstantKind::Class(_c) => {
            let class_name = ClassName::Str("java/lang/Class".to_string());
            VerificationType::Class(ClassWithLoader { class_name, loader: vf.bootstrap_loader.clone() })
        }
        ConstantKind::String(_) => {
            let class_name = ClassName::Str("java/lang/String".to_string());
            VerificationType::Class(ClassWithLoader { class_name, loader: vf.bootstrap_loader.clone() })
        }
        ConstantKind::MethodHandle(_) => unimplemented!(),
        ConstantKind::MethodType(_) => unimplemented!(),
        ConstantKind::Dynamic(_) => unimplemented!(),
        ConstantKind::InvokeDynamic(_) => unimplemented!(),
        _ => panic!()
    }
}

pub fn instruction_is_type_safe_ldc(cp: u8, env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let const_ = &get_class(&env.vf, env.method.class).constant_pool[cp as usize].kind;
    let type_: VerificationType = loadable_constant(&env.vf, const_);
    match type_ {
        VerificationType::DoubleType => { return Result::Err(unknown_error_verifying!()); }
        VerificationType::LongType => { return Result::Err(unknown_error_verifying!()); }
        _ => {}
    };
    let next_frame = valid_type_transition(env, vec![], &type_, stack_frame)?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames { next_frame, exception_frame }))
}

pub fn instruction_is_type_safe_ldc2_w(cp: CPIndex, env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let const_ = &get_class(&env.vf, env.method.class).constant_pool[cp as usize].kind;
    let type_: VerificationType = loadable_constant(&env.vf, const_);//todo dup
    match type_ {
        VerificationType::DoubleType => {}
        VerificationType::LongType => {}
        _ => { return Result::Err(unknown_error_verifying!()); }
    };
    let next_frame = valid_type_transition(env, vec![], &type_, stack_frame)?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames { next_frame, exception_frame }))
}

//
//#[allow(unused)]
//pub fn instruction_is_type_safe_lneg(env: &Environment, offset: usize, stack_frame: &Frame)  -> Result<InstructionTypeSafe, TypeSafetyError> {
//    unimplemented!()
//}

//
//#[allow(unused)]
//pub fn instruction_is_type_safe_lshl(env: &Environment, offset: usize, stack_frame: &Frame)  -> Result<InstructionTypeSafe, TypeSafetyError> {
//    unimplemented!()
//}
//
//
//
//#[allow(unused)]
//pub fn instruction_is_type_safe_nop(env: &Environment, offset: usize, stack_frame: &Frame)  -> Result<InstructionTypeSafe, TypeSafetyError> {
//    unimplemented!()
//}
//

pub fn instruction_is_type_safe_pop(env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
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

pub fn instruction_is_type_safe_sipush(env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env, stack_frame, vec![], VerificationType::IntType)
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

