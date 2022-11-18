use array_memory_layout::layout::ArrayMemoryLayout;
use rust_jvm_common::compressed_classfile::compressed_types::{CompressedParsedDescriptorType, CPDType};


use rust_jvm_common::runtime_type::RuntimeType;
use crate::accessor_ext::AccessorExt;
use crate::better_java_stack::frames::HasFrame;
use crate::exceptions::WasException;

use crate::interpreter::PostInstructionAction;
use crate::interpreter::real_interpreter_state::{InterpreterFrame};
use crate::JVMState;
use crate::new_java_values::owned_casts::OwnedCastAble;
use crate::stdlib::java::lang::null_pointer_exception::NullPointerException;
use crate::stdlib::java::NewAsObjectOrJavaValue;

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
    generic_array_store(current_frame, array_sub_type)
}

pub fn fastore<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let array_sub_type = CPDType::FloatType;
    generic_array_store(current_frame, array_sub_type)
}

pub fn dastore<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let array_sub_type = CPDType::DoubleType;
    generic_array_store(current_frame, array_sub_type)
}

pub fn bastore<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let array_sub_type = CPDType::ByteType;
    generic_array_store(current_frame, array_sub_type)
}

pub fn lastore<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let array_sub_type = CPDType::LongType;
    generic_array_store(current_frame, array_sub_type)
}

pub fn sastore<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let array_sub_type = CPDType::ShortType;
    generic_array_store(current_frame, array_sub_type)
}

pub fn iastore<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let array_sub_type = CPDType::IntType;
    generic_array_store(current_frame, array_sub_type)
}

pub fn aastore<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let array_sub_type = CPDType::object();
    generic_array_store(current_frame, array_sub_type)
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


fn generic_array_store<'gc, 'l, 'k, 'j>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>, array_sub_type: CompressedParsedDescriptorType) -> PostInstructionAction<'gc> {
    let val = current_frame.pop(array_sub_type.to_runtime_type().unwrap());
    let index = current_frame.pop(RuntimeType::IntType).unwrap_int();
    let arrar_ref_o = match current_frame.pop(RuntimeType::object()).unwrap_object() {
        Some(x) => x,
        None => {
            let jvm = current_frame.inner().inner().jvm();
            let npe = NullPointerException::new(jvm, current_frame.inner().inner()).expect("Exception creating exception").object().cast_throwable();
            return PostInstructionAction::Exception { exception: WasException { exception_obj: npe } }
        }
    };
    let array_layout = ArrayMemoryLayout::from_cpdtype(array_sub_type);
    let accessor = array_layout.calculate_index_address(arrar_ref_o,index);
    accessor.write_interpreter_jv(val, array_sub_type);
    PostInstructionAction::Next {}
}
