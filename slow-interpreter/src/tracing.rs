use std::sync::RwLock;

use jvmti_jni_bindings::{jvmtiError, jvmtiError_JVMTI_ERROR_NONE};
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::compressed_classfile::{CompressedClassfileStringPool, CPDType};
use rust_jvm_common::compressed_classfile::names::MethodName;
use rust_jvm_common::JavaThreadId;

use crate::java_values::JavaValue;
use crate::JVMState;
use crate::threading::monitors::Monitor;

pub struct TracingSettings {
    pub trace_function_end: RwLock<bool>,
    pub trace_function_start: RwLock<bool>,
    trace_jni_register: bool,
    _trace_jni_dynamic_link: bool,
    //todo implement this trace
    trace_class_loads: bool,
    trace_jdwp_events: bool,
    trace_jdwp_function_enter: bool,
    trace_jdwp_function_exit: bool,
    trace_monitor_lock: bool,
    trace_monitor_unlock: bool,
    trace_monitor_wait: bool,
    trace_monitor_notify: bool,
    trace_monitor_notify_all: bool,
}

impl TracingSettings {
    pub fn new() -> Self {
        TracingSettings {
            trace_function_end: RwLock::new(false),
            trace_function_start: RwLock::new(false),
            trace_jni_register: false,
            _trace_jni_dynamic_link: false,
            trace_class_loads: false,
            trace_jdwp_events: true,
            trace_jdwp_function_enter: true,
            trace_jdwp_function_exit: true, //todo parse this from options in future
            trace_monitor_lock: false,
            trace_monitor_unlock: false,
            trace_monitor_wait: false,
            trace_monitor_notify: false,
            trace_monitor_notify_all: false,
        }
    }

    pub fn disabled() -> Self {
        Self {
            trace_function_end: RwLock::new(false),
            trace_function_start: RwLock::new(false),
            trace_jni_register: false,
            _trace_jni_dynamic_link: false,
            trace_class_loads: false,
            trace_jdwp_events: false,
            trace_jdwp_function_enter: false,
            trace_jdwp_function_exit: false,
            trace_monitor_lock: false,
            trace_monitor_unlock: false,
            trace_monitor_wait: false,
            trace_monitor_notify: false,
            trace_monitor_notify_all: false,
        }
    }

    pub fn trace_function_enter<'l>(&self, pool: &'l CompressedClassfileStringPool, classname: &'l CPDType, meth_name: &'l MethodName, method_desc: &'l str, current_depth: usize, threadtid: JavaThreadId) -> FunctionEnterExitTraceGuard<'l> {
        // unsafe {
        // if TIMES > 25000000 && !classname.class_name_representation().contains("java") && !classname.class_name_representation().contains("google")
        //     && !meth_name.contains("hashCode")
        //     && !meth_name.contains("equals"){
        //     println!("{:indent$}start:{:?} {} {}","", classname, meth_name, method_desc,indent = current_depth);
        // }
        // }
        //IN BEG.<INIT>, second iterator
        //
        // if *self.trace_function_start.read().unwrap() {
        // }
        FunctionEnterExitTraceGuard {
            string_pool: pool,
            classname,
            meth_name,
            method_desc,
            current_depth,
            threadtid,
            trace_function_end: *self.trace_function_end.read().unwrap(),
        }
    }

    pub fn trace_jni_register(&self, classname: &ClassName, meth_name: &str) {
        if self.trace_jni_register {
            println!("[Registering JNI native method {}.{}]", classname.get_referred_name().replace("/", "."), meth_name);
        }
    }

    pub fn trace_class_loads(&self, classname: &ClassName) {
        if self.trace_class_loads {
            println!("[Loaded {} from unknown]", classname.get_referred_name().replace("/", "."));
        }
    }

