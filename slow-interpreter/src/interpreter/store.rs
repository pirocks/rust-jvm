use rust_jvm_common::runtime_type::RuntimeType;
use crate::interpreter::PostInstructionAction;
use crate::interpreter::real_interpreter_state::RealInterpreterStateGuard;
use crate::JVMState;

pub fn astore<'gc, 'l, 'k>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>, n: u16) -> PostInstructionAction<'gc>{
    let mut current_frame = int_state.current_frame_mut();
    let object_ref = current_frame.pop(RuntimeType::object());
    current_frame.local_set(n, object_ref);
    PostInstructionAction::Next {}
}

pub fn lstore<'gc, 'l, 'k>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>, n: u16) {
    /*let val = current_frame.pop(Some(RuntimeType::LongType));
    match val {
        JavaValue::Long(_) => {}
        _ => {
            dbg!(&val);
            panic!()
        }
    }
    current_frame.local_vars_mut().set(n, val);*/
    todo!()
}

pub fn dstore<'gc, 'l, 'k>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>, n: u16) {
    /*let jv = current_frame.pop(Some(RuntimeType::DoubleType));
    match jv {
        JavaValue::Double(_) => {}
        _ => {
            dbg!(&jv);
            panic!()
        }
    }
    current_frame.local_vars_mut().set(n, jv);*/
    todo!()
}

pub fn fstore<'gc, 'l, 'k>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>, n: u16) {
    /*let jv: JavaValue<'gc_life> = current_frame.pop(Some(RuntimeType::FloatType));
    jv.unwrap_float();
    let mut vars_mut: LocalVarsMut<'gc_life,'l,'_> = current_frame.local_vars_mut();
    vars_mut.set(n, jv);*/
    todo!()
}
