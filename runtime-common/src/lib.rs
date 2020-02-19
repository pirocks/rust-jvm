extern crate log;
extern crate simple_logger;
extern crate libloading;

use std::sync::{RwLock, Arc};
use std::cell::RefCell;
use std::collections::HashMap;
use rust_jvm_common::classnames::{ClassName, class_name};
use crate::runtime_class::RuntimeClass;
use crate::java_values::{Object, JavaValue};
use rust_jvm_common::classfile::CPIndex;
use libloading::Library;
use std::rc::Rc;
use rust_jvm_common::unified_types::PType;
use rust_jvm_common::string_pool::StringPool;

pub mod java_values;
pub mod runtime_class;

pub struct InterpreterState {
    pub terminate: bool,
    pub throw: Option<Arc<Object>>,
    pub function_return: bool,
    pub bootstrap_loader: LoaderArc,
    pub initialized_classes: RwLock<HashMap<ClassName, Arc<RuntimeClass>>>,
    pub string_internment: RefCell<HashMap<String, Arc<Object>>>,
    pub class_object_pool: RefCell<HashMap<Arc<RuntimeClass>, Arc<Object>>>,
    pub array_object_pool: RefCell<HashMap<PType, Arc<Object>>>,
    //todo needs to be used for all instances of getClass
    pub jni: LibJavaLoading,
    pub string_pool: StringPool,//todo this should really be in some sort of parser/jvm state
}

#[derive(Debug)]
pub struct StackEntry {
    pub last_call_stack: Option<Rc<StackEntry>>,
    pub class_pointer: Arc<RuntimeClass>,
    pub method_i: CPIndex,

    pub local_vars: RefCell<Vec<JavaValue>>,
    pub operand_stack: RefCell<Vec<JavaValue>>,
    pub pc: RefCell<usize>,
    //the pc_offset is set by every instruction. branch instructions and others may us it to jump
    pub pc_offset: RefCell<isize>,
}

impl StackEntry {
    pub fn pop(&self) -> JavaValue {
        self.operand_stack.borrow_mut().pop().unwrap_or_else(|| {
            let classfile = &self.class_pointer.classfile;
            let method = &classfile.methods[self.method_i as usize];
            dbg!(&method.method_name(&classfile));
            dbg!(&method.code_attribute().unwrap().code);
            dbg!(&self.pc);
            panic!()
        })
    }
    pub fn push(&self, j: JavaValue) {
        self.operand_stack.borrow_mut().push(j)
    }

    pub fn depth(&self) -> usize {
        match &self.last_call_stack {
            None => 0,
            Some(last) => last.depth() + 1,
        }
    }

    pub fn print_stack_trace(&self) {
        let class_file = &self.class_pointer.classfile;
        let method = &class_file.methods[self.method_i as usize];
        println!("{} {} {} {}",
                         &class_name(class_file).get_referred_name(),
                         method.method_name(class_file),
                         method.descriptor_str(class_file),
                         self.depth());
        match &self.last_call_stack{
            None => {},
            Some(last) => last.print_stack_trace(),
        }
    }
}


#[derive(Debug)]
pub struct LibJavaLoading {
    pub lib: Library,
    pub registered_natives: RefCell<HashMap<Arc<RuntimeClass>, RefCell<HashMap<u16, unsafe extern fn()>>>>,
}

