use runtime_common::StackEntry;
use std::rc::Rc;
use runtime_common::java_values::JavaValue;
use rust_jvm_common::classnames::class_name;

pub fn astore(current_frame: &Rc<StackEntry>, n: usize) -> () {
    let object_ref = current_frame.pop();
    match object_ref.clone() {
        JavaValue::Object(_) => {}
        _ => {
            dbg!(&object_ref);
            panic!()
        }
    }
    let classfile = &current_frame.class_pointer.classfile;
    dbg!(class_name(classfile).get_referred_name());
    dbg!(classfile.methods[current_frame.method_i as usize].method_name(classfile));
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
