use verification::verifier::{Frame, PrologClass};
use verification::unified_type::UnifiedType;
use classfile::code::InstructionInfo;
use verification::verifier::codecorrectness::{Environment, valid_type_transition};
use verification::verifier::filecorrectness::is_assignable;
use verification::verifier::codecorrectness::frame_is_assignable;
use verification::verifier::codecorrectness::Handler;
use verification::classnames::NameReference;
use std::sync::Arc;
use verification::verifier::codecorrectness::operand_stack_has_legal_length;
use verification::verifier::codecorrectness::handler_exception_class;
use verification::verifier::codecorrectness::stackmapframes::copy_recurse;
use verification::classnames::ClassName;
use verification::verifier::codecorrectness::MergedCodeInstruction;
use verification::verifier::TypeSafetyResult;
use verification::verifier::codecorrectness::MergedCodeInstruction::Instruction;
use verification::verifier::codecorrectness::MergedCodeInstruction::StackMap;
use verification::prolog_info_writer::class_name;
use verification::verifier::codecorrectness::init_handler_is_legal;
use verification::verifier::and;
use verification::verifier::merge_type_safety_results;

pub mod loads;

pub struct InstructionIsTypeSafeResult {
    pub(crate) next_frame: Frame,
    pub(crate) exception_frame: Frame,
}


//todo how to handle other values here
pub fn merged_code_is_type_safe(env: &Environment, merged_code: &[MergedCodeInstruction], after_frame: &Frame, after_goto: bool) -> TypeSafetyResult {
    let first = &merged_code[0];//todo inifte recursion
    let rest = &merged_code[1..merged_code.len()];
    match first {
        MergedCodeInstruction::Instruction(i) => {
            let instruction_res = instruction_is_type_safe(&i.instruction, env, i.offset, after_frame).unwrap();//todo unwrap
            let exception_stack_frame1 = instruction_satisfies_handlers(env, i.offset, &instruction_res.exception_frame);
            merged_code_is_type_safe(env, rest, &instruction_res.next_frame, false)
        }
        MergedCodeInstruction::StackMap(s) => {
            if after_goto {
                merged_code_is_type_safe(env, rest, &s.map_frame, false)
            } else {
                and(frame_is_assignable(after_frame, &s.map_frame),
                    merged_code_is_type_safe(env, rest, &s.map_frame, false))
            }
        }
    }
}

#[allow(unused)]
fn offset_stack_frame(env: &Environment, target: usize) -> Option<Frame> {
    unimplemented!()
}

fn target_is_type_safe(env: &Environment, stack_frame: &Frame, target: usize) -> TypeSafetyResult {
    let frame = offset_stack_frame(env, target);
    match frame {
        None => { return TypeSafetyResult::NotSafe("No frame fround at target".to_string()); }
        Some(f) => { frame_is_assignable(stack_frame, &f) }
    }
}

fn instruction_satisfies_handlers(env: &Environment, offset: usize, exception_stack_frame: &Frame) -> TypeSafetyResult {
    let handlers = &env.handlers;
    let mut applicable_handler = handlers.iter().filter(|h| {
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
    UnifiedType::ReferenceType(ClassName::Ref(NameReference {
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
        target_is_type_safe(env, &true_exc_stack_frame, target)
    } else {
        TypeSafetyResult::NotSafe("operand stack does not have legal length".to_string())
    }

}

pub fn nth0(_index: usize, _locals: &Vec<UnifiedType>) -> UnifiedType {
    unimplemented!()
}


pub fn handers_are_legal(env: &Environment) -> TypeSafetyResult {
    let handlers = &env.handlers;
    handlers.iter().map(|h| {
        handler_is_legal(env, h)
    });
    unimplemented!()
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
            let target_stack_frame = offset_stack_frame(env, h.target);
            match target_stack_frame {
                None => { TypeSafetyResult::NotSafe("No stack frame present at target".to_string()) }
                Some(t) => {
                    if instructions_include_end(env.merged_code.unwrap(), h.end) {
                        let exception_class = handler_exception_class(&h);
                        //todo how does bootstrap loader from throwable make its way into this
                        if is_assignable(&UnifiedType::ReferenceType(class_name(&exception_class.class)),
                                         &UnifiedType::ReferenceType(ClassName::Str("java/lang/Throwable".to_string()))) {
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


pub fn instructions_include_end(instructs: &Vec<MergedCodeInstruction>, end: usize) -> bool {
    unimplemented!()
}


#[allow(unused)]
pub(crate) fn instruction_is_type_safe(instruction: &InstructionInfo, env: &Environment, offset: usize, stack_frame: &Frame) -> Option<InstructionIsTypeSafeResult> {
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
//#[allow(unused)]
//fn instruction_is_type_safe_aload(index: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
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
//#[allow(unused)]
//fn instruction_is_type_safe_if_acmpeq(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
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
//#[allow(unused)]
//fn instruction_is_type_safe_invokestatic(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_invokevirtual(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
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
//#[allow(unused)]
//fn instruction_is_type_safe_lcmp(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_lconst_0(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
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
//#[allow(unused)]
//fn instruction_is_type_safe_lload(index: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
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
//#[allow(unused)]
//fn instruction_is_type_safe_return(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
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
fn load_is_type_safe(env: &Environment, index: usize, type_: &UnifiedType, frame: &Frame, next_frame: &Frame) -> bool {
    let locals = &frame.locals;
    let actual_type = nth0(index, locals);
    let type_transition = valid_type_transition(env, vec![], &actual_type, frame);

    is_assignable(&actual_type, type_) &&
        &type_transition == next_frame
}

#[allow(unused)]
fn store_is_type_safe(env: &Environment, index: usize, type_: &UnifiedType, frame: &Frame, next_frame: &Frame) {
    unimplemented!()
}