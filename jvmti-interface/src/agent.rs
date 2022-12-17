use std::os::raw::c_void;

use thread_priority::*;

use another_jit_vm::stack::CannotAllocateStack;
use jvmti_jni_bindings::{jint, jthread, JVMTI_THREAD_MAX_PRIORITY, JVMTI_THREAD_MIN_PRIORITY, JVMTI_THREAD_NORM_PRIORITY, jvmtiEnv, jvmtiError, jvmtiError_JVMTI_ERROR_NONE, jvmtiStartFunction};

use slow_interpreter::java_values::JavaValue;
use slow_interpreter::jvm_state::JVMState;
use slow_interpreter::rust_jni::jvmti::get_jvmti_interface;
use slow_interpreter::rust_jni::native_util::from_object;
use slow_interpreter::threading::java_thread::JavaThread;
use slow_interpreter::rust_jni::jvmti::{get_state};

struct ThreadArgWrapper {
    proc_: jvmtiStartFunction,
    arg: *const c_void,
}

unsafe impl Send for ThreadArgWrapper {}

unsafe impl Sync for ThreadArgWrapper {}

pub unsafe extern "C" fn run_agent_thread<'gc>(env: *mut jvmtiEnv, thread: jthread, proc_: jvmtiStartFunction, arg: *const c_void, priority: jint) -> jvmtiError {
    //todo implement thread priority
    let jvm: &'gc JVMState<'gc> = get_state(env);
    let tracing_guard = jvm.config.tracing.trace_jdwp_function_enter(jvm, "RunAgentThread");
    let thread_object = JavaValue::Object(from_object(jvm, thread)).cast_thread();
    let args = ThreadArgWrapper { proc_, arg };
    let java_thread = match JavaThread::background_new_with_stack(jvm, Some(thread_object), true, move |thread,frame|{
        let ThreadArgWrapper { proc_, arg } = args;
        if priority == JVMTI_THREAD_MAX_PRIORITY as i32 {
            set_current_thread_priority(ThreadPriority::Max).unwrap();
        } else if priority == JVMTI_THREAD_NORM_PRIORITY as i32 {} else if priority == JVMTI_THREAD_MIN_PRIORITY as i32 {
            set_current_thread_priority(ThreadPriority::Min).unwrap(); //todo pass these to object
        }

        jvm.thread_state.set_current_thread(thread.clone());
        thread.notify_alive();

        // assert!(should_be_nothing.old.is_none());
        jvm.native.jvmti_state.as_ref().unwrap().built_in_jdwp.thread_start(jvm, frame, thread.thread_object());

        let jvmti = get_jvmti_interface(jvm, todo!()/*&mut int_state*/);
        let jni_env = todo!()/*get_interface(jvm, todo!()/*&mut int_state*/, )*/;
        // let frame_for_agent = int_state.push_frame(todo!()/*StackEntryPush::new_completely_opaque_frame(jvm,LoaderName::BootstrapLoader, vec![],"agent_frame")*/);
        proc_.unwrap()(jvmti, jni_env, arg as *mut c_void);
        // java_thread.notify_terminated(jvm)
        todo!()
    }) {
        Ok(java_thread) => java_thread,
        Err(CannotAllocateStack {}) => {
            todo!()
        }
    };


    //todo handle join handles somehow
    jvm.config.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}