use std::borrow::Borrow;
use std::sync::Arc;

use rust_jvm_common::classnames::ClassName;

use crate::{InterpreterStateGuard, JVMState};
use crate::interpreter_util::check_inited_class;
use crate::java::util::properties::Properties;
use crate::java_values::{JavaValue, Object};

pub struct System {
    normal_object: Arc<Object>
}

impl System {
    pub fn props(jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> Properties {
        let system_class = check_inited_class(jvm, int_state, &ClassName::system().into(), jvm.bootstrap_loader.clone());
        let prop_jv = system_class.static_vars().borrow().get("props").unwrap().clone();
        prop_jv.cast_properties()
    }

    as_object_or_java_value!();
}
