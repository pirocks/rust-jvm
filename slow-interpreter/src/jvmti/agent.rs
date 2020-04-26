use jvmti_bindings::{jvmtiEnv, jthread, jvmtiStartFunction, jint, jvmtiError, _jobject, jvmtiError_JVMTI_ERROR_NONE};
use crate::jvmti::{get_state, get_jvmti_interface};
use crate::interpreter_util::check_inited_class;
use rust_jvm_common::classnames::ClassName;
use crate::{JavaThread, InterpreterState, SuspendedStatus};
use std::sync::{Arc, RwLock};
use crate::rust_jni::interface::get_interface;
use std::mem::transmute;
use std::os::raw::c_void;
use crate::rust_jni::native_util::from_object;
use crate::java_values::JavaValue;
use std::cell::RefCell;
use crate::stack_entry::StackEntry;
use std::rc::Rc;
use lock_api::Mutex;

struct ThreadArgWrapper {
    proc_: jvmtiStartFunction,
    arg: *const ::std::os::raw::c_void,
    thread: *mut _jobject,
}

unsafe impl Send for ThreadArgWrapper {}

unsafe impl Sync for ThreadArgWrapper {}

pub unsafe extern "C" fn run_agent_thread(env: *mut jvmtiEnv, thread: jthread, proc_: jvmtiStartFunction, arg: *const ::std::os::raw::c_void, _priority: jint) -> jvmtiError {
    //todo implement thread priority
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm,"RunAgentThread");
    let args = ThreadArgWrapper { proc_, arg, thread };
    let system_class = check_inited_class(jvm, &ClassName::system(), jvm.bootstrap_loader.clone());
//TODO ADD THREAD TO JVM STATE STRUCT
    std::thread::spawn(move || {
        let ThreadArgWrapper { proc_, arg, thread } = args;
        let thread_object = JavaValue::Object(from_object(transmute(thread))).cast_thread();
        let agent_thread = Arc::new(JavaThread {
            java_tid: thread_object.tid(),
// name: "agent thread".to_string(),
            call_stack: RefCell::new(vec![Rc::new(StackEntry {
                class_pointer: system_class.clone(),
                method_i: std::u16::MAX,
                local_vars: RefCell::new(vec![]),
                operand_stack: RefCell::new(vec![]),
                pc: RefCell::new(std::usize::MAX),
                pc_offset: RefCell::new(-1),
            })]),
            thread_object: RefCell::new(thread_object.into()),
            interpreter_state: InterpreterState {
                terminate: RefCell::new(false),
                throw: RefCell::new(None),
                function_return: RefCell::new(false),
                suspended: RwLock::new(SuspendedStatus {
                    suspended: false,
                    suspended_lock: Mutex::new(())
                })
            },
        });
        // let result = jvm.thread_state.alive_threads.write();
        // result.unwrap().insert(agent_thread.java_tid, agent_thread.clone());//todo needs to be done via JavaThread constructor
        // todo this isn't strictly a java thread so not alive?

        jvm.set_current_thread(agent_thread.clone());
        let mut jvmti = get_jvmti_interface(jvm);
        let mut jni_env = get_interface(jvm);
        proc_.unwrap()(&mut jvmti, transmute(&mut jni_env), arg as *mut c_void)
    });
    jvm.tracing.trace_jdwp_function_exit(jvm,"RunAgentThread");
    jvmtiError_JVMTI_ERROR_NONE
}
