use jvmti_bindings::{jvmtiEnv, jvmtiEventMode, jvmtiEvent, jthread, jvmtiError, jvmtiError_JVMTI_ERROR_NONE, jvmtiEventCallbacks};
use crate::jvmti::{get_state, DebuggerEventConsumer};
use jni_bindings::jint;
use std::mem::size_of;
use std::ops::Deref;


pub unsafe extern "C" fn set_event_notification_mode(
    env: *mut jvmtiEnv,
    mode: jvmtiEventMode,
    event_type: jvmtiEvent,
    event_thread: jthread,
    ...) -> jvmtiError {
    let state = get_state(env);
    let jdwp_copy = state.built_in_jdwp.clone();
    match event_type {
        51 => {//jvmtiEvent_JVMTI_EVENT_VM_DEATH
            //todo, for now we do nothing b/c its not like this vm is ever going to die in a non-crash manner
            jvmtiError_JVMTI_ERROR_NONE
        }
        50 => {//jvmtiEvent_JVMTI_EVENT_VM_INIT
            match mode {
                0 => jdwp_copy.deref().VMInit_disable(),//todo figure out why jvmtiEventMode_JVMTI_DISABLE causes warnings
                1 => jdwp_copy.deref().VMInit_enable(),
                _ => unimplemented!()
            }
            jvmtiError_JVMTI_ERROR_NONE
        }
        58 => {//jvmtiEvent_JVMTI_EVENT_EXCEPTION
            match mode {
                0 => jdwp_copy.deref().Exception_disable(),
                1 => jdwp_copy.deref().Exception_enable(),
                _ => unimplemented!()
            }
            jvmtiError_JVMTI_ERROR_NONE
        }
        52 => {//jvmtiEvent_JVMTI_EVENT_THREAD_START
            match mode {
                0 => jdwp_copy.deref().ThreadStart_disable(),
                1 => jdwp_copy.deref().ThreadStart_enable(),
                _ => unimplemented!()
            }
            jvmtiError_JVMTI_ERROR_NONE
        }
        53 => {//jvmtiEvent_JVMTI_EVENT_THREAD_END
            match mode {
                0 => jdwp_copy.deref().ThreadEnd_disable(),
                1 => jdwp_copy.deref().ThreadEnd_enable(),
                _ => unimplemented!()
            }
            jvmtiError_JVMTI_ERROR_NONE
        }
        56 => {//jvmtiEvent_JVMTI_EVENT_CLASS_PREPARE
            match mode {
                0 => jdwp_copy.deref().ThreadEnd_disable(),
                1 => jdwp_copy.deref().ThreadEnd_enable(),
                _ => unimplemented!()
            }
            jvmtiError_JVMTI_ERROR_NONE
        }
        82 => {//jvmtiEvent_JVMTI_EVENT_GARBAGE_COLLECTION_FINISH
            match mode {
                0 => jdwp_copy.deref().GarbageCollectionFinish_disable(),
                1 => jdwp_copy.deref().GarbageCollectionFinish_enable(),
                _ => unimplemented!()
            }
            jvmtiError_JVMTI_ERROR_NONE
        }
        _ => {
            dbg!(event_type);
            unimplemented!();
        }
    }
}


