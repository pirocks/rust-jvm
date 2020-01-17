use crate::InterpreterState;
use crate::CallStackEntry;
use std::rc::Rc;
use std::sync::Arc;
use runtime_common::runtime_class::RuntimeClass;
use rust_jni::LibJavaLoading;
use rust_jvm_common::utils::extract_string_from_utf8;
use classfile_parser::types::parse_method_descriptor;
use rust_jvm_common::classfile::ACC_STATIC;
use rust_jni::JNIContext;


pub fn run_native_method(
    state:&InterpreterState,
    frame: Rc<CallStackEntry>,
    class: Arc<RuntimeClass>,
    method_i : usize,
    jni: &LibJavaLoading
) {
    //todo only works for static void methods atm
    let classfile = &class.classfile;
    let method = &classfile.methods[method_i];
    assert!(method.access_flags& ACC_STATIC > 0);
    let descriptor_str = extract_string_from_utf8(&classfile.constant_pool[method.descriptor_index as usize]);
    let parsed = parse_method_descriptor(&class.loader,descriptor_str.as_str()).unwrap();
    let mut args = vec![];
    for _ in parsed.parameter_types{
        args.push(frame.operand_stack.borrow_mut().pop().unwrap());
    }

    jni.call(class.clone(),method_i,args,parsed.return_type);

}