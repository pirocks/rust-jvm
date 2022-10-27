use runtime_class_stuff::array_layout::ArrayMemoryLayout;
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::{CompressedParsedDescriptorType, CPDType};


use rust_jvm_common::runtime_type::RuntimeType;
use crate::accessor_ext::ArrayAccessorExt;

use crate::JVMState;
use crate::better_java_stack::frames::HasFrame;
use crate::interpreter::PostInstructionAction;
use crate::interpreter::real_interpreter_state::{InterpreterFrame, InterpreterJavaValue};
use crate::utils::throw_array_out_of_bounds_res;

pub fn aload<'gc, 'l, 'k, 'j>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>, n: u16) -> PostInstructionAction<'gc> {
    let ref_: InterpreterJavaValue = current_frame.local_get(n, RuntimeType::object());
    match ref_ {
        InterpreterJavaValue::Object(_) => {}
        _ => {
            dbg!(ref_);
            dbg!(n);
            panic!()
        }
    }
    current_frame.push(ref_);
    PostInstructionAction::Next {}
}

pub fn iload<'gc, 'j, 'k, 'l>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>, n: u16) -> PostInstructionAction<'gc> {
    let java_val = current_frame.local_get(n, RuntimeType::IntType);
    current_frame.push(java_val);
    PostInstructionAction::Next {}
}

pub fn lload<'gc, 'j, 'k, 'l>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>, n: u16) -> PostInstructionAction<'gc> {
    let java_val = current_frame.local_get(n, RuntimeType::LongType);
    match java_val {
        InterpreterJavaValue::Long(_) => {}
        _ => {
            dbg!(java_val);
            panic!()
        }
    }
    current_frame.push(java_val);
    PostInstructionAction::Next {}
}

pub fn fload<'gc, 'j, 'k, 'l>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>, n: u16) -> PostInstructionAction<'gc> {
    let java_val = current_frame.local_get(n, RuntimeType::FloatType);
    match java_val {
        InterpreterJavaValue::Float(_) => {}
        _ => {
            dbg!(java_val);
            panic!()
        }
    }
    current_frame.push(java_val);
    PostInstructionAction::Next {}
}

pub fn dload<'gc, 'j, 'k, 'l>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>, n: u16) -> PostInstructionAction<'gc> {
    let java_val = current_frame.local_get(n, RuntimeType::DoubleType);
    match java_val {
        InterpreterJavaValue::Double(_) => {}
        _ => {
            dbg!(java_val);
            panic!()
        }
    }
    current_frame.push(java_val);
    PostInstructionAction::Next {}
}


fn generic_array_load<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>, array_sub_type: CompressedParsedDescriptorType) -> PostInstructionAction<'gc> {
    let index = current_frame.pop(RuntimeType::IntType).unwrap_int();
    let temp = current_frame.pop(CClassName::object().into());
    let array_layout = ArrayMemoryLayout::from_cpdtype(array_sub_type);
    let array_ptr = match temp.unwrap_object() {
        Some(x) => x,
        None => {
            current_frame.inner().inner().debug_print_stack_trace(jvm);
            panic!()
        },
    };
    unsafe {
        let array_len = array_layout.calculate_len_address(array_ptr).as_ptr().read();
        if index < 0 || index >= array_len {
            return PostInstructionAction::Exception { exception: throw_array_out_of_bounds_res::<i64>(jvm, current_frame.inner().inner(), index).unwrap_err() };
        }
    }
    let accessor = array_layout.calculate_index_address(array_ptr,index);
    let res: InterpreterJavaValue = accessor.read_interpreter_jv(array_sub_type);
    current_frame.push(res);
    PostInstructionAction::Next {}
}


pub fn caload<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let array_sub_type = CPDType::CharType;
    generic_array_load(jvm, current_frame, array_sub_type)
}

pub fn aaload<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let array_sub_type = CPDType::object();
    generic_array_load(jvm, current_frame, array_sub_type)
}

pub fn iaload<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let array_sub_type = CPDType::IntType;
    generic_array_load(jvm, current_frame, array_sub_type)
}

pub fn faload<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let array_sub_type = CPDType::FloatType;
    generic_array_load(jvm, current_frame, array_sub_type)
}

pub fn daload<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let array_sub_type = CPDType::DoubleType;
    generic_array_load(jvm, current_frame, array_sub_type)
}

pub fn laload<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let array_sub_type = CPDType::LongType;
    generic_array_load(jvm, current_frame, array_sub_type)
}

pub fn saload<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let array_sub_type = CPDType::ShortType;
    generic_array_load(jvm, current_frame, array_sub_type)
}

pub fn baload<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let array_sub_type = CPDType::ByteType;
    generic_array_load(jvm, current_frame, array_sub_type)
}
