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
    pub initialized_classes : RwLock<HashMap<ClassName,Arc<RuntimeClass>>>,
    pub string_internment : RefCell<HashMap<String,Arc<Object>>>,
    pub class_object_pool : RefCell<HashMap<Arc<RuntimeClass>,Arc<Object>>>,//todo needs to be used for all instances of getClass
    pub jni: LibJavaLoading
}

#[derive(Debug)]
pub struct CallStackEntry {
    pub last_call_stack : Option<Rc<CallStackEntry>>,
    pub class_pointer: Arc<RuntimeClass>,
    pub method_i: CPIndex,

    pub local_vars: RefCell<Vec<JavaValue>>,
    pub operand_stack: RefCell<Vec<JavaValue>>,
    pub pc: RefCell<usize>,
    //the pc_offset is set by every instruction. branch instructions and others may us it to jump
    pub pc_offset: RefCell<isize>,
}


#[derive(Debug)]
pub struct LibJavaLoading {
    pub lib: Library,
    pub registered_natives: RefCell<HashMap<Arc<RuntimeClass>, RefCell<HashMap<CPIndex, unsafe extern fn()>>>>
}

