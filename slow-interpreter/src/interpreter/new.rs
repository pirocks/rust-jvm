use rust_jvm_common::classfile::Atype;
use rust_jvm_common::compressed_classfile::{CPDType};
use rust_jvm_common::compressed_classfile::names::CClassName;
use rust_jvm_common::runtime_type::RuntimeType;

use crate::{AllocatedHandle, check_initing_or_inited_class, JVMState, NewJavaValueHandle};
use crate::class_loading::{check_resolved_class};
use crate::interpreter::{PostInstructionAction, WasException};
use crate::interpreter::real_interpreter_state::{InterpreterJavaValue, RealInterpreterStateGuard};
use crate::interpreter_util::new_object;

pub fn new<'gc, 'k, 'l>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>, classname: CClassName)-> PostInstructionAction<'gc>  {
    let target_classfile = match check_initing_or_inited_class(jvm, int_state.inner(), classname.into()) {
        Ok(x) => x,
        Err(WasException {}) => {
            int_state.inner().debug_print_stack_trace(jvm);
            // int_state.throw().unwrap().lookup_field(jvm, FieldName::field_detailMessage());
            return PostInstructionAction::Exception { exception: WasException{} };
        }
    };
    let obj = new_object(jvm, int_state.inner(), &target_classfile);
    int_state.current_frame_mut().push(NewJavaValueHandle::Object(AllocatedHandle::NormalObject(obj)).to_interpreter_jv());
    PostInstructionAction::Next {}
}

pub fn anewarray<'gc, 'k, 'l>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>, cpdtype: &CPDType) -> PostInstructionAction<'gc> {
    let len = match int_state.current_frame_mut().pop(RuntimeType::IntType) {
        InterpreterJavaValue::Int(i) => i,
        _ => panic!(),
    };
    let type_ = cpdtype.clone();
    if let Err(WasException {}) = a_new_array_from_name(jvm, int_state, len, type_) {
        return PostInstructionAction::Exception { exception: WasException {} };
    }
    PostInstructionAction::Next {}
}

pub fn a_new_array_from_name<'gc, 'k, 'l>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>, len: i32, elem_type: CPDType) -> Result<(), WasException> {
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
        todo!("check array length");
    }
    match a_new_array_from_name(jvm,int_state,count,type_) {
        Ok(arr) => PostInstructionAction::Next {},
        Err(WasException {}) => PostInstructionAction::Exception { exception: WasException{} },
    }
}

// pub fn multi_a_new_array(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life,'l>, dims: u8, type_: &CPDType) {
//     if let Err(_) = check_resolved_class(jvm, int_state, type_.clone()) {
//         return;
//     };
//     let mut dimensions = vec![];
//     let mut unwrapped_type: CPDType = type_.clone();
//     for _ in 0..dims {
//         dimensions.push(int_state.current_frame_mut().pop(Some(RuntimeType::IntType)).unwrap_int());
//     }
//     for _ in 1..dims {
//         unwrapped_type = unwrapped_type.unwrap_array_type().clone()
//     }
//     let mut current = JavaValue::null();
//     let mut current_type = unwrapped_type;
//     for len in dimensions {
//         let next_type = CPDType::Ref(CPRefType::Array(box current_type));
//         let mut new_vec = vec![];
//         for _ in 0..len {
//             new_vec.push(current.deep_clone(jvm))
//         }
//         drop(current);
//         current = JavaValue::Object(
//             jvm.allocate_object(Object::Array(match ArrayObject::new_array(jvm, int_state, new_vec, next_type.clone(), jvm.thread_state.new_monitor("monitor for a multi dimensional array".to_string())) {
//                 Ok(arr) => arr,
//                 Err(WasException {}) => return,
//             }))
//                 .into(),
//         );
//         current_type = next_type;
//     }
//     int_state.push_current_operand_stack(current);
// }
