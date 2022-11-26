use rust_jvm_common::compressed_classfile::compressed_types::CPDType;
use rust_jvm_common::runtime_type::RuntimeType;
use crate::exceptions::WasException;

use crate::interpreter::common::special::invoke_instanceof;
use crate::interpreter::PostInstructionAction;
use crate::interpreter::real_interpreter_state::{InterpreterFrame, InterpreterJavaValue, RealInterpreterStateGuard};
use crate::JVMState;
use crate::new_java_values::owned_casts::OwnedCastAble;
use crate::stdlib::java::lang::class_cast_exception::ClassCastException;
use crate::stdlib::java::NewAsObjectOrJavaValue;

pub fn arraylength<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let array_o = match current_frame.pop(RuntimeType::object()).unwrap_object() {
        Some(x) => x,
        None => {
            todo!()
            /*return throw_npe(jvm, int_state);*/
        }
    };
    //todo use ArrayMemoryLayout
    let len = unsafe { (array_o.as_ptr() as *const i32).read() };
    current_frame.push(InterpreterJavaValue::Int(len));
    PostInstructionAction::Next {}
}

pub fn checkcast<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>, cpdtype: CPDType) -> PostInstructionAction<'gc> {
    let obj = int_state.current_frame_mut().pop(RuntimeType::object());
    if obj.unwrap_object().is_none() {
        int_state.current_frame_mut().push(obj);
        return PostInstructionAction::Next {};
    }
    int_state.current_frame_mut().push(obj);
    invoke_instanceof(jvm, int_state.current_frame_mut(), cpdtype);
    let res = int_state.current_frame_mut().pop(RuntimeType::IntType).unwrap_int();
    int_state.current_frame_mut().push(obj);
    if res == 0 {
        let class_cast_exception = ClassCastException::new(jvm, int_state.inner()).unwrap();
        return PostInstructionAction::Exception { exception: WasException { exception_obj: class_cast_exception.object().cast_throwable() } };
    }
    PostInstructionAction::Next {}
}