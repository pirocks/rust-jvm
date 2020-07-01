use jvmti_jni_bindings::{jvmtiEnv, jthread, jvmtiStartFunction, jint, jvmtiError, _jobject, jvmtiError_JVMTI_ERROR_NONE, JVMTI_THREAD_MAX_PRIORITY, JVMTI_THREAD_NORM_PRIORITY, JVMTI_THREAD_MIN_PRIORITY};
use crate::jvmti::{get_state, get_jvmti_interface};
use crate::interpreter_util::check_inited_class;
use rust_jvm_common::classnames::ClassName;
use crate::rust_jni::interface::get_interface;
use std::mem::transmute;
use std::os::raw::c_void;
use crate::rust_jni::native_util::from_object;
use crate::java_values::JavaValue;
use thread_priority::*;

struct ThreadArgWrapper {
    proc_: jvmtiStartFunction,
    arg: *const ::std::os::raw::c_void,
    thread: *mut _jobject,
}

unsafe impl Send for ThreadArgWrapper {}

unsafe impl Sync for ThreadArgWrapper {}

pub unsafe extern "C" fn run_agent_thread(env: *mut jvmtiEnv, thread: jthread, proc_: jvmtiStartFunction, arg: *const ::std::os::raw::c_void, priority: jint) -> jvmtiError {
    //todo implement thread priority
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm, "RunAgentThread");
    let name = JavaValue::Object(from_object(transmute(thread)))
        .cast_thread()
        .name()
        .to_rust_string();
    let args = ThreadArgWrapper { proc_, arg, thread };
    let system_class = check_inited_class(jvm, &ClassName::system().into(), jvm.bootstrap_loader.clone());
//TODO ADD THREAD TO JVM STATE STRUCT
    //todo handle join handles somehow
    let _join_handle = std::thread::Builder::new()
        .name(name)
        .spawn(move || {
            if priority == JVMTI_THREAD_MAX_PRIORITY as i32 {
                set_current_thread_priority(ThreadPriority::Max).unwrap();
            } else if priority == JVMTI_THREAD_NORM_PRIORITY as i32 {
                // unimplemented!()
            } else if priority == JVMTI_THREAD_MIN_PRIORITY as i32 {
                set_current_thread_priority(ThreadPriority::Min).unwrap();
            }
            let ThreadArgWrapper { proc_, arg, thread } = args;
            let thread_object = JavaValue::Object(from_object(transmute(thread))).cast_thread();
            // let agent_thread =
            // let result = jvm.thread_state.alive_threads.write();
            // result.unwrap().insert(agent_thread.java_tid, agent_thread.clone());//todo needs to be done via JavaThread constructor
            // todo this isn't strictly a java thread so not alive?
            println!("start thread:{}", &thread_object.name().to_rust_string());
            // jvm.init_signal_handler();
            // jvm.set_current_thread(agent_thread.clone());
            unimplemented!();
            let mut jvmti = get_jvmti_interface(jvm);
            let mut jni_env = get_interface(jvm);
            jvm.jvmti_state.as_ref().unwrap().built_in_jdwp.thread_start(jvm,thread_object);
            proc_.unwrap()(&mut jvmti, transmute(&mut jni_env), arg as *mut c_void)
        });
    jvm.tracing.trace_jdwp_function_exit(jvm, "RunAgentThread");
    jvmtiError_JVMTI_ERROR_NONE
}
