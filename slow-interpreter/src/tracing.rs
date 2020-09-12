use jvmti_jni_bindings::{jvmtiError, jvmtiError_JVMTI_ERROR_NONE};
use rust_jvm_common::classnames::ClassName;

use crate::JVMState;
use crate::threading::JavaThreadId;
use crate::threading::monitors::Monitor;

pub struct TracingSettings {
    trace_function_end: bool,
    trace_function_start: bool,
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
            trace_function_end: false,
            trace_function_start: false,
            trace_jni_register: false,
            _trace_jni_dynamic_link: false,
            trace_class_loads: false,
            trace_jdwp_events: true,
            trace_jdwp_function_enter: true,
            trace_jdwp_function_exit: true,//todo parse this from options in future
            trace_monitor_lock: false,
            trace_monitor_unlock: false,
            trace_monitor_wait: false,
            trace_monitor_notify: false,
            trace_monitor_notify_all: false,
        }
    }

    pub fn disabled() -> Self {
        Self {
            trace_function_end: false,
            trace_function_start: false,
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

    pub fn trace_function_enter<'l>(&self, classname: &'l ClassName, meth_name: &'l str, method_desc: &'l str, current_depth: usize, threadtid: JavaThreadId) -> FunctionEnterExitTraceGuard<'l> {
        if self.trace_function_start {
            println!("CALL END:{:?} {} {} {} {}", classname, meth_name, method_desc, current_depth, threadtid);
        }
        return FunctionEnterExitTraceGuard {
            classname,
            meth_name,
            method_desc,
            current_depth,
            threadtid,
            trace_function_end: self.trace_function_end,
        };
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

    pub fn trace_monitor_lock(&self, m: &Monitor, jvm: &'static JVMState) {
        if self.trace_monitor_lock {
            println!("Monitor lock:{}/{}, thread:{} {}", m.name, m.monitor_i, std::thread::current().name().unwrap_or("unknown"), Monitor::get_tid(jvm));
        }
    }

    pub fn trace_monitor_unlock(&self, m: &Monitor, jvm: &'static JVMState) {
        if self.trace_monitor_unlock {
            println!("Monitor unlock:{}/{}, thread:{} {}", m.name, m.monitor_i, jvm.thread_state.get_current_thread_name(), Monitor::get_tid(jvm));
        }
    }

    pub fn trace_monitor_wait(&self, m: &Monitor, jvm: &'static JVMState) {
        if self.trace_monitor_wait {
            println!("Monitor wait:{}, thread:{}", m.name, jvm.thread_state.get_current_thread_name());
        }
    }

    pub fn trace_monitor_notify(&self, m: &Monitor, jvm: &'static JVMState) {
        if self.trace_monitor_notify {
            println!("Monitor notify:{}, thread:{}", m.name, jvm.thread_state.get_current_thread_name());
        }
    }

    pub fn trace_monitor_notify_all(&self, m: &Monitor, jvm: &'static JVMState) {
        if self.trace_monitor_notify_all {
            println!("Monitor notify all:{}, thread:{}", m.name, jvm.thread_state.get_current_thread_name());
        }
    }

    pub fn trace_jdwp_function_enter(&self, jvm: &'static JVMState, function_name: &'static str) -> JVMTIEnterExitTraceGuard {
        let current_thread = std::thread::current();
        let thread_name = if jvm.vm_live() {
            current_thread.name().unwrap_or("unknown thread")
        } else {
            "VM not live"
        }.to_string();
        if self.trace_jdwp_function_enter {
            println!("JVMTI [{}] {} {{ ", thread_name, function_name);
        }
        JVMTIEnterExitTraceGuard {
            correctly_exited: false,
            thread_name,
            function_name,
            trace_jdwp_function_exit: self.trace_jdwp_function_exit,
        }
    }

    pub fn function_exit_guard(&self, guard: FunctionEnterExitTraceGuard) {
        drop(guard);
    }

    pub fn trace_jdwp_function_exit(&self, mut guard: JVMTIEnterExitTraceGuard, error: jvmtiError) -> jvmtiError {
        if error != jvmtiError_JVMTI_ERROR_NONE {
            println!("JVMTI [{}] {} }} {:?}", guard.thread_name, guard.function_name, error);
        } else {
            println!("JVMTI [{}] {} }}", guard.thread_name, guard.function_name);
        }
        guard.correctly_exited = true;
        drop(guard);
        error
    }

    pub fn trace_event_enable_global(&self, event_name: &str) {
        if self.trace_jdwp_events {
            println!("JVMTI [ALL] # user enabled event {}
JVMTI [-] # recompute enabled - before 0
JVMTI [-] # Enabling event {}
JVMTI [-] # recompute enabled - after 0", event_name, event_name);
        }
    }

    pub fn trace_event_disable_global(&self, event_name: &str) {
        if self.trace_jdwp_events {
            println!("JVMTI [ALL] # user disabled event {}
JVMTI [-] # recompute enabled - before 0
JVMTI [-] # Disabling event {}
JVMTI [-] # recompute enabled - after 0", event_name, event_name);
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
        assert!(self.correctly_exited);
    }
}

pub struct FunctionEnterExitTraceGuard<'l> {
    classname: &'l ClassName,
    meth_name: &'l str,
    method_desc: &'l str,
    current_depth: usize,
    threadtid: JavaThreadId,
    trace_function_end: bool,
}

impl Drop for FunctionEnterExitTraceGuard<'_> {
    fn drop(&mut self) {
        if self.trace_function_end {
            println!("CALL END:{:?} {} {} {} {}", self.classname, self.meth_name, self.method_desc, self.current_depth, self.threadtid);
        }
    }
}