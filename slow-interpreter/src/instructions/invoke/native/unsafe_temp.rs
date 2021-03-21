#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use crate::interpreter::WasException;
//all of these functions should be implemented in libjvm
use crate::java_values::JavaValue;
use crate::jvm_state::ClassStatus;
use crate::JVMState;

pub fn shouldBeInitialized(jvm: &JVMState, args: &mut Vec<JavaValue>) -> Result<JavaValue, WasException> {
    let class_to_check = args[1].cast_class().expect("todo").as_runtime_class(jvm);
    let is_init = matches!(class_to_check.status(),ClassStatus::INITIALIZED);
    Ok(JavaValue::Boolean(is_init as u8))
}