#[allow(non_snake_case)]
pub unsafe extern "C" fn set_event_callbacks(env: *mut jvmtiEnv, callbacks: *const jvmtiEventCallbacks, _size_of_callbacks: jint) -> jvmtiError {
    //todo use size_of_callbacks ?
    let state = get_state(env);
    let mut callback_copy = jvmtiEventCallbacks{
        VMInit: None,
        VMDeath: None,
        ThreadStart: None,
        ThreadEnd: None,
        ClassFileLoadHook: None,
        ClassLoad: None,
        ClassPrepare: None,
        VMStart: None,
        Exception: None,
        ExceptionCatch: None,
        SingleStep: None,
        FramePop: None,
        Breakpoint: None,
        FieldAccess: None,
        FieldModification: None,
        MethodEntry: None,
        MethodExit: None,
        NativeMethodBind: None,
        CompiledMethodLoad: None,
        CompiledMethodUnload: None,
        DynamicCodeGenerated: None,
        DataDumpRequest: None,
        reserved72: None,
        MonitorWait: None,
        MonitorWaited: None,
        MonitorContendedEnter: None,
        MonitorContendedEntered: None,
        reserved77: None,
        reserved78: None,
        reserved79: None,
        ResourceExhausted: None,
        GarbageCollectionStart: None,
        GarbageCollectionFinish: None,
        ObjectFree: None,
        VMObjectAlloc: None
    };
    libc::memcpy(&mut callback_copy as *mut jvmtiEventCallbacks as *mut libc::c_void,callbacks as *const libc::c_void,size_of::<jvmtiEventCallbacks>());
    let jvmtiEventCallbacks {
        VMInit,
        VMDeath,
        ThreadStart,
        ThreadEnd,
        ClassFileLoadHook,
        ClassLoad,
        ClassPrepare,
        VMStart,
        Exception,
        ExceptionCatch,
        SingleStep,
        FramePop,
        Breakpoint,
        FieldAccess,
        FieldModification,
        MethodEntry,
        MethodExit,
        NativeMethodBind,
        CompiledMethodLoad,
        CompiledMethodUnload,
        DynamicCodeGenerated,
        DataDumpRequest,
        reserved72,
        MonitorWait,
        MonitorWaited,
        MonitorContendedEnter,
        MonitorContendedEntered,
        reserved77:_,
        reserved78:_,
        reserved79:_,
        ResourceExhausted,
        GarbageCollectionStart,
        GarbageCollectionFinish,
        ObjectFree,
        VMObjectAlloc
    } = callback_copy;

    if VMInit.is_some(){
        *state.built_in_jdwp.vm_init_callback.write().unwrap() = VMInit;
    }
    if VMDeath.is_some(){
        *state.built_in_jdwp.vm_death_callback.write().unwrap() = VMDeath;
    }
    if ThreadStart.is_some(){
        *state.built_in_jdwp.thread_start_callback.write().unwrap() = ThreadStart;
    }
    if ThreadEnd.is_some(){
        *state.built_in_jdwp.thread_end_callback.write().unwrap() = ThreadEnd;
    }
    if ClassFileLoadHook.is_some(){
        unimplemented!()
    }
    if ClassLoad.is_some(){
        *state.built_in_jdwp.class_load_callback.write().unwrap() = ClassLoad;
    }
    if ClassPrepare.is_some(){
        *state.built_in_jdwp.class_prepare_callback.write().unwrap() = ClassPrepare;
    }
    if VMStart.is_some(){
        unimplemented!()
    }
    if Exception.is_some(){
        *state.built_in_jdwp.exception_callback.write().unwrap() = Exception;
    }
    if ExceptionCatch.is_some(){
        *state.built_in_jdwp.exception_catch_callback.write().unwrap() = ExceptionCatch;
    }
    if SingleStep.is_some(){
        *state.built_in_jdwp.single_step_callback.write().unwrap() = SingleStep;
    }
    if FramePop.is_some(){
        *state.built_in_jdwp.frame_pop_callback.write().unwrap() = FramePop;
    }
    if Breakpoint.is_some(){
        *state.built_in_jdwp.breakpoint_callback.write().unwrap() = Breakpoint;
    }
    if FieldAccess.is_some(){
        *state.built_in_jdwp.field_access_callback.write().unwrap() = FieldAccess;
    }
    if FieldModification.is_some(){
        *state.built_in_jdwp.field_modification_callback.write().unwrap() = FieldModification;
    }
    if MethodEntry.is_some(){
        *state.built_in_jdwp.method_entry_callback.write().unwrap() = MethodEntry;
    }
    if MethodExit.is_some(){
        *state.built_in_jdwp.method_exit_callback.write().unwrap() = MethodExit;
    }
    if NativeMethodBind.is_some(){
        unimplemented!()
    }
    if CompiledMethodLoad.is_some(){
        unimplemented!()
    }
    if CompiledMethodUnload.is_some(){
        unimplemented!()
    }
    if DynamicCodeGenerated.is_some(){
        unimplemented!()
    }
    if DataDumpRequest.is_some(){
        unimplemented!()
    }
    if reserved72.is_some(){
        unimplemented!()
    }
    if MonitorWait.is_some(){
        *state.built_in_jdwp.monitor_wait_callback.write().unwrap() = MonitorWait;
    }
    if MonitorWaited.is_some(){
        *state.built_in_jdwp.monitor_waited_callback.write().unwrap() = MonitorWaited;
    }
    if MonitorContendedEnter.is_some(){
        *state.built_in_jdwp.monitor_conteded_enter_callback.write().unwrap() = MonitorContendedEnter;
    }
    if MonitorContendedEntered.is_some(){
        *state.built_in_jdwp.monitor_conteded_entered_callback.write().unwrap() = MonitorContendedEntered;
    }
    if ResourceExhausted.is_some(){
        unimplemented!()
    }
    if GarbageCollectionStart.is_some(){
        unimplemented!()
    }
    if GarbageCollectionFinish.is_some(){
        *state.built_in_jdwp.garbage_collection_finish_callback.write().unwrap() = GarbageCollectionFinish;
    }
    if ObjectFree.is_some(){
        unimplemented!()
    }
    if VMObjectAlloc.is_some(){
        unimplemented!()
    }
    jvmtiError_JVMTI_ERROR_NONE
}