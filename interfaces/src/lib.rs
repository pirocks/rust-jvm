#![feature(c_variadic)]
#![allow(unused_variables)]
#![allow(unreachable_code)]

use jmm_interface::initial_jmm;
use jni_interface::jni::initial_jni_interface;
use jvmti_interface::initial_jvmti;
use slow_interpreter::rust_jni::PerStackInterfaces;
use crate::invoke_interface::initial_invoke_interface;

pub mod invoke_interface;

pub fn initial_per_stack_interfaces() -> PerStackInterfaces {
    PerStackInterfaces {
        jni: Box::new(initial_jni_interface()),
        jmm: Box::new(initial_jmm()),
        jvmti: Box::new(initial_jvmti()),
        invoke_interface: Box::new(initial_invoke_interface()),
    }
}
