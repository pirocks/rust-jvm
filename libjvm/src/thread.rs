use slow_interpreter::interpreter_util::{run_function, push_new_object, check_inited_class};
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::Arc;
use rust_jvm_common::classnames::ClassName;
use jni_bindings::{JNIEnv, jclass, jobject, jlong, jint, jboolean, jobjectArray, jstring, jintArray};
use slow_interpreter::rust_jni::native_util::{get_state, get_frame, to_object};
use slow_interpreter::rust_jni::interface::util::runtime_class_from_object;
use slow_interpreter::java_values::{JavaValue, Object};
use slow_interpreter::JVMState;
use slow_interpreter::runtime_class::RuntimeClass;
use slow_interpreter::stack_entry::StackEntry;
use std::ops::Deref;

#[no_mangle]
unsafe extern "system" fn JVM_StartThread(env: *mut JNIEnv, thread: jobject) {
//    assert!(Arc::ptr_eq(MAIN_THREAD.as_ref().unwrap(),&from_object(thread).unwrap()));//todo why does this not pass?
    MAIN_ALIVE = true
}

#[no_mangle]
unsafe extern "system" fn JVM_StopThread(env: *mut JNIEnv, thread: jobject, exception: jobject) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_IsThreadAlive(env: *mut JNIEnv, thread: jobject) -> jboolean {
    MAIN_ALIVE as jboolean // todo we don't do threads atm.
}

#[no_mangle]
unsafe extern "system" fn JVM_SuspendThread(env: *mut JNIEnv, thread: jobject) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ResumeThread(env: *mut JNIEnv, thread: jobject) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_SetThreadPriority(env: *mut JNIEnv, thread: jobject, prio: jint) {
    //todo threads not implemented, noop
}

#[no_mangle]
unsafe extern "system" fn JVM_Yield(env: *mut JNIEnv, threadClass: jclass) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_Sleep(env: *mut JNIEnv, threadClass: jclass, millis: jlong) {
    unimplemented!()
}

static mut MAIN_THREAD: Option<Arc<Object>> = None;

#[no_mangle]
unsafe extern "system" fn JVM_CurrentThread(env: *mut JNIEnv, threadClass: jclass) -> jobject {
    match MAIN_THREAD.clone() {
        None => {
            let state = get_state(env);
            let frame = get_frame(env);
            let runtime_thread_class = runtime_class_from_object(threadClass, state, &frame).unwrap();
            make_thread(&runtime_thread_class, state, &frame);
            let thread_object = frame.pop().unwrap_object();
            MAIN_THREAD = thread_object.clone();
            to_object(thread_object)
        }
        Some(_) => {
            to_object(MAIN_THREAD.clone())
        }
    }
    //threads are not a thing atm.
    //todo
}


static mut SYSTEM_THREAD_GROUP: Option<Arc<Object>> = None;

fn init_system_thread_group(jvm: &JVMState, frame: &StackEntry) {
    let thread_group_class = check_inited_class(jvm, &ClassName::Str("java/lang/ThreadGroup".to_string()),  frame.class_pointer.loader.clone());
    push_new_object(jvm, frame.clone(), &thread_group_class);
    let object = frame.pop();
    let (init_i, init) = thread_group_class.classfile.lookup_method("<init>".to_string(), "()V".to_string()).unwrap();
    let new_frame = StackEntry {
        last_call_stack: frame.clone().into(),
        class_pointer: thread_group_class.clone(),
        method_i: init_i as u16,
        local_vars: RefCell::new(vec![object.clone()]),
        operand_stack: RefCell::new(vec![]),
        pc: RefCell::new(0),
        pc_offset: RefCell::new(0),
    };
    unsafe { SYSTEM_THREAD_GROUP = object.unwrap_object(); }
    run_function(jvm, Rc::new(new_frame));
    let interpreter_state = &jvm.get_current_thread().interpreter_state;
    if interpreter_state.throw.borrow().is_some() || *interpreter_state.terminate.borrow() {
        unimplemented!()
    }
    if *interpreter_state.function_return.borrow() {
        interpreter_state.function_return.replace(false);
    }
}

unsafe fn make_thread(runtime_thread_class: &Arc<RuntimeClass>, jvm: &JVMState, frame: &StackEntry) {
    //todo refactor this at some point
    //first create thread group
    let thread_group_object = match SYSTEM_THREAD_GROUP.clone() {
        None => {
            init_system_thread_group(jvm, frame);
            SYSTEM_THREAD_GROUP.clone()
        }
        Some(_) => SYSTEM_THREAD_GROUP.clone(),
    };


    let thread_class = check_inited_class(jvm, &ClassName::Str("java/lang/Thread".to_string()),  frame.class_pointer.loader.clone());
    if !Arc::ptr_eq(&thread_class, &runtime_thread_class) {
        frame.print_stack_trace();
    }
    assert!(Arc::ptr_eq(&thread_class, &runtime_thread_class));
    push_new_object(jvm, frame.clone(), &thread_class);
    let object = frame.pop();
    let (init_i, init) = thread_class.classfile.lookup_method("<init>".to_string(), "()V".to_string()).unwrap();
    let new_frame = StackEntry {
        last_call_stack: frame.clone().into(),
        class_pointer: thread_class.clone(),
        method_i: init_i as u16,
        local_vars: RefCell::new(vec![object.clone()]),
        operand_stack: RefCell::new(vec![]),
        pc: RefCell::new(0),
        pc_offset: RefCell::new(0),
    };
    MAIN_THREAD = object.unwrap_object().clone();
    MAIN_THREAD.clone().unwrap().unwrap_normal_object().fields.borrow_mut().insert("group".to_string(), JavaValue::Object(thread_group_object));
    //for some reason the constructor doesn't handle priority.
    let NORM_PRIORITY = 5;
    MAIN_THREAD.clone().unwrap().unwrap_normal_object().fields.borrow_mut().insert("priority".to_string(), JavaValue::Int(NORM_PRIORITY));
    run_function(jvm, Rc::new(new_frame));
    frame.push(JavaValue::Object(MAIN_THREAD.clone()));
    let interpreter_state = &jvm.get_current_thread().interpreter_state;
    if interpreter_state.throw.borrow().is_some() || *interpreter_state.terminate.borrow() {
        unimplemented!()
    }
    if *interpreter_state.function_return.borrow() {
        interpreter_state.function_return.replace(false);
    }
}

//todo this should prob go in InterperteerState or similar
static mut MAIN_ALIVE: bool = false;


#[no_mangle]
unsafe extern "system" fn JVM_Interrupt(env: *mut JNIEnv, thread: jobject) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_IsInterrupted(env: *mut JNIEnv, thread: jobject, clearInterrupted: jboolean) -> jboolean {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_HoldsLock(env: *mut JNIEnv, threadClass: jclass, obj: jobject) -> jboolean {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_DumpAllStacks(env: *mut JNIEnv, unused: jclass) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetAllThreads(env: *mut JNIEnv, dummy: jclass) -> jobjectArray {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_SetNativeThreadName(env: *mut JNIEnv, jthread: jobject, name: jstring) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_DumpThreads(env: *mut JNIEnv, threadClass: jclass, threads: jobjectArray) -> jobjectArray {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetThreadStateValues(env: *mut JNIEnv, javaThreadState: jint) -> jintArray {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetThreadStateNames(env: *mut JNIEnv, javaThreadState: jint, values: jintArray) -> jobjectArray {
    unimplemented!()
}
