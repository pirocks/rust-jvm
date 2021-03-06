use std::os::raw::c_void;

use thread_priority::*;

use classfile_view::loading::LoaderName;
use jvmti_jni_bindings::{jint, jthread, JVMTI_THREAD_MAX_PRIORITY, JVMTI_THREAD_MIN_PRIORITY, JVMTI_THREAD_NORM_PRIORITY, jvmtiEnv, jvmtiError, jvmtiError_JVMTI_ERROR_NONE, jvmtiStartFunction};

use crate::InterpreterStateGuard;
use crate::java_values::JavaValue;
use crate::jvmti::{get_jvmti_interface, get_state};
use crate::rust_jni::interface::get_interface;
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
    let tracing_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "RunAgentThread");
    let thread_object = JavaValue::Object(from_object(thread)).cast_thread();
    let java_thread = JavaThread::new(jvm, thread_object.clone(), jvm.thread_state.threads.create_thread(thread_object.name().to_rust_string().into()), true);
    let args = ThreadArgWrapper { proc_, arg };
    java_thread.clone().get_underlying().start_thread(box move |_| {
        let ThreadArgWrapper { proc_, arg } = args;
        if priority == JVMTI_THREAD_MAX_PRIORITY as i32 {
            set_current_thread_priority(ThreadPriority::Max).unwrap();
        } else if priority == JVMTI_THREAD_NORM_PRIORITY as i32 {} else if priority == JVMTI_THREAD_MIN_PRIORITY as i32 {
            set_current_thread_priority(ThreadPriority::Min).unwrap();//todo pass these to object
        }


        jvm.thread_state.set_current_thread(java_thread.clone());
        java_thread.notify_alive();

        let mut int_state = InterpreterStateGuard::new(jvm, &java_thread);
        int_state.register_interpreter_state_guard(jvm);
        jvm.jvmti_state.as_ref().unwrap().built_in_jdwp.thread_start(jvm, &mut int_state, java_thread.thread_object());

        let jvmti = get_jvmti_interface(jvm, &mut int_state);
        let jni_env = get_interface(jvm, &mut int_state);
        assert!(int_state.int_state.as_ref().unwrap().call_stack.is_empty());
        let frame_for_agent = int_state.push_frame(StackEntry::new_completely_opaque_frame(LoaderName::BootstrapLoader));
        proc_.unwrap()(jvmti, jni_env, arg as *mut c_void);
        int_state.pop_frame(jvm, frame_for_agent, false);
        java_thread.notify_terminated()
    }, box ());

    //todo handle join handles somehow
    jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}
