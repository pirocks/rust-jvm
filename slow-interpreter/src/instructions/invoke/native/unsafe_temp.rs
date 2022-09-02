#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use runtime_class_stuff::ClassStatus;
use another_jit_vm_ir::WasException;
use crate::interpreter_state::InterpreterStateGuard;
//all of these functions should be implemented in libjvm
use crate::{JVMState, NewJavaValue, NewJavaValueHandle, pushable_frame_todo};
use crate::utils::unwrap_or_npe;

pub fn shouldBeInitialized<'l, 'gc>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc,'l>, args: Vec<NewJavaValue<'gc,'_>>) -> Result<NewJavaValueHandle<'gc>, WasException> {
    let class_to_check = unwrap_or_npe(jvm, pushable_frame_todo()/*int_state*/, args[1].cast_class())?.as_runtime_class(jvm);
    let is_init = matches!(class_to_check.status(), ClassStatus::INITIALIZED);
    Ok(NewJavaValueHandle::Boolean(is_init as u8))
}