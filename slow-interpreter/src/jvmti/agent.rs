use jvmti_jni_bindings::{jvmtiEnv, jthread, jvmtiStartFunction, jint, jvmtiError, _jobject, jvmtiError_JVMTI_ERROR_NONE, JVMTI_THREAD_MAX_PRIORITY, JVMTI_THREAD_NORM_PRIORITY, JVMTI_THREAD_MIN_PRIORITY, scanf};
use crate::jvmti::{get_state, get_jvmti_interface, get_interpreter_state};
use crate::interpreter_util::check_inited_class;
use rust_jvm_common::classnames::ClassName;
use crate::rust_jni::interface::get_interface;
use std::mem::{transmute, transmute_copy};
use std::os::raw::c_void;
use crate::rust_jni::native_util::{from_object};
use crate::java_values::JavaValue;
use thread_priority::*;
use crate::threading::JavaThread;
use userspace_threads::Threads;
use std::sync::Arc;
use crate::InterpreterStateGuard;

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
        jvm.jvmti_state.as_ref().unwrap().built_in_jdwp.thread_start(jvm, java_thread.thread_object());

        let mut guard = InterpreterStateGuard {
            int_state: java_thread.interpreter_state.write().unwrap().into(),
            thread: &java_thread,
        };

        let mut jvmti = get_jvmti_interface(jvm);
        let mut jni_env = get_interface(jvm,&mut guard);
        proc_.unwrap()(&mut jvmti, transmute(&mut jni_env), arg as *mut c_void);
    }, box ());

    //todo handle join handles somehow
    jvm.tracing.trace_jdwp_function_exit(jvm, "RunAgentThread");
    jvmtiError_JVMTI_ERROR_NONE
}
