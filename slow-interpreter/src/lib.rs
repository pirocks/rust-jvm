extern crate log;
extern crate simple_logger;

use std::sync::{Arc, RwLock};
use rust_jvm_common::classfile::CPIndex;
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::loading::Loader;
use std::error::Error;
use rust_jvm_common::utils::method_name;
use rust_jvm_common::utils::extract_string_from_utf8;
use classfile_parser::types::parse_method_descriptor;
use rust_jvm_common::unified_types::ParsedType;
use rust_jvm_common::unified_types::ArrayType;
use rust_jvm_common::unified_types::ClassWithLoader;
use crate::runtime_class::prepare_class;
use crate::interpreter_util::run_function;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use runtime_common::runtime_class::RuntimeClass;
use runtime_common::java_values::{JavaValue, VecPointer};
use rust_jni::LibJavaLoading;

pub struct InterpreterState {
//    pub call_stack: Vec<CallStackEntry>,
    pub terminate: bool,
    pub throw: bool,
    pub function_return: bool,
    pub bootstrap_loader: Arc<dyn Loader + Send + Sync>,
    pub initialized_classes : RwLock<HashMap<ClassName,Arc<RuntimeClass>>>
}

//impl InterpreterState {
//    fn current_frame_mut(&mut self) -> &mut CallStackEntry {
//        self.call_stack.last_mut().unwrap()
//    }
//    fn current_frame(&self) -> &CallStackEntry {
//        self.call_stack.last().unwrap()
//    }
//}

#[derive(Debug)]
pub struct CallStackEntry {
    pub last_call_stack : Option<Rc<CallStackEntry>>,
    pub class_pointer: Arc<RuntimeClass>,
    pub method_i: CPIndex,

    pub local_vars: Vec<JavaValue>,
    pub operand_stack: RefCell<Vec<JavaValue>>,
    pub pc: RefCell<usize>,
    //the pc_offset is set by every instruction. branch instructions and others may us it to jump
    pub pc_offset: RefCell<isize>,
}

//impl Clone for CallStackEntry{
//    fn clone(&self) -> Self {
//        CallStackEntry {
//            last_call_stack: self.last_call_stack.clone(),
//            class_pointer: self.class_pointer.clone(),
//            method_i: self.method_i,
//            local_vars: self.local_vars.clone(),
//            operand_stack: self.operand_stack.clone(),
//            pc: self.pc,
//            pc_offset: self.pc_offset
//        }
//    }
//}

pub fn run(
    main_class_name: &ClassName,
    bl: Arc<dyn Loader + Send + Sync>,
    args: Vec<String>,
    jni: LibJavaLoading
) -> Result<(), Box<dyn Error>> {
    let main = bl.clone().load_class(bl.clone(),main_class_name,bl.clone())?;
    let main_class = prepare_class(main.clone(), bl.clone());
    let (main_i, _main_method) = &main.methods.iter().enumerate().find(|(_, method)| {
        let name = method_name(&main, &method);
        if name == "main" {
            let descriptor_string = extract_string_from_utf8(&main.constant_pool[method.descriptor_index as usize]);
            let descriptor = parse_method_descriptor(&bl, descriptor_string.as_str()).unwrap();
            let string_name = ClassName::Str("java/lang/String".to_string());
            let string_class = ParsedType::Class(ClassWithLoader { class_name: string_name, loader: bl.clone() });
            let string_array = ParsedType::ArrayReferenceType(ArrayType { sub_type: Box::new(string_class) });
            descriptor.parameter_types.len() == 1 &&
                descriptor.return_type == ParsedType::VoidType &&
                descriptor.parameter_types.iter().zip(vec![string_array]).all(|(a, b)| a == &b)
        } else {
            false
        }
    }).unwrap();
    let mut state = InterpreterState {
//        call_stack: vec![CallStackEntry {
//            class_pointer: Arc::new(main_class),
//            method_i: *main_i as u16,
//            todo is that vec access safe, or does it not heap allocate?
//            local_vars: vec![JavaValue::Array(Some(VecPointer { object: unsafe {&vec![]} }))],//todo handle parameters
//            operand_stack: vec![],
//            pc: 0,
//            pc_offset: 0,
//        }],
        terminate: false,
        throw: false,
        function_return: false,
        bootstrap_loader: bl.clone(),
        initialized_classes: RwLock::new(HashMap::new())
    };
    let stack = CallStackEntry {
        last_call_stack: None,
        class_pointer: Arc::new(main_class),
            method_i: *main_i as u16,
//            todo is that vec access safe, or does it not heap allocate?
            local_vars: vec![JavaValue::Array(Some(VecPointer { object: Arc::new(vec![]) }))],//todo handle parameters
            operand_stack: vec![].into(),
            pc: RefCell::new(0),
            pc_offset: 0.into(),
        };
    run_function(&mut state,Rc::new(stack),&jni);
    if state.throw || state.terminate {
        unimplemented!()
    }
    Result::Ok(())
}

pub mod instructions;
pub mod interpreter_util;
pub mod runtime_class;
pub mod native;