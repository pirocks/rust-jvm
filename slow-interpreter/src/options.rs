use rust_jvm_common::classnames::ClassName;

use crate::loading::Classpath;

pub struct SharedLibraryPaths {
    libjava: String,
    libjdwp: String,
}

pub struct JVMOptions {
    main_class_name: ClassName,
    classpath: Classpath,
    args: Vec<String>,
    //todo args not implemented yet
    shared_libs: SharedLibraryPaths,
    enable_tracing: bool,
    enable_jvmti: bool,
    properties: Vec<String>,
}

impl JVMOptions {
    pub fn new(main_class_name: ClassName,
               classpath: Classpath,
               args: Vec<String>,
               libjava: String,
               libjdwp: String,
               enable_tracing: bool,
               enable_jvmti: bool,
               properties: Vec<String>,
    ) -> Self {
        Self {
            main_class_name,
            classpath,
            args,
            shared_libs: SharedLibraryPaths { libjava, libjdwp },
            enable_tracing,
            enable_jvmti,
            properties,
        }
    }
}
