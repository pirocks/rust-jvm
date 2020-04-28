use rust_jvm_common::classnames::ClassName;
use crate::{ThreadId, JVMState};
use crate::monitor::Monitor;

pub struct TracingSettings {
    trace_function_end: bool,
    trace_function_start: bool,
    trace_jni_register: bool,
    trace_jni_dynamic_link: bool,
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
            trace_jni_dynamic_link: false,
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

    pub fn trace_function_enter(&self, classname: &ClassName, meth_name: &str, method_desc: &str, current_depth: usize, threadtid: ThreadId) {
        if self.trace_function_start {
            println!("CALL END:{:?} {} {} {} {}", classname, meth_name, method_desc, current_depth, threadtid);
        }
    }

    pub fn trace_function_exit(&self, classname: &ClassName, meth_name: &str, method_desc: &str, current_depth: usize, threadtid: ThreadId) {
        if self.trace_function_end {
            println!("CALL END:{:?} {} {} {} {}", classname, meth_name, method_desc, current_depth, threadtid);
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

    pub fn trace_monitor_lock(&self, m: &Monitor, jvm: &JVMState) {
        if self.trace_monitor_lock {
            println!("Monitor lock:{}/{}, thread:{} {}", m.name, m.monitor_i, std::thread::current().name().unwrap_or("unknown"), Monitor::get_tid(jvm));
        }
    }

    pub fn trace_monitor_unlock(&self, m: &Monitor, jvm: &JVMState) {
        if self.trace_monitor_unlock {
            println!("Monitor unlock:{}/{}, thread:{} {}", m.name, m.monitor_i, jvm.get_current_thread_name(), Monitor::get_tid(jvm));
        }
    }

    pub fn trace_monitor_wait(&self, m: &Monitor, jvm: &JVMState) {
        if self.trace_monitor_wait {
            println!("Monitor wait:{}, thread:{}", m.name, jvm.get_current_thread_name());
        }
    }

    pub fn trace_monitor_notify(&self, m: &Monitor, jvm: &JVMState) {
        if self.trace_monitor_notify {
            println!("Monitor notify:{}, thread:{}", m.name, jvm.get_current_thread_name());
        }
    }

    pub fn trace_monitor_notify_all(&self, m: &Monitor, jvm: &JVMState) {
        if self.trace_monitor_notify_all {
            println!("Monitor notify all:{}, thread:{}", m.name, jvm.get_current_thread_name());
        }
    }

    pub fn trace_jdwp_function_enter(&self, jvm: &JVMState, function_name: &str) {
        if self.trace_jdwp_function_enter{
            let current_thread = std::thread::current();
            let vm_life = if jvm.vm_live() {
                current_thread.name().unwrap_or("unknown thread")
            }else {
                "VM not live"
            };
            println!("JVMTI [{}] {} {{",vm_life, function_name);
        }
    }

    pub fn trace_jdwp_function_exit(&self, jvm: &JVMState, function_name: &str) {
        if self.trace_jdwp_function_enter{
            let current_thread = std::thread::current();
            let vm_life = if jvm.vm_live() {
                current_thread.name().unwrap_or("unknown thread")
            }else {
                "VM not live"
            };
            println!("JVMTI [{}] {} }}",vm_life, function_name);
        }
    }
}
