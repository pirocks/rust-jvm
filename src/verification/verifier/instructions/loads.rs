use verification::verifier::codecorrectness::Environment;
use verification::verifier::Frame;
use verification::verifier::filecorrectness::is_assignable;
use verification::verifier::codecorrectness::valid_type_transition;
use verification::verifier::instructions::nth0;
use verification::unified_type::UnifiedType;
use verification::verifier::instructions::InstructionIsTypeSafeResult;
use verification::verifier::instructions::exception_stack_frame;
use verification::verifier::instructions::ResultFrames;

#[allow(unused)]
fn instruction_is_type_safe_aaload(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn load_is_type_safe(env: &Environment, index: usize, type_: &UnifiedType, frame: &Frame) -> Frame {
    let locals = &frame.locals;
    let actual_type = nth0(index, locals);
    let next_frame = valid_type_transition(env, vec![], &actual_type, frame);

    if is_assignable(&actual_type, type_) {
        return next_frame;
    }else {
        unimplemented!()
    }
}

pub fn instruction_is_type_safe_aload(index: usize, env: &Environment, offset: usize, stack_frame: &Frame) -> InstructionIsTypeSafeResult {
    let next_frame= load_is_type_safe(env,index,&UnifiedType::Reference,stack_frame);
    let exception_frame = exception_stack_frame(stack_frame);
    return InstructionIsTypeSafeResult::Safe(ResultFrames{
        exception_frame,next_frame
    })
}