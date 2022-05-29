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

pub fn iload<'gc, 'l, 'k>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>, n: u16) -> PostInstructionAction<'gc> {
    let java_val = int_state.current_frame_mut().local_get(n, RuntimeType::IntType);
    java_val.unwrap_int();
    int_state.current_frame_mut().push(java_val);
    PostInstructionAction::Next {}
}

pub fn lload<'gc, 'l, 'k>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>, n: u16) -> PostInstructionAction<'gc> {
    let java_val = int_state.current_frame_mut().local_get(n, RuntimeType::LongType);
    match java_val {
        InterpreterJavaValue::Long(_) => {}
        _ => {
            dbg!(java_val);
            // current_frame.print_stack_trace();
            // dbg!(&current_frame.local_vars(jvm)[1..]);
            panic!()
        }
    }
    int_state.current_frame_mut().push(java_val);
    PostInstructionAction::Next {}
}

pub fn fload<'gc, 'l, 'k>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>, n: u16) -> PostInstructionAction<'gc> {
    let java_val = int_state.current_frame_mut().local_get(n, RuntimeType::FloatType);
    match java_val {
        InterpreterJavaValue::Float(_) => {}
        _ => {
            dbg!(java_val);
            panic!()
        }
    }
    int_state.current_frame_mut().push(java_val);
    PostInstructionAction::Next {}
}
/*
pub fn dload(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, n: u16) {
    let java_val = current_frame.local_vars().get(n, RuntimeType::DoubleType);
    match java_val {
        JavaValue::Double(_) => {}
        _ => {
            dbg!(java_val);
            panic!()
        }
    }
    current_frame.push(java_val)
}

pub fn aaload(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life,'l>) {
    let mut current_frame = int_state.current_frame_mut();
    let index = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let temp = current_frame.pop(Some(CClassName::object().into()));
    let unborrowed = temp.unwrap_array();
    let jv_res = unborrowed.get_i(jvm, index);
    if index < 0 || index >= unborrowed.len() {
        return throw_array_out_of_bounds(jvm, int_state, index);
    }
    match jv_res {
        JavaValue::Object(_) => {}
        _ => panic!(),
    };
    current_frame.push(jv_res.clone())
}
*/
pub fn caload<'gc, 'l, 'k>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>) -> PostInstructionAction<'gc> {
    let array_sub_type = CPDType::CharType;
    generic_array_load::<u16>(int_state, array_sub_type)
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

pub fn aaload<'gc, 'l, 'k>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>) -> PostInstructionAction<'gc> {
    let array_sub_type = CPDType::CharType;
    generic_array_load::<u64>(int_state, array_sub_type)
}
/*
pub fn iaload(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let index = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let temp = current_frame.pop(Some(RuntimeType::object()));
    let unborrowed = temp.unwrap_array();
    let as_int = unborrowed.get_i(jvm, index).unwrap_int();
    current_frame.push(JavaValue::Int(as_int))
}

pub fn laload(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let index = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let temp = current_frame.pop(Some(RuntimeType::object()));
    let unborrowed = temp.unwrap_array();
    let long = unborrowed.get_i(jvm, index).unwrap_long();
    current_frame.push(JavaValue::Long(long))
}

pub fn faload(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let index = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let temp = current_frame.pop(Some(RuntimeType::object()));
    let unborrowed = temp.unwrap_array();
    let f = unborrowed.get_i(jvm, index).unwrap_float();
    current_frame.push(JavaValue::Float(f))
}

pub fn daload(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let index = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let temp = current_frame.pop(Some(RuntimeType::object()));
    let unborrowed = temp.unwrap_array();
    let d = unborrowed.get_i(jvm, index).unwrap_double();
    current_frame.push(JavaValue::Double(d))
}

pub fn saload(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let index = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let temp = current_frame.pop(Some(RuntimeType::object()));
    let unborrowed = temp.unwrap_array();
    let d = unborrowed.get_i(jvm, index).unwrap_short();
    current_frame.push(JavaValue::Short(d))
}

pub fn baload(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let index = current_frame.pop(Some(RuntimeType::IntType)).unwrap_int();
    let temp = current_frame.pop(Some(RuntimeType::object()));
    let unborrowed = temp.unwrap_array();
    let as_byte = match &unborrowed.get_i(jvm, index) {
        JavaValue::Byte(i) => *i,
        JavaValue::Boolean(i) => *i as i8,
        val => {
            dbg!(&unborrowed.elem_type);
            dbg!(val);
            panic!()
        }
    };
    current_frame.push(JavaValue::Int(as_byte as i32))
}
*/