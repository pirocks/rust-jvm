use jni_bindings::{JNIEnv, jbyte, jsize, jbyteArray, jarray};
use std::cell::RefCell;
use std::sync::Arc;
use crate::rust_jni::native_util::{to_object, from_object, get_state};
use std::ops::Deref;
use classfile_view::view::ptype_view::PTypeView;
use crate::java_values::{JavaValue, Object, ArrayObject};


pub unsafe extern "C" fn get_array_length(_env: *mut JNIEnv, array: jarray) -> jsize {
    let non_null_array: &Object = &from_object(array).unwrap();
    let len = match non_null_array {
        Object::Array(a) => {
            a.elems.borrow().len()
        }
        Object::Object(_o) => {
            unimplemented!()
        }
    };
    len as jsize
}


pub unsafe extern "C" fn get_byte_array_region(_env: *mut JNIEnv, array: jbyteArray, start: jsize, len: jsize, buf: *mut jbyte) {
    let non_null_array_obj = from_object(array).unwrap();
    let array_ref = non_null_array_obj.unwrap_array().elems.borrow();
    let array = array_ref.deref();
    for i in 0..len {
        let byte = array[(start + i) as usize].unwrap_int() as jbyte;
//        dbg!(byte as u8 as char);
        buf.offset(i as isize).write(byte)
    }
}


pub unsafe extern "C" fn new_byte_array(env: *mut JNIEnv, len: jsize) -> jbyteArray {
    let jvm = get_state(env);
    let mut the_vec = vec![];
    for _ in 0..len {
        the_vec.push(JavaValue::Byte(0))
    }
    to_object(Some(Arc::new(Object::Array(ArrayObject {
        elems: RefCell::new(the_vec),
        elem_type: PTypeView::ByteType,
        monitor: jvm.new_monitor("monitor for jni created byte array".to_string())
    }))))
}


pub unsafe extern "C" fn set_byte_array_region(_env: *mut JNIEnv, array: jbyteArray, start: jsize, len: jsize, buf: *const jbyte) {
    for i in 0..len {
        from_object(array)
            .unwrap()
            .unwrap_array()
            .elems
            .borrow_mut()
            .insert((start + i) as usize, JavaValue::Byte(buf.offset(i as isize).read() as i8));
    }
}