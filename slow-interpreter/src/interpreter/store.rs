use std::convert::TryFrom;
use std::fmt::Debug;
use gc_memory_layout_common::layout::ArrayMemoryLayout;
use rust_jvm_common::compressed_classfile::{CompressedParsedDescriptorType, CPDType};
use rust_jvm_common::runtime_type::RuntimeType;
use crate::interpreter::PostInstructionAction;
use crate::interpreter::real_interpreter_state::RealInterpreterStateGuard;
use crate::JVMState;

pub fn astore<'gc, 'l, 'k>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>, n: u16) -> PostInstructionAction<'gc> {
    let mut current_frame = int_state.current_frame_mut();
    let object_ref = current_frame.pop(RuntimeType::object());
    current_frame.local_set(n, object_ref);
    PostInstructionAction::Next {}
}

pub fn istore<'gc, 'l, 'k>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>, n: u16) -> PostInstructionAction<'gc> {
    let mut current_frame = int_state.current_frame_mut();
    let object_ref = current_frame.pop(RuntimeType::IntType);
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


pub fn castore<'gc, 'l, 'k>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>) -> PostInstructionAction<'gc>{
    let array_sub_type = CPDType::CharType;
    generic_array_store::<u16>(int_state, array_sub_type)
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
