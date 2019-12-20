use crate::verification::verifier::codecorrectness::{Environment, MergedCodeInstruction, frame_is_assignable, operand_stack_has_legal_length, valid_type_transition, can_pop, handler_exception_class, Handler, init_handler_is_legal};
use rust_jvm_common::classfile::{InstructionInfo, ConstantKind};
use crate::verification::verifier::{TypeSafetyResult, Frame, merge_type_safety_results, passes_protected_check, PrologClass, and};
use crate::verification::verifier::instructions::big_match::instruction_is_type_safe;
use crate::verification::verifier::codecorrectness::MergedCodeInstruction::{StackMap, Instruction};
use crate::verification::verifier::codecorrectness::stackmapframes::copy_recurse;
use rust_jvm_common::unified_types::UnifiedType;
use rust_jvm_common::classnames::{ClassName, NameReference, class_name};
use std::sync::Arc;
use crate::verification::instruction_outputer::{extract_class_from_constant_pool, name_and_type_extractor};
use rust_jvm_common::utils::extract_string_from_utf8;
use crate::verification::types::{parse_method_descriptor, MethodDescriptor};
use crate::verification::verifier::filecorrectness::is_assignable;

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
    NotSafe,
}

pub enum FrameResult<'l> {
    Regular(&'l Frame),
    AfterGoto,
}

//todo how to handle other values here
pub fn merged_code_is_type_safe<'l>(env: &Environment, merged_code: &[MergedCodeInstruction], after_frame: FrameResult<'l>) -> TypeSafetyResult {
    let first = &merged_code[0];//todo infinite recursion
    let rest = &merged_code[1..merged_code.len()];
    match first {
        MergedCodeInstruction::Instruction(i) => {
            let f = match after_frame {
                FrameResult::Regular(f) => f,
                FrameResult::AfterGoto => {
                    match i.instruction {
                        InstructionInfo::EndOfCode => return TypeSafetyResult::Safe(),
                        _ => return TypeSafetyResult::NotSafe("No stack frame after unconditional branch".to_string())
                    }
                }
            };
            match instruction_is_type_safe(&i.instruction, env, i.offset, f) {
                InstructionIsTypeSafeResult::Safe(s) => {
                    let _exception_stack_frame1 = instruction_satisfies_handlers(env, i.offset, &s.exception_frame);
                    merged_code_is_type_safe(env, rest, FrameResult::Regular(&s.next_frame))
                }
                InstructionIsTypeSafeResult::AfterGoto(ag) => {
                    let _exception_stack_frame1 = instruction_satisfies_handlers(env, i.offset, &ag.exception_frame);
                    merged_code_is_type_safe(env, rest, FrameResult::AfterGoto)
                }
                InstructionIsTypeSafeResult::NotSafe => { return TypeSafetyResult::NotSafe("todo message".to_string()); }//todo
            }
        }
        MergedCodeInstruction::StackMap(s) => {
            match after_frame {
                FrameResult::Regular(f) => {
                    and(frame_is_assignable(f, &s.map_frame),
                        merged_code_is_type_safe(env, rest, FrameResult::Regular(&s.map_frame)))
                }
                FrameResult::AfterGoto => {
                    merged_code_is_type_safe(env, rest, FrameResult::Regular(&s.map_frame))
                }
            }
        }
    }
}

fn offset_stack_frame(env: &Environment, offset: i16) -> Option<Frame> {
    env.merged_code.unwrap().iter().find(|x| {
        match x {
            Instruction(_) => false,
            StackMap(s) => {
                s.offset == offset as usize
            }
        }
    }).map(|x|{
        match x{
            Instruction(_) => panic!(),
            StackMap(s) => Frame {
                locals: s.map_frame.locals.iter().map(|x|copy_recurse(x)).collect(),
                stack_map: s.map_frame.stack_map.iter().map(|x|copy_recurse(x)).collect(),
                flag_this_uninit: s.map_frame.flag_this_uninit
            },
        }
    })
}

