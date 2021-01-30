use std::sync::Arc;

use classfile_parser::code::InstructionTypeNum::checkcast;
use classfile_view::view::constant_info_view::ConstantInfoView;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use rust_jvm_common::classfile::{Atype, MultiNewArray};

use crate::{InterpreterStateGuard, JVMState};
use crate::class_loading::{assert_inited_or_initing_class, check_initing_or_inited_class};
use crate::interpreter_util::push_new_object;
use crate::java_values::{ArrayObject, default_value, JavaValue, Object};

pub fn new(jvm: &JVMState, int_state: &mut InterpreterStateGuard, cp: usize) {
    let view = &int_state.current_frame_mut().class_pointer().view();
    let target_class_name = &view.constant_pool_view(cp as usize).unwrap_class().class_name().unwrap_name();
    // int_state.print_stack_trace();
    // dbg!(target_class_name);
    let target_classfile = check_initing_or_inited_class(jvm,
                                                         int_state, target_class_name.clone().into());
    push_new_object(jvm, int_state, &target_classfile);
}


pub fn anewarray(state: &JVMState, int_state: &mut InterpreterStateGuard, cp: u16) {
    let len = match int_state.current_frame_mut().pop() {
        JavaValue::Int(i) => i,
        _ => panic!()
    };
    let view = &int_state.current_frame_mut().class_pointer().view();
    let cp_entry = &view.constant_pool_view(cp as usize);
    match cp_entry {
        ConstantInfoView::Class(c) => {
            // int_state.print_stack_trace();
            //todo rename class_name
            let type_ = PTypeView::Ref(c.class_name());
            a_new_array_from_name(state, int_state, len, type_)
        }
        _ => {
            dbg!(cp_entry);
            panic!()
        }
    }
}

pub fn a_new_array_from_name(jvm: &JVMState, int_state: &mut InterpreterStateGuard, len: i32, t: PTypeView) {
    // if let Some(name) = t.unwrap_type_to_name(){
    check_initing_or_inited_class(
        jvm,
        int_state,
        t.clone(),
    );
    // }
    let new_array = JavaValue::new_vec(jvm, int_state, len as usize, JavaValue::Object(None), t);
    int_state.push_current_operand_stack(JavaValue::Object(Some(new_array.unwrap())))
}


pub fn newarray(jvm: &JVMState, int_state: &mut InterpreterStateGuard, a_type: Atype) {
    let count = match int_state.pop_current_operand_stack() {
        JavaValue::Int(i) => { i }
        _ => panic!()
    };
    let type_ = match a_type {
        Atype::TChar => {
            PTypeView::CharType
        }
        Atype::TInt => {
            PTypeView::IntType
        }
        Atype::TByte => {
            PTypeView::ByteType
        }
        Atype::TBoolean => {
            PTypeView::BooleanType
        }
        Atype::TShort => {
            PTypeView::ShortType
        }
        Atype::TLong => {
            PTypeView::LongType
        }
        Atype::TDouble => {
            PTypeView::DoubleType
        }
        Atype::TFloat => {
            PTypeView::FloatType
        }
    };
    let new_array = JavaValue::new_vec(jvm, int_state, count as usize, default_value(type_.clone()), type_);
    int_state.push_current_operand_stack(JavaValue::Object(new_array));
}


pub fn multi_a_new_array(jvm: &JVMState, int_state: &mut InterpreterStateGuard, cp: MultiNewArray) {
    let dims = cp.dims;
    let temp = int_state.current_frame_mut().class_pointer().view().constant_pool_view(cp.index as usize);
    let type_ = temp.unwrap_class().class_name();

    check_initing_or_inited_class(jvm, int_state, PTypeView::Ref(type_.clone()));
    //todo need to start doing this at some point
    let mut dimensions = vec![];
    // dbg!(&type_);
    let mut unwrapped_type: PTypeView = PTypeView::Ref(type_);
    // dbg!(dims);
    for _ in 0..dims {
        dimensions.push(int_state.current_frame_mut().pop().unwrap_int());
    }
    for _ in 1..dims {
        unwrapped_type = unwrapped_type.unwrap_array_type()
    }
    let mut current = JavaValue::Object(None);
    let mut current_type = unwrapped_type;//todo fix this as a matter of urgency
    for len in dimensions {
        let next_type = PTypeView::Ref(ReferenceTypeView::Array(Box::new(current_type)));
        let mut new_vec = vec![];
        for _ in 0..len {
            new_vec.push(current.deep_clone(jvm))
        }
        current = JavaValue::Object(Arc::new(Object::Array(ArrayObject::new_array(
            jvm,
            int_state,
            new_vec,
            next_type.clone(),
            jvm.thread_state.new_monitor("monitor for a multi dimensional array".to_string())
        ))).into());
        current_type = next_type;
    }
    // dbg!(&current);
    int_state.push_current_operand_stack(current);
}
