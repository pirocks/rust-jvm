use rust_jvm_common::classfile::{ Atype, MultiNewArray};
use crate::interpreter_util::{push_new_object, check_inited_class};
use rust_jvm_common::classnames::ClassName;
use std::sync::Arc;
use std::cell::RefCell;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use crate::java_values::{JavaValue, Object, ArrayObject, default_value};
use crate::{JVMState, StackEntry};
use classfile_view::view::constant_info_view::ConstantInfoView;

pub fn new(jvm: &JVMState, current_frame: &StackEntry, cp: usize) -> () {
    let loader_arc = &current_frame.class_pointer.loader(jvm);
    let view = &current_frame.class_pointer.view();
    let class_name_index = &view.constant_pool_view(cp as usize).unwrap_class().class_name().unwrap_name();
    let target_class_name = ClassName::Str(view.constant_pool_view(class_name_index as usize).extract_string_from_utf8());
    let target_classfile = check_inited_class(
        jvm,
        &target_class_name,
        loader_arc.clone(),
    );
    push_new_object(jvm, current_frame, &target_classfile);
}


pub fn anewarray(state: &JVMState, current_frame: &StackEntry, cp: u16) -> () {
    let len = match current_frame.pop() {
        JavaValue::Int(i) => i,
        _ => panic!()
    };
    let view = &current_frame.class_pointer.view();
    let cp_entry = &view.constant_pool_view(cp as usize);
    match cp_entry {
        ConstantInfoView::Class(c) => {
            let name = ClassName::Str(c.class_name().unwrap_name().clone().get_referred_name().to_string());//todo fix this jankyness
            a_new_array_from_name(state, current_frame, len, &name)
        }
        _ => {
            dbg!(cp_entry);
            panic!()
        }
    }
}

pub fn a_new_array_from_name(jvm: &JVMState, current_frame: &StackEntry, len: i32, name: &ClassName) -> () {
    check_inited_class(
        jvm,
        &name,
        current_frame.class_pointer.loader(jvm).clone(),
    );
    let t = PTypeView::Ref(ReferenceTypeView::Class(name.clone()));
    current_frame.push(JavaValue::Object(Some(JavaValue::new_vec(jvm, len as usize, JavaValue::Object(None), t).unwrap()).into()))
}


pub fn newarray(jvm: &JVMState, current_frame: &StackEntry, a_type: Atype) -> () {
    let count = match current_frame.pop() {
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
    current_frame.push(JavaValue::Object(JavaValue::new_vec(jvm, count as usize, default_value(type_.clone()), type_)));
}


pub fn multi_a_new_array(jvm: &JVMState, current_frame: &StackEntry, cp: MultiNewArray) -> () {
    let dims = cp.dims;
    let temp = current_frame.class_pointer.view().constant_pool_view(cp.index as usize);
    let type_ = temp.unwrap_class();
    let name = type_.class_name();
    dbg!(&name);

    check_inited_class(jvm, &name.unwrap_arrays_to_name().unwrap(), current_frame.class_pointer.loader(jvm).clone());
    //todo need to start doing this at some point
    let mut dimensions = vec![];
    for _ in 0..dims {
        dimensions.push(current_frame.pop().unwrap_int());
    }
    let mut current = JavaValue::Object(None);
    let mut current_type = PTypeView::Ref(ReferenceTypeView::Class(ClassName::Str("sketch hack".to_string())));//todo fix this as a matter of urgency
    for len in dimensions {
        let next_type = PTypeView::Ref(ReferenceTypeView::Array(Box::new(current_type)));
        let mut new_vec = vec![];
        for _ in 0..len {
            new_vec.push(current.deep_clone(jvm))
        }
        current = JavaValue::Object(Arc::new(Object::Array(ArrayObject {
            elems: RefCell::new(new_vec),
            elem_type: next_type.clone(),
            monitor: jvm.new_monitor("monitor for a multi dimensional array".to_string()),
        })).into());
        current_type = next_type;
    }
    current_frame.push(current);
}
