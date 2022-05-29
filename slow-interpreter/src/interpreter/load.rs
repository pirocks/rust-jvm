use gc_memory_layout_common::layout::ArrayMemoryLayout;
use rust_jvm_common::compressed_classfile::{CompressedParsedDescriptorType, CPDType};
use rust_jvm_common::compressed_classfile::names::CClassName;
use rust_jvm_common::runtime_type::RuntimeType;
use crate::interpreter::PostInstructionAction;
use crate::interpreter::real_interpreter_state::{InterpreterFrame, InterpreterJavaValue, RealInterpreterStateGuard};
use crate::JVMState;


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


fn generic_array_load<'gc, 'l, 'k, T: Into<u64>>(int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>, array_sub_type: CompressedParsedDescriptorType) -> PostInstructionAction<'gc> {
    let index = int_state.current_frame_mut().pop(RuntimeType::IntType).unwrap_int();
    let temp = int_state.current_frame_mut().pop(CClassName::object().into());
    let array_layout = ArrayMemoryLayout::from_cpdtype(array_sub_type);
    let array_ptr = temp.unwrap_object().unwrap();
    unsafe {
        if index < 0 || index >= (array_ptr.as_ptr().offset(array_layout.len_entry_offset() as isize) as *mut i32).read() {
            todo!()
            /*return throw_array_out_of_bounds(jvm, int_state, index);*/
        }
    }
    let res_ptr = unsafe { array_ptr.as_ptr().offset(array_layout.elem_0_entry_offset() as isize).offset((array_layout.elem_size() * index as usize) as isize) };
    let res = InterpreterJavaValue::from_raw(unsafe { (res_ptr as *mut T).read() }.into(), array_sub_type.to_runtime_type().unwrap());
    int_state.current_frame_mut().push(res);
    PostInstructionAction::Next {}
}


pub fn caload<'gc, 'l, 'k>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>) -> PostInstructionAction<'gc> {
    let array_sub_type = CPDType::CharType;
    generic_array_load::<u16>(int_state, array_sub_type)
}

pub fn aaload<'gc, 'l, 'k>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>) -> PostInstructionAction<'gc> {
    let array_sub_type = CPDType::CharType;
    generic_array_load::<u64>(int_state, array_sub_type)
}

pub fn iaload<'gc, 'l, 'k>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>) -> PostInstructionAction<'gc> {
    let array_sub_type = CPDType::IntType;
    generic_array_load::<u32>(int_state, array_sub_type)
}

pub fn faload<'gc, 'l, 'k>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>) -> PostInstructionAction<'gc> {
    let array_sub_type = CPDType::FloatType;
    generic_array_load::<u32>(int_state, array_sub_type)
}

pub fn daload<'gc, 'l, 'k>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>) -> PostInstructionAction<'gc> {
    let array_sub_type = CPDType::DoubleType;
    generic_array_load::<u64>(int_state, array_sub_type)
}

pub fn laload<'gc, 'l, 'k>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>) -> PostInstructionAction<'gc> {
    let array_sub_type = CPDType::LongType;
    generic_array_load::<u64>(int_state, array_sub_type)
}

pub fn saload<'gc, 'l, 'k>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>) -> PostInstructionAction<'gc> {
    let array_sub_type = CPDType::ShortType;
    generic_array_load::<u16>(int_state, array_sub_type)
}

pub fn baload<'gc, 'l, 'k>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>) -> PostInstructionAction<'gc> {
    let array_sub_type = CPDType::ByteType;
    generic_array_load::<u8>(int_state, array_sub_type)
}
