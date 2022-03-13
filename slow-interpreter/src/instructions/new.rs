use rust_jvm_common::classfile::Atype;
use rust_jvm_common::compressed_classfile::{CPDType, CPRefType};
use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName};
use rust_jvm_common::runtime_type::RuntimeType;

use crate::{InterpreterStateGuard, JVMState, NewJavaValue};
use crate::class_loading::{check_initing_or_inited_class, check_resolved_class};
use crate::interpreter::WasException;
use crate::interpreter_util::new_object;
use crate::java_values::{ArrayObject, default_value, JavaValue, Object};
use crate::new_java_values::NewJavaValueHandle;

pub fn new<'gc_life, 'l>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life,'l>, classname: CClassName) {
    let target_classfile = match check_initing_or_inited_class(jvm, int_state, classname.into()) {
        Ok(x) => x,
        Err(WasException {}) => {
            int_state.debug_print_stack_trace(jvm);
            // int_state.throw().unwrap().lookup_field(jvm, FieldName::field_detailMessage());
            return;
        }
    };
    let obj = new_object(jvm, int_state, &target_classfile).to_jv();
    int_state.push_current_operand_stack(obj);
}

pub fn anewarray<'gc_life, 'l>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life,'l>, cpdtype: &CPDType) {
    let len = match int_state.current_frame_mut().pop(Some(RuntimeType::IntType)) {
        JavaValue::Int(i) => i,
        _ => panic!(),
    };
    let type_ = cpdtype.clone();
    if let Err(_) = a_new_array_from_name(jvm, int_state, len, type_) {
        return;
    }
}

pub fn a_new_array_from_name<'l, 'gc_life>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life,'l>, len: i32, t: CPDType) -> Result<NewJavaValueHandle<'gc_life>, WasException> {
    check_resolved_class(jvm, int_state, t.clone())?;
    let new_array = JavaValue::new_vec(jvm, int_state, len as usize, NewJavaValue::Null, t)?;
    Ok(NewJavaValueHandle::Object(new_array))
    /*Ok(int_state.push_current_operand_stack(JavaValue::Object(Some(new_array.unwrap().to_gc_managed()))))*/
}

pub fn newarray<'gc_life, 'l>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life,'l>, a_type: Atype) {
    let count = int_state.pop_current_operand_stack(Some(RuntimeType::IntType)).unwrap_int();
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
        todo!("check array length");
    }
    let new_array = todo!()/*match JavaValue::new_vec(jvm, int_state, count as usize, default_value(type_.clone()).to_jv(), type_) {
        Ok(arr) => arr,
        Err(WasException {}) => return,
    }*/;
    int_state.push_current_operand_stack(todo!()/*JavaValue::Object(new_array)*/);
}

pub fn multi_a_new_array<'gc_life, 'l>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life,'l>, dims: u8, type_: &CPDType) {
    if let Err(_) = check_resolved_class(jvm, int_state, type_.clone()) {
        return;
    };
    let mut dimensions = vec![];
    let mut unwrapped_type: CPDType = type_.clone();
    for _ in 0..dims {
        dimensions.push(int_state.current_frame_mut().pop(Some(RuntimeType::IntType)).unwrap_int());
    }
    for _ in 1..dims {
        unwrapped_type = unwrapped_type.unwrap_array_type().clone()
    }
    let mut current = JavaValue::null();
    let mut current_type = unwrapped_type;
    for len in dimensions {
        let next_type = CPDType::array(current_type);
        let mut new_vec = vec![];
        for _ in 0..len {
            new_vec.push(current.deep_clone(jvm))
        }
        drop(current);
        current = todo!()/*JavaValue::Object(
            jvm.allocate_object(todo!()/*Object::Array(match ArrayObject::new_array(jvm, int_state, new_vec, next_type.clone(), jvm.thread_state.new_monitor("monitor for a multi dimensional array".to_string())) {
                Ok(arr) => arr,
                Err(WasException {}) => return,
            })*/).to_gc_managed()
                .into(),
        )*/;
        current_type = next_type;
    }
    int_state.push_current_operand_stack(current);
}