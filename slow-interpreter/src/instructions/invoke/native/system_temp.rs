use std::borrow::Borrow;

use crate::interpreter_state::InterpreterStateGuard;
use crate::java_values::JavaValue;
use crate::jvm_state::JVMState;
use crate::utils::throw_npe;

