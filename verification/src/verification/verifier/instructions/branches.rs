use verification::verifier::codecorrectness::{Environment, can_pop};
use verification::verifier::{Frame, TypeSafetyResult};
use verification::unified_type::UnifiedType;
use verification::verifier::instructions::{InstructionIsTypeSafeResult, target_is_type_safe, ResultFrames};
use verification::verifier::instructions::AfterGotoFrames;
use verification::verifier::instructions::exception_stack_frame;

pub fn instruction_is_type_safe_return(env: &Environment, offset: usize, stack_frame: &Frame) -> InstructionIsTypeSafeResult {
    match env.return_type{
        UnifiedType::VoidType => {},
        _ => {return InstructionIsTypeSafeResult::NotSafe;}
    };
    if stack_frame.flag_this_uninit {
        return InstructionIsTypeSafeResult::NotSafe;
    }
    let exception_frame = exception_stack_frame(stack_frame);
    return InstructionIsTypeSafeResult::AfterGoto(AfterGotoFrames{
        exception_frame
    })

}


pub fn instruction_is_type_safe_if_acmpeq(target:i16,env: &Environment, offset: usize, stack_frame: &Frame) -> InstructionIsTypeSafeResult {
    match can_pop(stack_frame,vec![UnifiedType::Reference,UnifiedType::Reference]){
        None => InstructionIsTypeSafeResult::NotSafe,
        Some(next_frame) => {
            match target_is_type_safe(env,&next_frame,target){
                TypeSafetyResult::NotSafe(_) => InstructionIsTypeSafeResult::NotSafe,
                TypeSafetyResult::Safe() => {
                    let exception_frame = exception_stack_frame(stack_frame);
                    InstructionIsTypeSafeResult::Safe(ResultFrames {next_frame, exception_frame })
                },
                TypeSafetyResult::NeedToLoad(s) => unimplemented!(),
            }
        },
    }
}