fn target_is_type_safe(env: &Environment, stack_frame: &Frame, target: i16) -> TypeSafetyResult {
    let frame = offset_stack_frame(env, target);
    match frame {
        None => { return TypeSafetyResult::NotSafe("No frame fround at target".to_string()); }
        Some(f) => { frame_is_assignable(stack_frame, &f) }
    }
}

fn instruction_satisfies_handlers(env: &Environment, offset: usize, exception_stack_frame: &Frame) -> TypeSafetyResult {
    let handlers = &env.handlers;
    let applicable_handler = handlers.iter().filter(|h| {
        is_applicable_handler(offset as usize, h)
    });
    merge_type_safety_results(applicable_handler.map(|h| {
        instruction_satisfies_handler(env, exception_stack_frame, h)
    }).collect())
}

fn is_applicable_handler(offset: usize, handler: &Handler) -> bool {
    offset <= handler.start && offset < handler.end
}

fn class_to_type(class: &PrologClass) -> UnifiedType {
    UnifiedType::Class(ClassName::Ref(NameReference {
        index: class.class.this_class,
        class_file: Arc::downgrade(&class.class),
    }))
}

fn instruction_satisfies_handler(env: &Environment, exc_stack_frame: &Frame, handler: &Handler) -> TypeSafetyResult {
    let target = handler.target;
    let _class_loader = &env.class_loader;
    let exception_class = handler_exception_class(handler);
    let locals = &exc_stack_frame.locals;
    let flags = exc_stack_frame.flag_this_uninit;
    let locals_copy = locals.iter().map(|x| { copy_recurse(x) }).collect();
    let true_exc_stack_frame = Frame { locals: locals_copy, stack_map: vec![class_to_type(&exception_class)], flag_this_uninit: flags };
    if operand_stack_has_legal_length(env, &vec![class_to_type(&exception_class)]) {
        target_is_type_safe(env, &true_exc_stack_frame, target as i16)
    } else {
        TypeSafetyResult::NotSafe("operand stack does not have legal length".to_string())
    }
}

pub fn nth0(index: usize, locals: &Vec<UnifiedType>) -> UnifiedType {
    match locals.get(index) {
        None => unimplemented!(),
        Some(res) => copy_recurse(res),
    }
}


pub fn handers_are_legal(env: &Environment) -> TypeSafetyResult {
    let handlers = &env.handlers;
    merge_type_safety_results(handlers.iter().map(|h| {
        handler_is_legal(env, h)
    }).collect())
}

pub fn start_is_member_of(start: usize, merged_instructs: &Vec<MergedCodeInstruction>) -> bool {
    merged_instructs.iter().any(|m| match m {
        Instruction(i) => { i.offset == start }
        StackMap(s) => { s.offset == start }
    })
}

