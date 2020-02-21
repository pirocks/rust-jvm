use runtime_common::StackEntry;
use std::rc::Rc;
use runtime_common::java_values::JavaValue;
use rust_jvm_common::unified_types::PType;
use rust_jvm_common::view::ptype_view::PTypeView;

pub fn astore(current_frame: &Rc<StackEntry>, n: usize) -> () {
    let object_ref = current_frame.pop();
    match object_ref.clone() {
        JavaValue::Object(_) => {}
        _ => {
            dbg!(&object_ref);
            panic!()
        }
    }
    current_frame.local_vars.borrow_mut()[n] = object_ref;
}

pub fn lstore(current_frame: &Rc<StackEntry>, n: usize) -> () {
    let val = current_frame.pop();
    match val.clone() {
        JavaValue::Long(_) => {}
        _ => {
            dbg!(&val);
            panic!()
        }
    }
    current_frame.local_vars.borrow_mut()[n] = val;
}

pub fn dstore(current_frame: &Rc<StackEntry>, n: usize) -> () {
    let object_ref = current_frame.pop();
    match object_ref.clone() {
        JavaValue::Double(_) => {}
        _ => {
            dbg!(&object_ref);
            panic!()
        }
    }
    current_frame.local_vars.borrow_mut()[n] = object_ref;
}

pub fn castore(current_frame: &Rc<StackEntry>) -> () {
    let val = current_frame.pop().unwrap_int();
    let index = current_frame.pop().unwrap_int();
    let arrar_ref_o = current_frame.pop().unwrap_object().unwrap();
    let array_ref = &mut arrar_ref_o.unwrap_array().elems.borrow_mut();
    let char_ = val as u8 as char;
    array_ref[index as usize] = JavaValue::Char(char_);
}

pub fn bastore(current_frame: &Rc<StackEntry>) -> () {
    let val = current_frame.pop().unwrap_int();
    let index = current_frame.pop().unwrap_int();
    let array_ref_o = current_frame.pop().unwrap_object().unwrap();
    assert!(array_ref_o.unwrap_array().elem_type == PTypeView::ByteType || array_ref_o.unwrap_array().elem_type == PTypeView::BooleanType);
    let array_ref = &mut array_ref_o.unwrap_array().elems.borrow_mut();
    array_ref[index as usize] = JavaValue::Byte(val as i8);
}


pub fn iastore(current_frame: &Rc<StackEntry>) -> () {
    let val = current_frame.pop().unwrap_int();
    let index = current_frame.pop().unwrap_int();
    let arrar_ref_o = current_frame.pop().unwrap_object().unwrap();
    let array_ref = &mut arrar_ref_o.unwrap_array().elems.borrow_mut();
    let int_ = val;
    array_ref[index as usize] = JavaValue::Int(int_);
}


pub fn aastore(current_frame: &Rc<StackEntry>) -> () {
    let val = current_frame.pop();
    let index = current_frame.pop().unwrap_int();
    let arrary_ref_o = current_frame.pop().unwrap_object().unwrap();
    let mut array_ref = arrary_ref_o.unwrap_array().elems.borrow_mut();
    match val {
        JavaValue::Object(_) => {}
        _ => panic!(),
    }
    array_ref[index as usize] = val.clone();
}
