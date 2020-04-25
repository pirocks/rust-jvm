use rust_jvm_common::classnames::ClassName;
use crate::ThreadId;

pub struct TracingSettings {
    trace_fucntion_end: bool,
    trace_function_start: bool,
    trace_jni_register : bool,
    trace_jni_dynamic_link : bool,
    trace_class_loads: bool,
    trace_jdwp_events: bool,
    trace_jdwp_function_enter: bool,
    trace_jdwp_function_exit: bool,

}

impl TracingSettings{
    pub fn new() -> Self{
        TracingSettings {
            trace_fucntion_end: false,
            trace_function_start: false,
            trace_jni_register: false,
            trace_jni_dynamic_link: false,
            trace_class_loads: false,
            trace_jdwp_events: false,
            trace_jdwp_function_enter: false,
            trace_jdwp_function_exit: false//todo parse this from options in future
        }
    }
    pub fn trace_function_enter(&self, classname: &ClassName,meth_name: &str,method_desc: &str, current_depth: usize, threadtid: ThreadId){
        println!("CALL END:{:?} {} {} {}", classname, meth_name,method_desc, current_depth, );
    }
    pub fn trace_function_exit(&self, classname: &ClassName,meth_name: &str,method_desc: &str, current_depth: usize, threadtid: ThreadId){
        println!("CALL END:{:?} {} {} {}", classname, meth_name,method_desc, current_depth, );
    }
}
