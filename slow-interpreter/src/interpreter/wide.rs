use rust_jvm_common::classfile::{IInc, Wide, WideAload, WideAstore, WideDload, WideDstore, WideFload, WideFstore, WideIload, WideIstore, WideLload, WideLstore, WideRet};
use rust_jvm_common::runtime_type::RuntimeType;
use crate::interpreter::load::{aload, dload, fload, iload, lload};
use crate::interpreter::PostInstructionAction;
use crate::interpreter::real_interpreter_state::{InterpreterFrame, InterpreterJavaValue};
use crate::interpreter::store::{astore, dstore, fstore, istore, lstore};
use crate::JVMState;

pub fn wide<'gc, 'j, 'k, 'l,'h>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j, 'h>, w: &Wide) -> PostInstructionAction<'gc>{
    match w {
        Wide::Iload(WideIload { index }) => iload(jvm, current_frame, *index),
        Wide::Fload(WideFload { index }) => fload(jvm, current_frame, *index),
        Wide::Aload(WideAload { index }) => aload(current_frame, *index),
        Wide::Lload(WideLload { index }) => lload(jvm, current_frame, *index),
        Wide::Dload(WideDload { index }) => dload(jvm, current_frame, *index),
        Wide::Istore(WideIstore { index }) => istore(jvm, current_frame, *index),
        Wide::Fstore(WideFstore { index }) => fstore(jvm, current_frame, *index),
        Wide::Astore(WideAstore { index }) => astore(jvm,current_frame, *index),
        Wide::Lstore(WideLstore { index }) => lstore(jvm, current_frame, *index),
        Wide::Ret(WideRet { index }) => todo!()/*ret(jvm, current_frame, *index)*/,
        Wide::Dstore(WideDstore { index }) => dstore(jvm, current_frame, *index),
        Wide::IInc(iinc) => {
            let IInc { index, const_ } = iinc;
            let mut val = current_frame.local_get(*index, RuntimeType::IntType).unwrap_int();
            val += *const_ as i32;
            current_frame.local_set(*index, InterpreterJavaValue::Int(val));
            PostInstructionAction::Next {}
        }
    }
}