pub fn handler_is_legal(env: &Environment, h: &Handler) -> TypeSafetyResult {
    if h.start < h.end {
        if start_is_member_of(h.start, env.merged_code.unwrap()) {
            let target_stack_frame = offset_stack_frame(env, h.target as i16);
            match target_stack_frame {
                None => { TypeSafetyResult::NotSafe("No stack frame present at target".to_string()) }
                Some(_t) => {
                    if instructions_include_end(env.merged_code.unwrap(), h.end) {
                        let exception_class = handler_exception_class(&h);
                        //todo how does bootstrap loader from throwable make its way into this
                        if is_assignable(&UnifiedType::Class(class_name(&exception_class.class)),
                                         &UnifiedType::Class(ClassName::Str("java/lang/Throwable".to_string()))) {
                            return init_handler_is_legal(env, h);
                        } else {
                            TypeSafetyResult::NotSafe("Handler exception class not assignable to Throwable".to_string())
                        }
                    } else {
                        TypeSafetyResult::NotSafe("Instructions do not include handler end".to_string())
                    }
                }
            }
        } else {
            TypeSafetyResult::NotSafe("No instruction found at handler start.".to_string())
        }
    } else {
        TypeSafetyResult::NotSafe("Handler start not less than end".to_string())
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
//#[allow(unused)]
//fn instruction_is_type_safe_goto(target: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
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
//#[allow(unused)]
//fn instruction_is_type_safe_iconst_m1(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
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
pub fn instruction_is_type_safe_invokestatic(cp: usize, env: &Environment, _offset: usize, stack_frame: &Frame) -> InstructionIsTypeSafeResult {
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
    return InstructionIsTypeSafeResult::Safe(ResultFrames { exception_frame, next_frame });
}


pub fn instruction_is_type_safe_invokevirtual(cp: usize, env: &Environment, _offset: usize, stack_frame: &Frame) -> InstructionIsTypeSafeResult {
    let (class_name, method_name, parsed_descriptor) = get_method_descriptor(cp, env);
    if method_name.contains("arrayOf") || method_name.contains("[") || method_name == "<init>" || method_name == "<clinit>" {
        unimplemented!();
    }
    let operand_arg_list = &parsed_descriptor.parameter_types;
    let arg_list: Vec<UnifiedType> = operand_arg_list.iter()
        .rev()
        .map(|x| copy_recurse(x))
        .collect();
    let _current_loader = &env.class_loader;
    //todo deal with loaders in class names/types
    let mut stack_arg_list: Vec<UnifiedType> = arg_list.iter().map(|x| copy_recurse(x)).collect();
    stack_arg_list.push(UnifiedType::Class(ClassName::Str(class_name.clone())));
    stack_arg_list.reverse();
    match valid_type_transition(env, stack_arg_list, &parsed_descriptor.return_type, stack_frame) {
        Ok(nf) => {
            let popped_frame = can_pop(stack_frame, arg_list).unwrap_or_else(|| unimplemented!());
            passes_protected_check(env, class_name.clone(), method_name, &parsed_descriptor, &popped_frame);
            let exception_stack_frame = exception_stack_frame(stack_frame);
            InstructionIsTypeSafeResult::Safe(ResultFrames { exception_frame: exception_stack_frame, next_frame: nf })
        }
        Err(_e) => InstructionIsTypeSafeResult::NotSafe,
    }
}

fn get_method_descriptor(cp: usize, env: &Environment) -> (String, String, MethodDescriptor) {
    let classfile = &env.method.prolog_class.class;
    let c = &classfile.constant_pool[cp].kind;
    let (class_name, method_name, parsed_descriptor) = match c {
        ConstantKind::Methodref(m) => {
            let c = extract_class_from_constant_pool(m.class_index, &classfile);
            let class_name = extract_string_from_utf8(&classfile.constant_pool[c.name_index as usize]);
            let (method_name, descriptor) = name_and_type_extractor(m.name_and_type_index, classfile);
            let parsed_descriptor = match parse_method_descriptor(descriptor.as_str()) {
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
fn instruction_is_type_safe_lcmp(env: &Environment, _offset: usize, stack_frame: &Frame) -> InstructionIsTypeSafeResult {
    let next_frame = match valid_type_transition(env, vec![UnifiedType::LongType, UnifiedType::LongType], &UnifiedType::IntType, stack_frame) {
        Ok(nf) => nf,
        Err(_) => return InstructionIsTypeSafeResult::NotSafe,
    };
    let exception_frame = exception_stack_frame(stack_frame);
    InstructionIsTypeSafeResult::Safe(ResultFrames { next_frame, exception_frame })
}

pub fn instruction_is_type_safe_lconst_0(env: &Environment, _offset: usize, stack_frame: &Frame) -> InstructionIsTypeSafeResult {
    let next_frame = match valid_type_transition(env, vec![], &UnifiedType::LongType, stack_frame) {
        Ok(nf) => nf,
        Err(_) => return InstructionIsTypeSafeResult::NotSafe,
    };
    let exception_frame = exception_stack_frame(stack_frame);
    return InstructionIsTypeSafeResult::Safe(ResultFrames { next_frame, exception_frame });
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

