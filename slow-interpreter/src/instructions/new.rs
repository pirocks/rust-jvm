use std::rc::Rc;
use rust_jvm_common::classfile::{ConstantKind, Atype, MultiNewArray};
use crate::interpreter_util::{push_new_object, check_inited_class};
use rust_jvm_common::classnames::ClassName;
use std::sync::Arc;
use std::cell::RefCell;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use crate::java_values::{JavaValue, Object, ArrayObject, default_value};
use crate::{JVMState, StackEntry};

pub fn new(state: & JVMState, current_frame: & StackEntry, cp: usize) -> () {
    let loader_arc = &current_frame.class_pointer.loader;
    let constant_pool = &current_frame.class_pointer.classfile.constant_pool;
    let class_name_index = match &constant_pool[cp as usize].kind {
        ConstantKind::Class(c) => c.name_index,
        _ => panic!()
    };
    let target_class_name = ClassName::Str(constant_pool[class_name_index as usize].extract_string_from_utf8());
//    dbg!(&target_class_name);
    let target_classfile = check_inited_class(
        state,
        &target_class_name,
        loader_arc.clone()
    );
    push_new_object(state,current_frame, &target_classfile);
}


pub fn anewarray(state: & JVMState, current_frame: &StackEntry, cp: u16) -> () {
    let len = match current_frame.pop() {
        JavaValue::Int(i) => i,
        _ => panic!()
    };
    let constant_pool = &current_frame.class_pointer.classfile.constant_pool;
    let cp_entry = &constant_pool[cp as usize].kind;
    match cp_entry {
        ConstantKind::Class(c) => {
            let name = ClassName::Str(constant_pool[c.name_index as usize].extract_string_from_utf8());
            a_new_array_from_name(state, current_frame, len, &name)
        }
        _ => {
            dbg!(cp_entry);
            panic!()
        }
    }
}

pub fn a_new_array_from_name(state: & JVMState, current_frame: &StackEntry, len: i32, name: &ClassName) -> () {
    check_inited_class(
        state,
        &name,
        current_frame.class_pointer.loader.clone()
    );
    let t = PTypeView::Ref(ReferenceTypeView::Class(name.clone()));
    current_frame.push(JavaValue::Object(Some(JavaValue::new_vec(len as usize, JavaValue::Object(None), t).unwrap()).into()))
}


pub fn newarray(current_frame: & StackEntry, a_type: Atype) -> () {
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
    current_frame.push(JavaValue::Object(JavaValue::new_vec(count as usize, default_value(type_.clone()), type_)));
}


pub fn multi_a_new_array(state: & JVMState, current_frame: & StackEntry, cp: MultiNewArray) -> () {
    let dims = cp.dims;
    let temp = current_frame.class_pointer.class_view.constant_pool_view(cp.index as usize);
    let type_ = temp.unwrap_class();
    let name = type_.class_name();
    dbg!(&name);

   check_inited_class(state, &name.unwrap_arrays_to_name().unwrap(),  current_frame.class_pointer.loader.clone());
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
            new_vec.push(current.deep_clone())
        }
        current = JavaValue::Object(Arc::new(Object::Array(ArrayObject { elems: RefCell::new(new_vec), elem_type: next_type.clone() })).into());
        current_type = next_type;
    }
    current_frame.push(current);
}
