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
        _ => {
            dbg!(event_type);
            unimplemented!();
        }
    }
}


pub unsafe extern "C" fn set_event_callbacks(env: *mut jvmtiEnv, callbacks: *const jvmtiEventCallbacks, size_of_callbacks: jint) -> jvmtiError {
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
        reserved77,
        reserved78,
        reserved79,
        ResourceExhausted,
        GarbageCollectionStart,
        GarbageCollectionFinish,
        ObjectFree,
        VMObjectAlloc
    } = callback_copy;

    if VMInit.is_some(){
        state.built_in_jdwp.vm_init_callback.replace(VMInit);
    }
    if VMDeath.is_some(){
        state.built_in_jdwp.vm_death_callback.replace(VMDeath);
    }
    if ThreadStart.is_some(){
        unimplemented!()
    }
    if ThreadEnd.is_some(){
        unimplemented!()
    }
    if ClassFileLoadHook.is_some(){
        unimplemented!()
    }
    if ClassLoad.is_some(){
        unimplemented!()
    }
    if ClassPrepare.is_some(){
        unimplemented!()
    }
    if VMStart.is_some(){
        unimplemented!()
    }
    if Exception.is_some(){
        state.built_in_jdwp.exception_callback.replace(Exception);
    }
    if ExceptionCatch.is_some(){
        unimplemented!()
    }
    if SingleStep.is_some(){
        unimplemented!()
    }
    if FramePop.is_some(){
        unimplemented!()
    }
    if Breakpoint.is_some(){
        unimplemented!()
    }
    if FieldAccess.is_some(){
        unimplemented!()
    }
    if FieldModification.is_some(){
        unimplemented!()
    }
    if MethodEntry.is_some(){
        unimplemented!()
    }
    if MethodExit.is_some(){
        unimplemented!()
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
        unimplemented!()
    }
    if MonitorWaited.is_some(){
        unimplemented!()
    }
    if MonitorContendedEnter.is_some(){
        unimplemented!()
    }
    if MonitorContendedEntered.is_some(){
        unimplemented!()
    }
    if reserved77.is_some(){
        unimplemented!()
    }
    if reserved78.is_some(){
        unimplemented!()
    }
    if reserved79.is_some(){
        unimplemented!()
    }
    if ResourceExhausted.is_some(){
        unimplemented!()
    }
    if GarbageCollectionStart.is_some(){
        unimplemented!()
    }
    if GarbageCollectionFinish.is_some(){
        unimplemented!()
    }
    if ObjectFree.is_some(){
        unimplemented!()
    }
    jvmtiError_JVMTI_ERROR_NONE
}