use std::sync::Arc;

use rust_jvm_common::classnames::ClassName;

use crate::{InterpreterStateGuard, JVMState};
use crate::class_loading::assert_inited_or_initing_class;
use crate::java::util::properties::Properties;
use crate::java_values::{JavaValue, Object};

pub struct System {
    normal_object: Arc<Object>
}

impl System {
    pub fn props(jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> Properties {
        let system_class = assert_inited_or_initing_class(jvm, ClassName::system().into());
        let prop_jv = system_class.static_vars().get("props").unwrap().clone();
        prop_jv.cast_properties()
    }

    as_object_or_java_value!();
}
