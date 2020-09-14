#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

//all of these functions should be implemented in libjvm
use crate::java_values::JavaValue;
use crate::JVMState;

pub fn shouldBeInitialized(state: &JVMState, args: &mut Vec<JavaValue>) -> Option<JavaValue> {
    let class_name_to_check = args[1].cast_class().as_type();
    JavaValue::Boolean(state.classes.initialized_classes.read().unwrap().get(&class_name_to_check).is_some() as u8).into()
}
