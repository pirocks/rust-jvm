use std::sync::Arc;
use std::collections::HashMap;
use std::fmt::{Formatter, Debug, Error};
use crate::java_values::JavaValue;
use rust_jvm_common::classfile::Classfile;
use std::hash::{Hash, Hasher};
use std::cell::RefCell;
use rust_jvm_common::loading::LoaderArc;
use rust_jvm_common::view::ClassView;

pub struct RuntimeClass {
    pub classfile: Arc<Classfile>,
    pub class_view: ClassView,
    pub loader: LoaderArc,
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
        self.loader.name().to_string().hash(state)
    }
}


impl PartialEq for RuntimeClass {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.loader, &other.loader) && self.classfile == other.classfile && self.static_vars == other.static_vars
    }
}

impl Eq for RuntimeClass {}