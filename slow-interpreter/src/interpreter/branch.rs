use rust_jvm_common::compressed_classfile::names::CClassName;
use rust_jvm_common::runtime_type::RuntimeType;

use crate::interpreter::PostInstructionAction;
use crate::interpreter::real_interpreter_state::{InterpreterFrame, InterpreterJavaValue};
use crate::jvm_state::JVMState;

pub fn goto_<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, target: i32) -> PostInstructionAction<'gc> {
    PostInstructionAction::NextOffset { offset_change: target }
}

pub fn ifnull<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>, offset: i16) -> PostInstructionAction<'gc> {
    let val = current_frame.pop(CClassName::object().into());
    let succeeds = match val {
        InterpreterJavaValue::Object(o) => o.is_none(),
        _ => panic!(),
    };
    if succeeds {
        PostInstructionAction::NextOffset { offset_change: offset as i32 }
    } else {
        PostInstructionAction::Next {}
    }
}

pub fn ifnonnull<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>, offset: i16) -> PostInstructionAction<'gc> {
    let val = current_frame.pop(CClassName::object().into());
    let succeeds = match val {
        InterpreterJavaValue::Object(o) => o.is_some(),
        _ => panic!(),
    };
    if succeeds {
        PostInstructionAction::NextOffset { offset_change: offset as i32 }
    } else {
        PostInstructionAction::Next {}
    }
}

pub fn ifle<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>, offset: i16) -> PostInstructionAction<'gc> {
    let val = current_frame.pop(RuntimeType::IntType);
    let succeeds = val.unwrap_int() <= 0;
    if succeeds {
        PostInstructionAction::NextOffset { offset_change: offset as i32 }
    } else {
        PostInstructionAction::Next {}
    }
}

pub fn ifgt<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>, offset: i16) -> PostInstructionAction<'gc> {
    let val = current_frame.pop(RuntimeType::IntType);
    let succeeds = val.unwrap_int() > 0;
    if succeeds {
        PostInstructionAction::NextOffset { offset_change: offset as i32 }
    } else {
        PostInstructionAction::Next {}
    }
}

pub fn ifge<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>, offset: i16) -> PostInstructionAction<'gc> {
    let val = current_frame.pop(RuntimeType::IntType);
    let succeeds = val.unwrap_int() >= 0;
    if succeeds {
        PostInstructionAction::NextOffset { offset_change: offset as i32 }
    } else {
        PostInstructionAction::Next {}
    }
}

pub fn iflt<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>, offset: i16) -> PostInstructionAction<'gc> {
    let val = current_frame.pop(RuntimeType::IntType);
    let succeeds = val.unwrap_int() < 0;
    if succeeds {
        PostInstructionAction::NextOffset { offset_change: offset as i32 }
    } else {
        PostInstructionAction::Next {}
    }
}

pub fn ifne<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>, offset: i16) -> PostInstructionAction<'gc> {
    let val = current_frame.pop(RuntimeType::IntType);
    let succeeds = val.unwrap_int() != 0;
    if succeeds {
        PostInstructionAction::NextOffset { offset_change: offset as i32 }
    } else {
        PostInstructionAction::Next {}
    }
}

pub fn ifeq<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>, offset: i16) -> PostInstructionAction<'gc> {
    //todo dup
    let val = current_frame.pop(RuntimeType::IntType);
    let succeeds = val.unwrap_int() == 0;
    if succeeds {
        PostInstructionAction::NextOffset { offset_change: offset as i32 }
    } else {
        PostInstructionAction::Next {}
    }
}

pub fn if_icmpgt<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>, offset: i16) -> PostInstructionAction<'gc> {
    let value2 = current_frame.pop(RuntimeType::IntType).unwrap_int();
    let value1 = current_frame.pop(RuntimeType::IntType).unwrap_int();
    let succeeds = value1 > value2;
    if succeeds {
        PostInstructionAction::NextOffset { offset_change: offset as i32 }
    } else {
        PostInstructionAction::Next {}
    }
}

pub fn if_icmplt<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>, offset: i16) -> PostInstructionAction<'gc> {
    let value2 = current_frame.pop(RuntimeType::IntType).unwrap_int();
    let value1 = current_frame.pop(RuntimeType::IntType).unwrap_int();
    let succeeds = value1 < value2;
    if succeeds {
        PostInstructionAction::NextOffset { offset_change: offset as i32 }
    } else {
        PostInstructionAction::Next {}
    }
}

pub fn if_icmple<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>, offset: i16) -> PostInstructionAction<'gc> {
    let value2 = current_frame.pop(RuntimeType::IntType).unwrap_int();
    let value1 = current_frame.pop(RuntimeType::IntType).unwrap_int();
    let succeeds = value1 <= value2;
    if succeeds {
        PostInstructionAction::NextOffset { offset_change: offset as i32 }
    } else {
        PostInstructionAction::Next {}
    }
}

pub fn if_icmpge<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>, offset: i16) -> PostInstructionAction<'gc> {
    let value2 = current_frame.pop(RuntimeType::IntType).unwrap_int();
    let value1 = current_frame.pop(RuntimeType::IntType).unwrap_int();
    let succeeds = value1 >= value2;
    if succeeds {
        PostInstructionAction::NextOffset { offset_change: offset as i32 }
    } else {
        PostInstructionAction::Next {}
    }
}

pub fn if_icmpne<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>, offset: i16) -> PostInstructionAction<'gc> {
    let value2 = current_frame.pop(RuntimeType::IntType).unwrap_int();
    let value1 = current_frame.pop(RuntimeType::IntType).unwrap_int();
    let succeeds = value1 != value2;
    if succeeds {
        PostInstructionAction::NextOffset { offset_change: offset as i32 }
    } else {
        PostInstructionAction::Next {}
    }
}

pub fn if_icmpeq<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>, offset: i16) -> PostInstructionAction<'gc> {
    let value2 = current_frame.pop(RuntimeType::IntType).unwrap_int();
    let value1 = current_frame.pop(RuntimeType::IntType).unwrap_int();
    let succeeds = value1 == value2;
    if succeeds {
        PostInstructionAction::NextOffset { offset_change: offset as i32 }
    } else {
        PostInstructionAction::Next {}
    }
}

pub fn if_acmpne<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>, offset: i16) -> PostInstructionAction<'gc> {
    let value2 = current_frame.pop(RuntimeType::object());
    let value1 = current_frame.pop(RuntimeType::object());
    let succeeds = !equal_ref(value2, value1);
    if succeeds {
        PostInstructionAction::NextOffset { offset_change: offset as i32 }
    } else {
        PostInstructionAction::Next {}
    }
}

pub fn if_acmpeq<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>, offset: i16) -> PostInstructionAction<'gc> {
    let value2 = current_frame.pop(RuntimeType::object());
    let value1 = current_frame.pop(RuntimeType::object());
    let succeeds = equal_ref(value2, value1);
    if succeeds {
        PostInstructionAction::NextOffset { offset_change: offset as i32 }
    } else {
        PostInstructionAction::Next {}
    }
}

fn equal_ref<'gc>(value2: InterpreterJavaValue, value1: InterpreterJavaValue) -> bool {
    match value1 {
        InterpreterJavaValue::Object(o1) => match value2 {
            InterpreterJavaValue::Object(o2) => match o1 {
                None => o2.is_none(),
                Some(o1_ptr) => match o2 {
                    None => false,
                    Some(o2_ptr) => o1_ptr == o2_ptr,
                },
            },
            _ => panic!(),
        },
        _ => panic!(),
    }
}

