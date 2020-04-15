use jvmti_bindings::{jvmtiEnv, jthread, jvmtiStartFunction, jint, jvmtiError, _jobject, jvmtiError_JVMTI_ERROR_NONE};
use crate::jvmti::{get_state, get_jvmti_interface};
use crate::interpreter_util::check_inited_class;
use rust_jvm_common::classnames::ClassName;
use crate::{JavaThread, InterpreterState};
use std::sync::Arc;
use crate::rust_jni::interface::get_interface;
use std::mem::transmute;
use std::os::raw::c_void;
use crate::rust_jni::native_util::from_object;
use crate::java_values::JavaValue;
use std::cell::RefCell;
use crate::stack_entry::StackEntry;
use std::rc::Rc;

struct ThreadArgWrapper {
    proc_: jvmtiStartFunction,
    arg: *const ::std::os::raw::c_void,
    thread: *mut _jobject,
}

unsafe impl Send for ThreadArgWrapper {}

unsafe impl Sync for ThreadArgWrapper {}

pub unsafe extern "C" fn run_agent_thread(env: *mut jvmtiEnv, thread: jthread, proc_: jvmtiStartFunction, arg: *const ::std::os::raw::c_void, priority: jint) -> jvmtiError {
    let jvm = get_state(env);
    let args = ThreadArgWrapper { proc_, arg, thread };
    let system_class = check_inited_class(jvm, &ClassName::system(), jvm.bootstrap_loader.clone());
//TODO ADD THREAD TO JVM STATE STRUCT
    std::thread::spawn(move || {
        let ThreadArgWrapper { proc_, arg, thread } = args;
//unsafe extern "C" fn(jvmti_env: *mut jvmtiEnv, jni_env: *mut JNIEnv, arg: *mut ::std::os::raw::c_void)
        let agent_thread = Arc::new(JavaThread {
            java_tid: 1,
// name: "agent thread".to_string(),
            call_stack: RefCell::new(vec![Rc::new(StackEntry {
                class_pointer: system_class.clone(),
                method_i: std::u16::MAX,
                local_vars: RefCell::new(vec![]),
                operand_stack: RefCell::new(vec![]),
                pc: RefCell::new(std::usize::MAX),
                pc_offset: RefCell::new(-1),
            })]),
            thread_object: RefCell::new(JavaValue::Object(from_object(transmute(thread))).cast_thread().into()),
            interpreter_state: InterpreterState {
                terminate: RefCell::new(false),
                throw: RefCell::new(None),
                function_return: RefCell::new(false),
            },
        });
        jvm.alive_threads.write().unwrap().insert(agent_thread.java_tid, agent_thread.clone());//todo needs to be done via constructor
        jvm.set_current_thread(agent_thread.clone());
        let mut jvmti = get_jvmti_interface(jvm);
        let mut jni_env = get_interface(jvm);
        proc_.unwrap()(&mut jvmti, transmute(&mut jni_env), arg as *mut c_void)
    });
    jvmtiError_JVMTI_ERROR_NONE
}
