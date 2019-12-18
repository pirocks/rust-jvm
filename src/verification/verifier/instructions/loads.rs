use verification::verifier::codecorrectness::Environment;
use verification::verifier::{Frame, TypeSafetyResult};
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

fn load_is_type_safe(env: &Environment, index: usize, type_: &UnifiedType, frame: &Frame) -> Result<Frame,TypeSafetyResult> {
    let locals = &frame.locals;
    let actual_type = nth0(index, locals);
    let next_frame_res = valid_type_transition(env, vec![], &actual_type, frame);

    match next_frame_res {
        Ok(next_frame) => {
            if is_assignable(&actual_type, type_) {
                Ok(next_frame)
            } else {
                unimplemented!()
            }
        }
        Err(e) => {Result::Err(e)}
    }
}

pub fn instruction_is_type_safe_lload(index: usize, env: &Environment, _offset: usize, stack_frame: &Frame) -> InstructionIsTypeSafeResult {
    let next_frame = match load_is_type_safe(env,index,&UnifiedType::LongType,stack_frame){
        Ok(nf) => nf,
        Err(_) => return InstructionIsTypeSafeResult::NotSafe,
    };
    let exception_frame = exception_stack_frame(stack_frame);
    InstructionIsTypeSafeResult::Safe(ResultFrames {exception_frame,next_frame})
}


pub fn instruction_is_type_safe_aload(index: usize, env: &Environment, offset: usize, stack_frame: &Frame) -> InstructionIsTypeSafeResult {
    let next_frame = match load_is_type_safe(env, index, &UnifiedType::Reference, stack_frame){
        Ok(nf) => {nf},
        Err(e) => {return InstructionIsTypeSafeResult::NotSafe;}
    };
    let exception_frame = exception_stack_frame(stack_frame);
    return InstructionIsTypeSafeResult::Safe(ResultFrames {
        exception_frame,
        next_frame,
    });
}