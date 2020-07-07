use std::mem::transmute;
use std::os::raw::c_void;

use thread_priority::*;

use jvmti_jni_bindings::{jint, jthread, JVMTI_THREAD_MAX_PRIORITY, JVMTI_THREAD_MIN_PRIORITY, JVMTI_THREAD_NORM_PRIORITY, jvmtiEnv, jvmtiError, jvmtiError_JVMTI_ERROR_NONE, jvmtiStartFunction};
use rust_jvm_common::classnames::ClassName;

use crate::interpreter_util::check_inited_class;
use crate::InterpreterStateGuard;
use crate::java_values::JavaValue;
use crate::jvmti::{get_jvmti_interface, get_state};
use crate::rust_jni::interface::get_interface;
use crate::rust_jni::interface::local_frame::{clear_local_refs, local_refs_len};
use crate::rust_jni::native_util::from_object;
use crate::stack_entry::StackEntry;
use crate::threading::JavaThread;

struct ThreadArgWrapper {
    proc_: jvmtiStartFunction,
    arg: *const ::std::os::raw::c_void,
}

unsafe impl Send for ThreadArgWrapper {}

unsafe impl Sync for ThreadArgWrapper {}

pub unsafe extern "C" fn run_agent_thread(env: *mut jvmtiEnv, thread: jthread, proc_: jvmtiStartFunction, arg: *const ::std::os::raw::c_void, priority: jint) -> jvmtiError {
    //todo implement thread priority
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm, "RunAgentThread");
    let thread_object = JavaValue::Object(from_object(transmute(thread))).cast_thread();
    let java_thread = JavaThread::new(jvm, thread_object, jvm.thread_state.threads.create_thread(), true);
    let args = ThreadArgWrapper { proc_, arg };
    java_thread.clone().get_underlying().start_thread(box move |_| {
        let ThreadArgWrapper { proc_, arg } = args;
        if priority == JVMTI_THREAD_MAX_PRIORITY as i32 {
            set_current_thread_priority(ThreadPriority::Max).unwrap();
        } else if priority == JVMTI_THREAD_NORM_PRIORITY as i32 {} else if priority == JVMTI_THREAD_MIN_PRIORITY as i32 {
            set_current_thread_priority(ThreadPriority::Min).unwrap();
        }


        let mut guard = InterpreterStateGuard {
            int_state: java_thread.interpreter_state.write().unwrap().into(),
            thread: &java_thread,
        };
        jvm.thread_state.set_current_thread(java_thread.clone());
        jvm.jvmti_state.as_ref().unwrap().built_in_jdwp.thread_start(jvm, &mut guard, java_thread.thread_object());
        let thread_class = check_inited_class(jvm, &mut guard, &ClassName::thread().into(), jvm.bootstrap_loader.clone());
        guard.push_frame(StackEntry {
            class_pointer: thread_class,
            method_i: 0,
            local_vars: vec![],
            operand_stack: vec![],
            pc: 0,
            pc_offset: 0,
        });

        let jvmti = get_jvmti_interface(jvm, &mut guard);
        let jni_env = get_interface(jvm, &mut guard);
        let local_refs_len = local_refs_len();
        proc_.unwrap()(jvmti, jni_env, arg as *mut c_void);
        clear_local_refs(local_refs_len);
    }, box ());

    //todo handle join handles somehow
    jvm.tracing.trace_jdwp_function_exit(jvm, "RunAgentThread");
    jvmtiError_JVMTI_ERROR_NONE
}
