extern crate log;
extern crate simple_logger;
extern crate libloading;

use std::sync::{RwLock, Arc};
use std::cell::RefCell;
use rust_jvm_common::loading::Loader;
use std::collections::HashMap;
use rust_jvm_common::classnames::ClassName;
use crate::runtime_class::RuntimeClass;
use crate::java_values::{Object, JavaValue};
use rust_jvm_common::classfile::CPIndex;
use libloading::Library;
use std::rc::Rc;

pub mod java_values;
pub mod runtime_class;

pub struct InterpreterState {
    pub terminate: bool,
    pub throw: bool,
    pub function_return: bool,
    pub bootstrap_loader: Arc<dyn Loader + Send + Sync>,
    pub initialized_classes: RwLock<HashMap<ClassName, Arc<RuntimeClass>>>,
    pub string_internment: RefCell<HashMap<String, Arc<Object>>>,
    pub class_object_pool: RefCell<HashMap<Arc<RuntimeClass>, Arc<Object>>>,
    //todo needs to be used for all instances of getClass
    pub jni: LibJavaLoading,
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
        self.operand_stack.borrow_mut().pop().unwrap()
    }
    pub fn push(&self, j: JavaValue) {
        self.operand_stack.borrow_mut().push(j)
    }
}


#[derive(Debug)]
pub struct LibJavaLoading {
    pub lib: Library,
    pub registered_natives: RefCell<HashMap<Arc<RuntimeClass>, RefCell<HashMap<u16, unsafe extern fn()>>>>,
}

