use rust_jvm_common::classfile::Atype;
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::CPDType;


use rust_jvm_common::runtime_type::RuntimeType;

use crate::{AllocatedHandle, check_initing_or_inited_class, JavaValueCommon, JVMState, NewJavaValueHandle, WasException};
use crate::better_java_stack::frames::HasFrame;
use crate::class_loading::check_resolved_class;
use crate::interpreter::PostInstructionAction;
use crate::interpreter::real_interpreter_state::{InterpreterJavaValue, RealInterpreterStateGuard};
use crate::interpreter_util::new_object;
use crate::ir_to_java_layer::exit_impls::multi_allocate_array::multi_new_array_impl;
use crate::java_values::default_value;

pub fn new<'gc, 'k, 'l>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>, classname: CClassName) -> PostInstructionAction<'gc> {
    let target_classfile = match check_initing_or_inited_class(jvm, int_state.inner(), classname.into()) {
        Ok(x) => x,
        Err(WasException { exception_obj }) => {
            // int_state.throw().unwrap().lookup_field(jvm, FieldName::field_detailMessage());
            return PostInstructionAction::Exception { exception: WasException { exception_obj } };
        }
    };
    let obj = new_object(jvm, int_state.inner(), &target_classfile, false);
    int_state.current_frame_mut().push(NewJavaValueHandle::Object(AllocatedHandle::NormalObject(obj)).to_interpreter_jv());
    PostInstructionAction::Next {}
}

pub fn anewarray<'gc, 'k, 'l>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>, cpdtype: &CPDType) -> PostInstructionAction<'gc> {
    let len = match int_state.current_frame_mut().pop(RuntimeType::IntType) {
        InterpreterJavaValue::Int(i) => i,
        _ => panic!(),
    };
    let type_ = cpdtype.clone();
    if let Err(WasException { exception_obj }) = a_new_array_from_name(jvm, int_state, len, type_) {
        return PostInstructionAction::Exception { exception: WasException { exception_obj } };
    }
    PostInstructionAction::Next {}
}

pub fn a_new_array_from_name<'gc, 'k, 'l>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>, len: i32, elem_type: CPDType) -> Result<(), WasException<'gc>> {
    if len < 0 {
        todo!("check array length");
    }
    let whole_array_runtime_class = check_resolved_class(jvm, int_state.inner(), CPDType::array(elem_type))?;
    let new_array = NewJavaValueHandle::new_default_array(jvm, len, whole_array_runtime_class, elem_type);
    Ok(int_state.current_frame_mut().push(new_array.to_interpreter_jv()))
}

pub fn newarray<'gc, 'k, 'l>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>, a_type: Atype) -> PostInstructionAction<'gc> {
    let count = int_state.current_frame_mut().pop(RuntimeType::IntType).unwrap_int();
    let type_ = match a_type {
        Atype::TChar => CPDType::CharType,
        Atype::TInt => CPDType::IntType,
        Atype::TByte => CPDType::ByteType,
        Atype::TBoolean => CPDType::BooleanType,
        Atype::TShort => CPDType::ShortType,
        Atype::TLong => CPDType::LongType,
        Atype::TDouble => CPDType::DoubleType,
        Atype::TFloat => CPDType::FloatType,
    };
    if count < 0 {
        int_state.inner().debug_print_stack_trace(jvm);
        todo!("check array length, this one seems like it would be easy to debug");
    }
    match a_new_array_from_name(jvm, int_state, count, type_) {
        Ok(arr) => PostInstructionAction::Next {},
        Err(WasException { exception_obj }) => PostInstructionAction::Exception { exception: WasException { exception_obj } },
    }
}

pub fn multi_a_new_array<'gc, 'k, 'l>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>, dims: u8, type_: CPDType) -> PostInstructionAction<'gc> {
    if let Err(_) = check_resolved_class(jvm, int_state.inner(), type_) {
        return todo!();
    };
    let mut elem_type = type_;
    let mut dimensions = vec![];
    for _ in 0..dims {
        elem_type = elem_type.unwrap_array_type();
        dimensions.push(int_state.current_frame_mut().pop(RuntimeType::IntType).unwrap_int());
    }
    dimensions.reverse();
    let array_type = type_;
    let rc = check_initing_or_inited_class(jvm, int_state.inner(), array_type).unwrap();
    let default = default_value(elem_type);
    let res = multi_new_array_impl(jvm, array_type, dimensions.as_slice(), default.as_njv());
    int_state.current_frame_mut().push(res.to_interpreter_jv());
    PostInstructionAction::Next {}
}
