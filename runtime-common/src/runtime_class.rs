use std::sync::Arc;
use std::collections::HashMap;
use std::fmt::{Formatter, Debug, Error};
use crate::java_values::JavaValue;
use rust_jvm_common::classfile::Classfile;
use rust_jvm_common::loading::Loader;
use std::hash::{Hash, Hasher};
use std::cell::RefCell;

pub struct RuntimeClass {
    pub classfile: Arc<Classfile>,
    pub loader: Arc<dyn Loader + Send + Sync>,
    pub static_vars: RefCell<HashMap<String, JavaValue>>,
}

impl Debug for RuntimeClass {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{:?}:{:?}", self.classfile, self.static_vars)
    }
}

impl Hash for RuntimeClass {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.classfile.hash(state);
        //todo add loader to hash
    }
}


impl PartialEq for RuntimeClass {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.loader, &other.loader) && self.classfile == other.classfile && self.static_vars == other.static_vars
    }
}

impl Eq for RuntimeClass {}