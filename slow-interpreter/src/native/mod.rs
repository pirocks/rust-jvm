use crate::InterpreterState;
use crate::CallStackEntry;
use std::rc::Rc;
use std::sync::Arc;
use runtime_common::runtime_class::RuntimeClass;
use rust_jni::LibJavaLoading;
use rust_jvm_common::utils::{extract_string_from_utf8, method_name};
use classfile_parser::types::parse_method_descriptor;
use rust_jvm_common::classfile::ACC_STATIC;
use rust_jni::JNIContext;
use runtime_common::java_values::{JavaValue, Object};
use std::cell::RefCell;
use std::borrow::Borrow;


pub fn run_native_method(
    state: &InterpreterState,
    frame: Rc<CallStackEntry>,
    class: Arc<RuntimeClass>,
    method_i: usize,
    jni: &LibJavaLoading,
) {
    //todo only works for static void methods atm
    let classfile = &class.classfile;
    let method = &classfile.methods[method_i];
    assert!(method.access_flags & ACC_STATIC > 0);
    let descriptor_str = extract_string_from_utf8(&classfile.constant_pool[method.descriptor_index as usize]);
    let parsed = parse_method_descriptor(&class.loader, descriptor_str.as_str()).unwrap();
    let mut args = vec![];
//    dbg!(&frame.operand_stack);
    for _ in parsed.parameter_types {
        args.push(frame.operand_stack.borrow_mut().pop().unwrap());
    }
    args.reverse();
    if method_name(classfile, method) == "desiredAssertionStatus0".to_string() {//todo and descriptor matches and class matches
        frame.operand_stack.borrow_mut().push(JavaValue::Boolean(false))
    } else if method_name(classfile, method) == "arraycopy".to_string() {
//        public static void arraycopy​(Object src,
//        int srcPos,
//        Object dest,
//        int destPos,
//        int length)
        let src = unwrap_array(args[0].clone());
        let src_pos = unwrap_int(args[1].clone()) as usize;
        let dest = unwrap_array(args[2].clone());
        let dest_pos = unwrap_int(args[3].clone()) as usize;
        let length = unwrap_int(args[4].clone()) as usize;
        for i in 0..length {
            let borrowed: &RefCell<Vec<JavaValue>> = src.borrow();
            let temp = (borrowed.borrow())[src_pos + i].borrow().clone();
            dest.borrow_mut()[dest_pos + i] = temp;
        }
    }else if method_name(classfile, method) == "getPrimitiveClass".to_string() {
        let string_value = unwrap_array(unwrap_object(args[0].clone()).fields.borrow().get("value").unwrap().clone());
//        dbg!(string_value);
        let borrowed: &RefCell<Vec<JavaValue>> = string_value.borrow();
        if borrowed.borrow()[0] == JavaValue::Char('f') {
            unimplemented!()
        }//todo need to spell out float
        unimplemented!()
    } else {
        dbg!(method_name(classfile, method));
        match jni.call(class.clone(), method_i, args, parsed.return_type) {
            None => {}
            Some(_) => unimplemented!(),
        }
    }
}


fn unwrap_int(j: JavaValue) -> i32 {
    match j {
        JavaValue::Int(i) => {
            i
        }
        _ => panic!()
    }
}

fn unwrap_array(j: JavaValue) -> Arc<RefCell<Vec<JavaValue>>> {
    match j {
        JavaValue::Array(a) => {
            a.unwrap().object
        }
        _ => {
            dbg!(j);
            panic!()
        }
    }
}

fn unwrap_object(j: JavaValue) -> Arc<Object> {
    match j {
        JavaValue::Object(o) => {
            o.unwrap().object
        }
        _ => {
            dbg!(j);
            panic!()
        }
    }
}