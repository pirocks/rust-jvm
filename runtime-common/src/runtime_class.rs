use std::sync::Arc;
use std::collections::HashMap;
use std::fmt::{Formatter, Debug, Error};
use crate::java_values::JavaValue;
use rust_jvm_common::classfile::Classfile;
use rust_jvm_common::loading::Loader;

pub struct RuntimeClass {
    pub classfile: Arc<Classfile>,
    pub loader: Arc<dyn Loader + Send + Sync>,
    pub static_vars: HashMap<String, JavaValue>,
}

impl Debug for RuntimeClass{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f,"{:?}:{:?}",self.classfile,self.static_vars)
    }
}

