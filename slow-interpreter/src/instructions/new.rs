use std::sync::Arc;

use classfile_view::view::constant_info_view::ConstantInfoView;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use rust_jvm_common::classfile::{Atype, MultiNewArray};
use rust_jvm_common::classnames::ClassName;

use crate::{InterpreterStateGuard, JVMState};
use crate::interpreter_util::{check_inited_class, push_new_object};
use crate::java_values::{ArrayObject, default_value, JavaValue, Object};

pub fn new(jvm: &JVMState, int_state: &mut InterpreterStateGuard, cp: usize) {
    let loader_arc = &int_state.current_frame_mut().class_pointer().loader(jvm);
    let view = &int_state.current_frame_mut().class_pointer().view();
    let target_class_name = &view.constant_pool_view(cp as usize).unwrap_class().class_name().unwrap_name();
    let target_classfile = check_inited_class(jvm, int_state, &target_class_name.clone().into(), loader_arc.clone());
    push_new_object(jvm, int_state, &target_classfile, None);
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
            let name = ClassName::Str(c.class_name().unwrap_name().get_referred_name().to_string());//todo fix this jankyness
            a_new_array_from_name(state, int_state, len, &name)
        }
        _ => {
            dbg!(cp_entry);
            panic!()
        }
    }
}

pub fn a_new_array_from_name(jvm: &JVMState, int_state: &mut InterpreterStateGuard, len: i32, name: &ClassName) {
    check_inited_class(
        jvm,
        int_state,
        &name.clone().into(),
        int_state.current_loader(jvm).clone(),
    );
    let t = PTypeView::Ref(ReferenceTypeView::Class(name.clone()));
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

    check_inited_class(jvm, int_state, &PTypeView::Ref(type_.clone()), int_state.current_loader(jvm).clone());
    //todo need to start doing this at some point
    let mut dimensions = vec![];
    let mut unwrapped_type = type_;
    for _ in 0..dims {
        dimensions.push(int_state.current_frame_mut().pop().unwrap_int());
        unwrapped_type = unwrapped_type.unwrap_array().unwrap_ref_type().clone()
    }
    let mut current = JavaValue::Object(None);
    let mut current_type = PTypeView::Ref(unwrapped_type);//todo fix this as a matter of urgency
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
            jvm.thread_state.new_monitor("monitor for a multi dimensional array".to_string()),
        ))).into());
        current_type = next_type;
    }
    int_state.push_current_operand_stack(current);
}
