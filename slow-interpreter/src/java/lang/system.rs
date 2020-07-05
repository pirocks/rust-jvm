use std::sync::Arc;
use crate::java::util::properties::Properties;
use crate::{JVMState, InterpreterStateGuard};
use crate::interpreter_util::check_inited_class;
use rust_jvm_common::classnames::ClassName;
use std::borrow::Borrow;
use crate::java_values::{Object, JavaValue};

pub struct System {
    normal_object: Arc<Object>
}

impl System {
    pub fn props<'l>(jvm: &'static JVMState, int_state: & mut InterpreterStateGuard) -> Properties{
        let system_class = check_inited_class(jvm,int_state,&ClassName::system().into(),jvm.bootstrap_loader.clone());
        let prop_jv = system_class.static_vars().borrow().get("props").unwrap().clone();
        prop_jv.cast_properties()
    }

    as_object_or_java_value!();
}
