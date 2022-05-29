use std::convert::TryFrom;
use std::fmt::Debug;
use gc_memory_layout_common::layout::ArrayMemoryLayout;
use rust_jvm_common::compressed_classfile::{CompressedParsedDescriptorType, CPDType};
use rust_jvm_common::runtime_type::RuntimeType;
use crate::interpreter::PostInstructionAction;
use crate::interpreter::real_interpreter_state::{InterpreterFrame, RealInterpreterStateGuard};
use crate::JVMState;

fn generic_store<'gc, 'l, 'k, 'j>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>, n: u16, runtime_type: RuntimeType) -> PostInstructionAction<'gc> {
    let object_ref = current_frame.pop(runtime_type);
    current_frame.local_set(n, object_ref);
    PostInstructionAction::Next {}
}

pub fn astore<'gc, 'j, 'k, 'l>(jvm: &'gc JVMState<'gc>, current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>, n: u16) -> PostInstructionAction<'gc> {
    let runtime_type = RuntimeType::object();
    generic_store(current_frame, n, runtime_type)
}

pub fn istore<'gc, 'j, 'k, 'l>(jvm: &'gc JVMState<'gc>, current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>, n: u16) -> PostInstructionAction<'gc> {
    let runtime_type = RuntimeType::IntType;
    generic_store(current_frame, n, runtime_type)
}

pub fn lstore<'gc, 'j, 'k, 'l>(jvm: &'gc JVMState<'gc>, current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>, n: u16) -> PostInstructionAction<'gc> {
    let runtime_type = RuntimeType::LongType;
    generic_store(current_frame, n, runtime_type)
}

pub fn dstore<'gc, 'j, 'k, 'l>(jvm: &'gc JVMState<'gc>, current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>, n: u16) -> PostInstructionAction<'gc> {
    let runtime_type = RuntimeType::DoubleType;
    generic_store(current_frame, n, runtime_type)
}

pub fn fstore<'gc, 'j, 'k, 'l>(jvm: &'gc JVMState<'gc>, current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>, n: u16) -> PostInstructionAction<'gc> {
    let runtime_type = RuntimeType::FloatType;
    generic_store(current_frame, n, runtime_type)
}


pub fn castore<'gc, 'l, 'k>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>) -> PostInstructionAction<'gc>{
    let array_sub_type = CPDType::CharType;
    generic_array_store::<u16>(int_state, array_sub_type)
}

pub fn fastore<'gc, 'l, 'k>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>) -> PostInstructionAction<'gc>{
    let array_sub_type = CPDType::FloatType;
    generic_array_store::<u32>(int_state, array_sub_type)
}

pub fn dastore<'gc, 'l, 'k>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>) -> PostInstructionAction<'gc>{
    let array_sub_type = CPDType::DoubleType;
    generic_array_store::<u64>(int_state, array_sub_type)
}

pub fn bastore<'gc, 'l, 'k>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>) -> PostInstructionAction<'gc>{
    let array_sub_type = CPDType::ByteType;
    generic_array_store::<u8>(int_state, array_sub_type)
}

pub fn lastore<'gc, 'l, 'k>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>) -> PostInstructionAction<'gc>{
    let array_sub_type = CPDType::LongType;
    generic_array_store::<u64>(int_state, array_sub_type)
}

pub fn sastore<'gc, 'l, 'k>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>) -> PostInstructionAction<'gc>{
    let array_sub_type = CPDType::ShortType;
    generic_array_store::<u64>(int_state, array_sub_type)
}

pub fn iastore<'gc, 'l, 'k>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>) -> PostInstructionAction<'gc>{
    let array_sub_type = CPDType::IntType;
    generic_array_store::<u32>(int_state, array_sub_type)
}

pub fn aastore<'gc, 'l, 'k>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>) -> PostInstructionAction<'gc>{
    let array_sub_type = CPDType::object();
    generic_array_store::<u64>(int_state, array_sub_type)
}

fn generic_array_store<'gc, 'l, 'k, T: TryFrom<u64>>(int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>, array_sub_type: CompressedParsedDescriptorType) -> PostInstructionAction<'gc> where <T as TryFrom<u64>>::Error: Debug{
    let mut current_frame = int_state.current_frame_mut();
    let val = current_frame.pop(array_sub_type.to_runtime_type().unwrap()).to_raw();
    let index = current_frame.pop(RuntimeType::IntType).unwrap_int();
    let arrar_ref_o = match current_frame.pop(RuntimeType::object()).unwrap_object() {
        Some(x) => x,
        None => {
            todo!()
            /*return throw_npe(jvm, int_state);*/
        }
    };
    let val = T::try_from(val).unwrap();
    let array_layout = ArrayMemoryLayout::from_cpdtype(array_sub_type);
    unsafe {
        let target_char_ptr = arrar_ref_o.as_ptr().offset(array_layout.elem_0_entry_offset() as isize).offset((array_layout.elem_size() * index as usize) as isize) as *mut T;
        target_char_ptr.write(val);
    }
    PostInstructionAction::Next {}
}
