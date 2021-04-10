use std::ffi::OsString;

use rust_jvm_common::classnames::ClassName;

use crate::loading::Classpath;

pub struct SharedLibraryPaths {
    pub(crate) libjava: OsString,
    pub(crate) libjdwp: OsString,
}

pub struct JVMOptions {
    pub(crate) main_class_name: ClassName,
    pub(crate) classpath: Classpath,
    pub(crate) args: Vec<String>,
    pub(crate) shared_libs: SharedLibraryPaths,
    pub(crate) enable_tracing: bool,
    pub(crate) enable_jvmti: bool,
    pub(crate) properties: Vec<String>,
    pub(crate) unittest_mode: bool,
    pub(crate) store_generated_classes: bool,
    pub(crate) debug_print_exceptions: bool,
    pub(crate) assertions_enabled: bool
}

impl JVMOptions {
    pub fn new(main_class_name: ClassName,
               classpath: Classpath,
               args: Vec<String>,
               libjava: OsString,
               libjdwp: OsString,
               enable_tracing: bool,
               enable_jvmti: bool,
               properties: Vec<String>,
               unittest_mode: bool,
               store_generated_classes: bool,
               debug_print_exceptions: bool,
               assertions_enabled: bool,
    ) -> Self {
        Self {
            main_class_name,
            classpath,
            args,
            shared_libs: SharedLibraryPaths { libjava, libjdwp },
            enable_tracing,
            enable_jvmti,
            properties,
            unittest_mode,
            store_generated_classes,
            debug_print_exceptions,
            assertions_enabled,
        }
    }
}
