use std::sync::Arc;

use rust_jvm_common::classnames::ClassName;

use crate::{InterpreterStateGuard, JVMState};
use crate::class_loading::assert_inited_or_initing_class;
use crate::java::util::properties::Properties;
use crate::java_values::{JavaValue, Object};

pub struct System<'gc_life> {
    normal_object: Arc<Object<'gc_life>>,
}

impl<'gc_life> System<'gc_life> {
    pub fn props<'l, 'k : 'l>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'k mut InterpreterStateGuard<'l, 'gc_life>) -> Properties<'gc_life> {
        let system_class = assert_inited_or_initing_class(jvm, ClassName::system().into());
        let prop_jv = system_class.static_vars().get("props").unwrap().clone();
        prop_jv.cast_properties()
    }

    as_object_or_java_value!();
}
