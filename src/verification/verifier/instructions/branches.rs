use verification::verifier::codecorrectness::Environment;
use verification::verifier::Frame;
use verification::unified_type::UnifiedType;
use verification::verifier::instructions::InstructionIsTypeSafeResult;
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
