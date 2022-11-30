use std::collections::HashSet;
use std::iter::FromIterator;
use std::path::PathBuf;

use itertools::Itertools;

use jvm_args::JVMArgs;
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::MethodId;

use crate::JVMState;
use crate::loading::Classpath;

pub struct SharedLibraryPaths {
    pub libjava: PathBuf,
    pub libjdwp: PathBuf,
}

pub struct JVMOptions {
    pub main_class_name: ClassName,
    pub classpath: Classpath,
    pub args: Vec<String>,
    pub shared_libs: SharedLibraryPaths,
    pub enable_tracing: bool,
    pub enable_jvmti: bool,
    pub properties: Vec<(String, String)>,
    pub unittest_mode: bool,
    pub store_generated_classes: bool,
    pub debug_print_exceptions: bool,
    pub assertions_enabled: bool,
    pub instruction_trace_options: InstructionTraceOptions,
    pub exit_trace_options: ExitTracingOptions,
    pub thread_tracing_options: ThreadTracingOptions,
    pub java_home: PathBuf
}

pub struct JVMOptionsStart {
    main: String,
    java_home: PathBuf,
    classpath: Vec<PathBuf>,
    ext_classpath: Vec<PathBuf>,
    properties: Vec<(String, String)>,
    args: Vec<String>,
    enable_assertions: bool,
    store_anon_class: bool,
    debug_print_exceptions: bool,
}

impl JVMOptionsStart {
    pub fn classpath_format() -> impl Iterator<Item=&'static str> {
        // basically from hotspot/src/share/vm/runtime/os.cpp
        vec!["lib/resources.jar", "lib/rt.jar", "lib/sunrsasign.jar", "lib/jsse.jar", "lib/jce.jar", "lib/charsets.jar", "lib/jfr.jar", "classes"].into_iter()
    }

    pub fn ext_classpath_format() -> impl Iterator<Item=&'static str> {
        // basically from hotspot/src/share/vm/runtime/os.cpp
        vec!["/lib/ext"].into_iter()
    }

    pub fn from_java_home(java_home: PathBuf, parsed: JVMArgs) -> JVMOptionsStart {
        let JVMArgs {
            java_home,
            classpath,
            main,
            properties,
            args,
            enable_assertions,
            debug_exceptions,
            store_anon_class
        } = parsed.clone();
        let classpath = Self::classpath_format()
            .map(|classpath_elem| java_home.join(classpath_elem))
            .filter(|elem|elem.exists())
            .chain(classpath.into_iter().map(PathBuf::from))
            .collect_vec();

        let ext_classpath = Self::ext_classpath_format()
            .map(|classpath_elem| java_home.join(classpath_elem))
            .collect_vec();

        JVMOptionsStart {
            main,
            java_home,
            classpath,
            ext_classpath,
            properties,
            args,
            enable_assertions,
            store_anon_class,
            debug_print_exceptions: debug_exceptions,
        }
    }
}

pub struct ThreadTracingOptions {
    pub trace_monitor_wait_enter: bool,
    pub trace_monitor_wait_exit: bool,
    pub trace_monitor_notify: bool,
    pub trace_monitor_notify_all: bool,
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
    pub fn from_options_start(options_start: JVMOptionsStart) -> JVMOptions {
        let JVMOptionsStart { main, java_home, classpath, ext_classpath, properties, args, enable_assertions, store_anon_class, debug_print_exceptions } = options_start;
        let classpath = Classpath::from_dirs(classpath.into_iter().map(|path|path.into_boxed_path()).collect_vec());
        Self::new(
            ClassName::Str(main.replace('.', "/")),
            java_home.clone(),
            classpath,
            args,
            java_home.join("lib/amd64/libjava.so"),
            java_home.join("lib/amd64/libjdwp.so"),
            false,
            false,
            properties,
            false,
            store_anon_class,
            debug_print_exceptions,
            enable_assertions
        )
    }

    pub fn new(
        main_class_name: ClassName,
        java_home: PathBuf,
        classpath: Classpath,
        args: Vec<String>,
        libjava: PathBuf,
        libjdwp: PathBuf,
        enable_tracing: bool,
        enable_jvmti: bool,
        properties: Vec<(String, String)>,
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
            //     combined: "DebuggingClass/main".to_string()
            // },
            // MethodToTrace {
            //     combined: "java/lang/Short/valueOf".to_string()
            // },
            // MethodToTrace {
            //     combined: "java/lang/Character/compareTo".to_string()
            // },
            // MethodToTrace {
            //     combined: "java/lang/Short/compare".to_string(),
            // },
            // MethodToTrace {
            //     combined: "io/netty/buffer/UnpooledHeapByteBuf/_getByte".to_string(),
            // },
            // MethodToTrace {
            //     combined: "io/netty/buffer/AbstractByteBuf/readByte".to_string(),
            // },
            // MethodToTrace {
            //     combined: "hd/readByte".to_string(),
            // },
            // MethodToTrace {
            //     combined: "xx/cm".to_string(),
            // },
            // MethodToTrace {
            //     combined: "uv/a".to_string(),
            // },
            // MethodToTrace {
            //     combined: "PowTests/testCrossProduct".to_string(),
            // },
            // MethodToTrace {
            //     combined: "sun/nio/ch/ServerSocketChannelImpl/translateReadyOps".to_string(),
            // },
            // MethodToTrace {
            //     combined: "java/util/TimeZone/getDisplayName".to_string(),
            // },
        ].into_iter());
        let trace_options = InstructionTraceOptions::TraceMethods(trace_set);
        let thread_tracing_options = ThreadTracingOptions {
            trace_monitor_wait_enter: false,
            trace_monitor_wait_exit: false,
            trace_monitor_notify: false,
            trace_monitor_notify_all: false,
        };
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
            thread_tracing_options,
            java_home,
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
            thread_tracing_options: todo!(),
        }
    }
}