    pub fn trace_monitor_lock<'gc>(&self, m: &Monitor, jvm: &'gc JVMState<'gc>) {
        if self.trace_monitor_lock {
            println!("Monitor lock:{}/{}, thread:{} {}", m.name, m.monitor_i, std::thread::current().name().unwrap_or("unknown"), Monitor::get_tid(jvm));
        }
    }

    pub fn trace_monitor_unlock<'gc>(&self, m: &Monitor, jvm: &'gc JVMState<'gc>) {
        if self.trace_monitor_unlock {
            println!("Monitor unlock:{}/{}, thread:{} {}", m.name, m.monitor_i, jvm.thread_state.get_current_thread_name(jvm), Monitor::get_tid(jvm));
        }
    }

    pub fn trace_monitor_wait<'gc>(&self, m: &Monitor, jvm: &'gc JVMState<'gc>) {
        if self.trace_monitor_wait {
            println!("Monitor wait:{}, thread:{}", m.name, jvm.thread_state.get_current_thread_name(jvm));
        }
    }

    pub fn trace_monitor_notify<'gc>(&self, m: &Monitor, jvm: &'gc JVMState<'gc>) {
        if self.trace_monitor_notify {
            println!("Monitor notify:{}, thread:{}", m.name, jvm.thread_state.get_current_thread_name(jvm));
        }
    }

    pub fn trace_monitor_notify_all<'gc>(&self, m: &Monitor, jvm: &'gc JVMState<'gc>) {
        if self.trace_monitor_notify_all {
            println!("Monitor notify all:{}, thread:{}", m.name, jvm.thread_state.get_current_thread_name(jvm));
        }
    }

    pub fn trace_jdwp_function_enter<'gc>(&self, jvm: &'gc JVMState<'gc>, function_name: &'static str) -> JVMTIEnterExitTraceGuard {
        let current_thread = std::thread::current();
        let thread_name = if jvm.vm_live() { current_thread.name().unwrap_or("unknown thread") } else { "VM not live" }.to_string();
        if self.trace_jdwp_function_enter && function_name != "Deallocate" && function_name != "Allocate" && function_name != "RawMonitorNotify" && function_name != "RawMonitorExit" && function_name != "RawMonitorWait" && function_name != "RawMonitorEnter" {
            println!("JVMTI [{}] {} {{ ", thread_name, function_name);
        }
        JVMTIEnterExitTraceGuard {
            correctly_exited: false,
            thread_name,
            function_name,
            trace_jdwp_function_exit: self.trace_jdwp_function_exit,
        }
    }

    pub fn function_exit_guard<'gc>(&self, guard: FunctionEnterExitTraceGuard, _res: JavaValue<'gc>) {
        // if TIMES > 25000000 && !guard.classname.class_name_representation().contains("java") && !guard.classname.class_name_representation().contains("google")
        //     && !guard.meth_name.contains("hashCode")
        //     && !guard.meth_name.contains("equals"){
        //     println!("{:indent$}exit:{} {} {}","", guard.classname.class_name_representation(), guard.meth_name,res.try_unwrap_int().map(|int|int.to_string()).unwrap_or("not int".to_string()),indent= guard.current_depth);
        // }
        drop(guard);
    }

    pub fn trace_jdwp_function_exit(&self, mut guard: JVMTIEnterExitTraceGuard, error: jvmtiError) -> jvmtiError {
        if guard.trace_jdwp_function_exit && guard.function_name != "Deallocate" && guard.function_name != "Allocate" && guard.function_name != "RawMonitorNotify" && guard.function_name != "RawMonitorExit" && guard.function_name != "RawMonitorWait" && guard.function_name != "RawMonitorEnter" {
            if error != jvmtiError_JVMTI_ERROR_NONE {
                println!("JVMTI [{}] {} }} {:?}", guard.thread_name, guard.function_name, error);
            } else {
                println!("JVMTI [{}] {} }}", guard.thread_name, guard.function_name);
            }
        }
        guard.correctly_exited = true;
        drop(guard);
        error
    }

    pub fn trace_event_enable_global(&self, event_name: &str) {
        if self.trace_jdwp_events {
            println!(
                "JVMTI [ALL] # user enabled event {}
JVMTI [-] # recompute enabled - before 0
JVMTI [-] # Enabling event {}
JVMTI [-] # recompute enabled - after 0",
                event_name, event_name
            );
        }
    }

    pub fn trace_event_disable_global(&self, event_name: &str) {
        if self.trace_jdwp_events {
            println!(
                "JVMTI [ALL] # user disabled event {}
JVMTI [-] # recompute enabled - before 0
JVMTI [-] # Disabling event {}
JVMTI [-] # recompute enabled - after 0",
                event_name, event_name
            );
        }
    }

    pub fn trace_event_trigger(&self, event_name: &str) {
        if self.trace_jdwp_events {
            println!("JVMTI Trg {} triggered", event_name)
        }
    }
}

pub struct JVMTIEnterExitTraceGuard {
    pub correctly_exited: bool,
    pub thread_name: String,
    pub function_name: &'static str,
    pub trace_jdwp_function_exit: bool,
}

impl Drop for JVMTIEnterExitTraceGuard {
    fn drop(&mut self) {
        // assert!(self.correctly_exited);
    }
}

pub struct FunctionEnterExitTraceGuard<'l> {
    string_pool: &'l CompressedClassfileStringPool,
    classname: &'l CPDType,
    meth_name: &'l MethodName,
    method_desc: &'l str,
    current_depth: usize,
    threadtid: JavaThreadId,
    trace_function_end: bool,
}

impl Drop for FunctionEnterExitTraceGuard<'_> {
    fn drop(&mut self) {
        if self.trace_function_end {
            println!("CALL END:{:?} {} {} {} {}", self.classname, self.meth_name.0.to_str(self.string_pool), self.method_desc, self.current_depth, self.threadtid);
        }
    }
}