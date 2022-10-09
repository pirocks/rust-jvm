use std::collections::HashSet;
use std::ffi::OsString;
use std::iter::FromIterator;

use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::MethodId;

use crate::JVMState;
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
    pub(crate) assertions_enabled: bool,
    pub(crate) instruction_trace_options: InstructionTraceOptions,
    pub(crate) exit_trace_options: ExitTracingOptions,
}

pub enum ExitTracingOptions {
    TraceAll,
    TraceNone,
    TraceSome(!),
}

impl ExitTracingOptions {
    pub fn tracing_enabled(&self) -> bool {
        match self {
            ExitTracingOptions::TraceAll => true,
            ExitTracingOptions::TraceNone => false,
            ExitTracingOptions::TraceSome(_) => {
                todo!()
            }
        }
    }
}

#[derive(Eq, PartialEq, Clone, Debug, Hash)]
pub struct MethodToTrace {
    // method_name: String,
    // class_name: String,
    combined: String,
}

pub enum InstructionTraceOptions {
    TraceAll,
    TraceNone,
    TraceMethods(HashSet<MethodToTrace>),
}

impl InstructionTraceOptions {
    pub fn partial_tracing(&self) -> bool {
        match self {
            InstructionTraceOptions::TraceAll => false,
            InstructionTraceOptions::TraceNone => true,
            InstructionTraceOptions::TraceMethods(_) => true
        }
    }

    pub fn should_trace<'gc>(&self, method_id: MethodId, jvm: &'gc JVMState<'gc>) -> bool {
        match self {
            InstructionTraceOptions::TraceAll => {
                true
            }
            InstructionTraceOptions::TraceNone => {
                false
            }
            InstructionTraceOptions::TraceMethods(methods) => {
                let method = jvm.method_table.read().unwrap().lookup_method_string_no_desc(method_id, &jvm.string_pool);
                methods.contains(&MethodToTrace { combined: method })
            }
        }
    }
}

impl JVMOptions {
    pub fn new(
        main_class_name: ClassName,
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
        let trace_set = HashSet::from_iter(vec![
            //     /* MethodToTrace {
            //          combined: "com/google/common/base/Preconditions/checkNotNull".to_string(),
            //      },*/
            //      /*MethodToTrace {
            //          combined: "com/google/common/collect/StandardTable/put".to_string(),
            //      },*/
            //    /* MethodToTrace {
            //         combined: "java/util/AbstractMap/hashCode".to_string(),
            //     },
            //     MethodToTrace {
            //         combined: "java/util/HashMap/hash".to_string(),
            //     },*/
            // MethodToTrace {
            //     combined: "java/util/ArrayList/remove".to_string(),
            // },
            // MethodToTrace {
            //     combined: "java/util/ArrayList/checkForComodification".to_string(),
            // },
            MethodToTrace {
                combined: "io/netty/buffer/AbstractByteBuf/readByte".to_string(),
            },
            MethodToTrace {
                combined: "hd/readByte".to_string(),
            },
            MethodToTrace {
                combined: "hd/e".to_string(),
            },
            // MethodToTrace {
            //     combined: "uv/a".to_string(),
            // },
            // MethodToTrace {
            //     combined: "asd/a".to_string(),
            // },
            // MethodToTrace {
            //     combined: "ws/b".to_string(),
            // },
            // MethodToTrace {
            //     combined: "java/lang/Byte/valueOf".to_string(),
            // },
        ].into_iter());
        let trace_options = InstructionTraceOptions::TraceMethods(trace_set);
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
            instruction_trace_options: trace_options,
            exit_trace_options: ExitTracingOptions::TraceNone,
        }
    }

    #[cfg(test)]
    pub fn test_options() -> JVMOptions {
        JVMOptions {
            main_class_name: ClassName::Str("Main".to_string()),
            classpath: Classpath::from_dirs(vec![]),
            args: vec![],
            shared_libs: SharedLibraryPaths { libjava: Default::default(), libjdwp: Default::default() },
            enable_tracing: false,
            enable_jvmti: false,
            properties: vec![],
            unittest_mode: false,
            store_generated_classes: false,
            debug_print_exceptions: false,
            assertions_enabled: false,
            instruction_trace_options: InstructionTraceOptions::TraceNone,
            exit_trace_options: ExitTracingOptions::TraceNone,
        }
    }
}
