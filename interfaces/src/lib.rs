#![feature(box_syntax)]
#![feature(once_cell)]
#![feature(c_variadic)]
#![allow(unused_variables)]
#![allow(unreachable_code)]

use slow_interpreter::rust_jni::PerStackInterfaces;
use crate::jni_interface::initial_invoke_interface;
use crate::jni_interface::jmm::initial_jmm;
use crate::jni_interface::jni::initial_jni_interface;
use crate::jvmti_interface::initial_jvmti;

pub mod jni_interface;
pub mod jvmti_interface;
pub mod jmm_interface;
pub mod invoke_interface;

pub fn initial_per_stack_interfaces() -> PerStackInterfaces {
    PerStackInterfaces {
        jni: initial_jni_interface(),
        jmm: initial_jmm(),
        jvmti: initial_jvmti(),
        invoke_interface: initial_invoke_interface(),
    }
}
