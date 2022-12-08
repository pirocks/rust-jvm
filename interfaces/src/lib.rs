#![feature(box_syntax)]
#![feature(once_cell)]
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
        jni: box initial_jni_interface(),
        jmm: box initial_jmm(),
        jvmti: box initial_jvmti(),
        invoke_interface: box initial_invoke_interface(),
    }
}
