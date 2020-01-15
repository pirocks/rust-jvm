use std::sync::Arc;
use crate::runtime_class::RuntimeClass;
use crate::java_values::JavaValue;
use rust_jvm_common::classfile::CPIndex;
use rust_jvm_common::utils::code_attribute;
use classfile_parser::code::parse_instruction;
use classfile_parser::code::CodeParserContext;
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

pub struct InterpreterState {
    //    pub method_area : //todo
    pub call_stack: Vec<CallStackEntry>,
    pub terminate: bool,
    pub throw: bool,
    pub function_return: bool,
}

impl InterpreterState {
    fn current_frame_mut(&mut self) -> &mut CallStackEntry {
        self.call_stack.last_mut().unwrap()
    }
    fn current_frame(&self) -> &CallStackEntry {
        self.call_stack.last().unwrap()
    }
}

pub struct CallStackEntry {
    pub class_pointer: Arc<RuntimeClass>,
    pub method_i: CPIndex,

    pub local_vars: Vec<JavaValue>,
    pub operand_stack: Vec<JavaValue>,
    pub pc: usize,
    //the pc_offset is set by every instruction. branch instructions and others may us it to jump
    pub pc_offset: isize,
}


pub fn run(main_class_name: &ClassName, loader: Arc<Loader + Send + Sync>) -> Result<(), Box<dyn Error>> {
    let object = loader.load_class(&ClassName::Str("java/lang/Object".to_string()))?;
    let main = loader.load_class(main_class_name)?;//should load superclasses so Object load is redundant
    let main_class = prepare_class(main,loader);
    let (main_i,main_method) = main.methods.iter().enumerate().find(|(_,method)|{
        let name = method_name(&main, &method);
        if name == "main" {
            let descriptor_string = extract_string_from_utf8(&main.constant_pool[method.descriptor_index as usize]);
            let descriptor = parse_method_descriptor(loader, descriptor_string.as_str()).unwrap();
            descriptor.return_type == ParsedType::VoidType &&
                descriptor.parameter_types == ParsedType::ArrayReferenceType(ArrayType { sub_type: Box::new(ParsedType::Class(ClassWithLoader { class_name: main_class_name.clone(), loader: loader.clone(), }))})
        }
        false
    }).unwrap();
    let mut state = InterpreterState {
        call_stack: vec![CallStackEntry {
            class_pointer,
            method_i: main_i as u16,
            local_vars: vec![JavaValue::Object(None)],//todo handle parameters
            operand_stack: vec![],
            pc: 0,
            pc_offset: 0
        }],
        terminate: false,
        throw: false,
        function_return: false
    };
    run_function(&mut state)
}

pub mod instructions;
pub mod interpreter_util;
pub mod runtime_class;
pub mod java_values;