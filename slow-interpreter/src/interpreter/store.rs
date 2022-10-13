use gc_memory_layout_common::layout::ArrayMemoryLayout;
use rust_jvm_common::compressed_classfile::compressed_types::{CompressedParsedDescriptorType, CPDType};


use rust_jvm_common::runtime_type::RuntimeType;

use crate::interpreter::PostInstructionAction;
use crate::interpreter::real_interpreter_state::InterpreterFrame;
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


pub fn castore<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let array_sub_type = CPDType::CharType;
    generic_array_store::<u16>(current_frame, array_sub_type)
}

pub fn fastore<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let array_sub_type = CPDType::FloatType;
    generic_array_store::<u32>(current_frame, array_sub_type)
}

pub fn dastore<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let array_sub_type = CPDType::DoubleType;
    generic_array_store::<u64>(current_frame, array_sub_type)
}

pub fn bastore<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let array_sub_type = CPDType::ByteType;
    generic_array_store::<u8>(current_frame, array_sub_type)
}

pub fn lastore<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let array_sub_type = CPDType::LongType;
    generic_array_store::<u64>(current_frame, array_sub_type)
}

pub fn sastore<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let array_sub_type = CPDType::ShortType;
    generic_array_store::<u64>(current_frame, array_sub_type)
}

pub fn iastore<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let array_sub_type = CPDType::IntType;
    generic_array_store::<u32>(current_frame, array_sub_type)
}

pub fn aastore<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let array_sub_type = CPDType::object();
    generic_array_store::<u64>(current_frame, array_sub_type)
}

trait CastFromU64 {
    fn cast(as_u64: u64) -> Self;
}

impl CastFromU64 for u64 {
    fn cast(as_u64: u64) -> Self {
        as_u64
    }
}

impl CastFromU64 for u32 {
    fn cast(as_u64: u64) -> Self {
        as_u64 as u32
    }
}

impl CastFromU64 for u16 {
    fn cast(as_u64: u64) -> Self {
        as_u64 as u16
    }
}

impl CastFromU64 for u8 {
    fn cast(as_u64: u64) -> Self {
        as_u64 as u8
    }
}


fn generic_array_store<'gc, 'l, 'k, 'j, T: CastFromU64>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>, array_sub_type: CompressedParsedDescriptorType) -> PostInstructionAction<'gc> {
    let val = current_frame.pop(array_sub_type.to_runtime_type().unwrap()).to_raw();
    let index = current_frame.pop(RuntimeType::IntType).unwrap_int();
    let arrar_ref_o = match current_frame.pop(RuntimeType::object()).unwrap_object() {
        Some(x) => x,
        None => {
            todo!()
            /*return throw_npe(jvm, int_state);*/
        }
    };
    let val = T::cast(val);
    let array_layout = ArrayMemoryLayout::from_cpdtype(array_sub_type);
    unsafe {
        let target_char_ptr = arrar_ref_o.as_ptr().offset(array_layout.elem_0_entry_offset() as isize).offset((array_layout.elem_size() * index as usize) as isize) as *mut T;
        target_char_ptr.write(val);
    }
    PostInstructionAction::Next {}
}
