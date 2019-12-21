use crate::verifier::codecorrectness::{Environment, MergedCodeInstruction, frame_is_assignable, operand_stack_has_legal_length, valid_type_transition, can_pop, handler_exception_class, Handler, init_handler_is_legal};
use rust_jvm_common::classfile::{InstructionInfo, ConstantKind};
use crate::verifier::{Frame, passes_protected_check, PrologClass};
use crate::verifier::instructions::big_match::instruction_is_type_safe;
use crate::verifier::codecorrectness::MergedCodeInstruction::{StackMap, Instruction};
use crate::verifier::codecorrectness::stackmapframes::copy_recurse;
use rust_jvm_common::unified_types::UnifiedType;
use rust_jvm_common::classnames::{ClassName, NameReference, class_name};
use std::sync::Arc;
use crate::instruction_outputer::{extract_class_from_constant_pool, name_and_type_extractor};
use rust_jvm_common::utils::extract_string_from_utf8;
use crate::types::{parse_method_descriptor, MethodDescriptor};
use crate::verifier::filecorrectness::is_assignable;
use rust_jvm_common::unified_types::ClassType;
use rust_jvm_common::loading::BOOTSTRAP_LOADER;
use crate::verifier::TypeSafetyError;

pub mod loads;

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
                locals: s.map_frame.locals.iter().map(|x| copy_recurse(x)).collect(),
                stack_map: s.map_frame.stack_map.iter().map(|x| copy_recurse(x)).collect(),
                flag_this_uninit: s.map_frame.flag_this_uninit,
            },
        }
    }) {
        None => { Result::Err(TypeSafetyError::NotSafe("todo message".to_string())) }//todo msg
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

fn class_to_type(class: &PrologClass) -> UnifiedType {
    let class_name = ClassName::Ref(NameReference {
        index: class.class.this_class,
        class_file: Arc::downgrade(&class.class),
    });
    UnifiedType::Class(ClassType { class_name, loader: class.loader.clone() })
}

fn instruction_satisfies_handler(env: &Environment, exc_stack_frame: &Frame, handler: &Handler) -> Result<(), TypeSafetyError> {
    let target = handler.target;
    let _class_loader = &env.class_loader;
    let exception_class = handler_exception_class(handler);
    let locals = &exc_stack_frame.locals;
    let flags = exc_stack_frame.flag_this_uninit;
    let locals_copy = locals.iter().map(|x| { copy_recurse(x) }).collect();
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
        Some(res) => copy_recurse(res),
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
            let target_stack_frame = offset_stack_frame(env, h.target)?;
            if instructions_include_end(env.merged_code.unwrap(), h.end) {
                let exception_class = handler_exception_class(&h);
                //todo how does bootstrap loader from throwable make its way into this
                let class_name = class_name(&exception_class.class);
                let assignable = is_assignable(&UnifiedType::Class(ClassType { class_name, loader: env.class_loader.clone() }),
                                               &UnifiedType::Class(ClassType { class_name: ClassName::Str("java/lang/Throwable".to_string()), loader: BOOTSTRAP_LOADER.clone() }));
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
//fn instruction_is_type_safe_aastore(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_aconst_null(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//


//
//#[allow(unused)]
//fn instruction_is_type_safe_anewarray(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_areturn(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_arraylength(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}

//#[allow(unused)]
//fn instruction_is_type_safe_astore(index: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_athrow(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_baload(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_bastore(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_caload(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_castore(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_checkcast(index: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_d2f(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_d2i(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_d2l(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_dadd(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_daload(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_dastore(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_dcmpg(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_dconst_0(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_dload(index: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_dneg(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_dreturn(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_dstore(index: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
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
//#[allow(unused)]
//fn instruction_is_type_safe_f2d(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_f2i(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_f2l(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_fadd(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_faload(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_fastore(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_fcmpg(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_fconst_0(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_fload(index: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_fneg(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_freturn(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_fstore(index: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_getfield(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_getstatic(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
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
//#[allow(unused)]
//fn instruction_is_type_safe_iaload(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_iastore(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//

pub fn instruction_is_type_safe_iconst_m1(env: &Environment, offset: usize, stack_frame: &Frame) -> Result<InstructionIsTypeSafeResult,TypeSafetyError> {
    let next_frame = valid_type_transition(env,vec![],&UnifiedType::IntType,stack_frame)?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionIsTypeSafeResult::Safe(ResultFrames {exception_frame, next_frame}))
}
//
//
//#[allow(unused)]
//fn instruction_is_type_safe_if_icmpeq(target: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_ifeq(target: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_ifnonnull(target: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_iinc(index: usize, value: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_iload(index: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_ineg(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_instanceof(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_invokedynamic(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_invokeinterface(cp: usize, count: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_invokespecial(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
pub fn instruction_is_type_safe_invokestatic(cp: usize, env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionIsTypeSafeResult, TypeSafetyError> {
    let (_class_name, method_name, parsed_descriptor) = get_method_descriptor(cp, env);
    if method_name.contains("arrayOf") || method_name.contains("[") || method_name == "<init>" || method_name == "<clinit>" {
        unimplemented!();
    }
    let operand_arg_list = parsed_descriptor.parameter_types;
    let stack_arg_list: Vec<UnifiedType> = operand_arg_list.iter()
        .rev()
        .map(|x| copy_recurse(x))
        .collect();
    let next_frame = match valid_type_transition(env, stack_arg_list, &parsed_descriptor.return_type, stack_frame) {
        Ok(nf) => nf,
        Err(_) => unimplemented!(),
    };
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionIsTypeSafeResult::Safe(ResultFrames { exception_frame, next_frame }))
}


pub fn instruction_is_type_safe_invokevirtual(cp: usize, env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionIsTypeSafeResult, TypeSafetyError> {
    let (class_name, method_name, parsed_descriptor) = get_method_descriptor(cp, env);
    if method_name.contains("arrayOf") || method_name.contains("[") || method_name == "<init>" || method_name == "<clinit>" {
        unimplemented!();
    }
    let operand_arg_list = &parsed_descriptor.parameter_types;
    let arg_list: Vec<UnifiedType> = operand_arg_list.iter()
        .rev()
        .map(|x| copy_recurse(x))
        .collect();
    let current_loader = &env.class_loader;
//todo deal with loaders in class names/types
    let mut stack_arg_list: Vec<UnifiedType> = arg_list.iter().map(|x| copy_recurse(x)).collect();
    let class_type = ClassType { class_name: ClassName::Str(class_name.clone()), loader: current_loader.clone() };//todo better name
    stack_arg_list.push(UnifiedType::Class(class_type));
    stack_arg_list.reverse();
    let nf = valid_type_transition(env, stack_arg_list, &parsed_descriptor.return_type, stack_frame)?;
    let popped_frame = can_pop(stack_frame, arg_list)?;
    passes_protected_check(env, class_name.clone(), method_name, &parsed_descriptor, &popped_frame)?;
    let exception_stack_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionIsTypeSafeResult::Safe(ResultFrames { exception_frame: exception_stack_frame, next_frame: nf }))
}

fn get_method_descriptor(cp: usize, env: &Environment) -> (String, String, MethodDescriptor) {
    let classfile = &env.method.prolog_class.class;
    let c = &classfile.constant_pool[cp].kind;
    let (class_name, method_name, parsed_descriptor) = match c {
        ConstantKind::Methodref(m) => {
            let c = extract_class_from_constant_pool(m.class_index, &classfile);
            let class_name = extract_string_from_utf8(&classfile.constant_pool[c.name_index as usize]);
            let (method_name, descriptor) = name_and_type_extractor(m.name_and_type_index, classfile);
            let parsed_descriptor = match parse_method_descriptor(&env.class_loader, descriptor.as_str()) {
                None => { unimplemented!() }
                Some(pd) => { pd }
            };
            (class_name, method_name, parsed_descriptor)
        }
        _ => unimplemented!()
    };
    (class_name, method_name, parsed_descriptor)
}

//
//#[allow(unused)]
//fn instruction_is_type_safe_ireturn(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_istore(index: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
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
//#[allow(unused)]
//fn instruction_is_type_safe_laload(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_lastore(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
fn instruction_is_type_safe_lcmp(env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionIsTypeSafeResult,TypeSafetyError> {
    let next_frame = valid_type_transition(env, vec![UnifiedType::LongType, UnifiedType::LongType], &UnifiedType::IntType, stack_frame)?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionIsTypeSafeResult::Safe(ResultFrames { next_frame, exception_frame }))
}

pub fn instruction_is_type_safe_lconst_0(env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionIsTypeSafeResult,TypeSafetyError> {
    let next_frame = valid_type_transition(env, vec![], &UnifiedType::LongType, stack_frame)?;
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
//fn instruction_is_type_safe_lookupswitch(targets: Vec<usize>, keys: Vec<usize>, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_lreturn(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_lshl(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_lstore(index: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_monitorenter(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_multianewarray(cp: usize, dim: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
////todo start using CPIndex instead of usize
//
//#[allow(unused)]
//fn instruction_is_type_safe_new(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_newarray(type_code: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
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
//fn instruction_is_type_safe_putfield(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_putstatic(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//
//#[allow(unused)]
//fn instruction_is_type_safe_saload(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_sastore(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
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
//#[allow(unused)]
//fn instruction_is_type_safe_tableswitch(targets: Vec<usize>, keys: Vec<usize>, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}

#[allow(unused)]
fn different_package_name(class1: &PrologClass, class2: &PrologClass) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn same_package_name(class1: &PrologClass, class2: &PrologClass) -> bool {
    unimplemented!()
}


#[allow(unused)]
fn store_is_type_safe(env: &Environment, index: usize, type_: &UnifiedType, frame: &Frame, next_frame: &Frame) {
    unimplemented!()
}

pub mod big_match;
pub mod branches;


pub fn exception_stack_frame(f: &Frame) -> Frame {
    Frame { locals: f.locals.iter().map(|x| copy_recurse(x)).collect(), stack_map: vec![], flag_this_uninit: f.flag_this_uninit }
}

