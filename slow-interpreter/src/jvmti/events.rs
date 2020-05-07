use jvmti_jni_bindings::*;
use crate::jvmti::get_state;
use std::ops::Deref;
use crate::jvmti::event_callbacks::DebuggerEventConsumer;
use crate::rust_jni::native_util::from_object;
use crate::java_values::JavaValue;
use crate::ThreadId;


pub unsafe extern "C" fn set_event_notification_mode(env: *mut jvmtiEnv, mode: jvmtiEventMode, event_type: jvmtiEvent, event_thread: jthread, ...) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm, "SetEventNotificationMode");
    let thread_obj = if event_thread.is_null() {
        None
    } else {
        JavaValue::Object(from_object(event_thread)).cast_thread().into()
    };
    let tid:Option<ThreadId> = thread_obj.map(|it| it.tid());
    // let vm_thread = tid.map(|it|jvm.thread_state.alive_threads.read().unwrap().get(&it).unwrap());
    let jdwp_copy = jvm.jvmti_state.built_in_jdwp.clone();
    // does not support per thread notification
    // VMInit
    // VMStart
    // VMDeath
    // ThreadStart
    // CompiledMethodLoad
    // CompiledMethodUnload
    // DynamicCodeGenerated
    // DataDumpRequest

    let res = match event_type {
        51 => {//jvmtiEvent_JVMTI_EVENT_VM_DEATH
            if tid.is_some(){
                return jvmtiError_JVMTI_ERROR_ILLEGAL_ARGUMENT//can't do jvminit on a per thread basis as per spec
            }
            //todo, for now we do nothing b/c its not like this vm is ever going to die in a non-crash manner
            jvmtiError_JVMTI_ERROR_NONE
        }
        50 => {//jvmtiEvent_JVMTI_EVENT_VM_INIT
            if tid.is_some(){
                return jvmtiError_JVMTI_ERROR_ILLEGAL_ARGUMENT//can't do jvminit on a per thread basis as per spec
            }
            match mode {
                0 => jdwp_copy.deref().VMInit_disable(&jvm.tracing),//todo figure out why jvmtiEventMode_JVMTI_DISABLE causes warnings
                1 => jdwp_copy.deref().VMInit_enable(&jvm.tracing),
                _ => unimplemented!()
            }
            jvmtiError_JVMTI_ERROR_NONE
        }
        58 => {//jvmtiEvent_JVMTI_EVENT_EXCEPTION
            if tid.is_some(){
                unimplemented!()
            }
            match mode {
                0 => jdwp_copy.deref().Exception_disable(&jvm.tracing),
                1 => jdwp_copy.deref().Exception_enable(&jvm.tracing),
                _ => unimplemented!()
            }
            jvmtiError_JVMTI_ERROR_NONE
        }
        52 => {//jvmtiEvent_JVMTI_EVENT_THREAD_START

            if tid.is_some(){
                return jvmtiError_JVMTI_ERROR_ILLEGAL_ARGUMENT//can't do jvminit on a per thread basis as per spec
            }
            match mode {
                0 => jdwp_copy.deref().ThreadStart_disable(&jvm.tracing),
                1 => jdwp_copy.deref().ThreadStart_enable(&jvm.tracing),
                _ => unimplemented!()
            }
            jvmtiError_JVMTI_ERROR_NONE
        }
        53 => {//jvmtiEvent_JVMTI_EVENT_THREAD_END
            if tid.is_some(){
                unimplemented!()
            }
            match mode {
                0 => jdwp_copy.deref().ThreadEnd_disable(&jvm.tracing),
                1 => jdwp_copy.deref().ThreadEnd_enable(&jvm.tracing),
                _ => unimplemented!()
            }
            jvmtiError_JVMTI_ERROR_NONE
        }
        56 => {//jvmtiEvent_JVMTI_EVENT_CLASS_PREPARE
            if tid.is_some(){
                unimplemented!()
            }
            match mode {
                0 => jdwp_copy.deref().ClassPrepare_disable(&jvm.tracing),
                1 => jdwp_copy.deref().ClassPrepare_enable(&jvm.tracing),
                _ => unimplemented!()
            }
            jvmtiError_JVMTI_ERROR_NONE
        }
        82 => {//jvmtiEvent_JVMTI_EVENT_GARBAGE_COLLECTION_FINISH
            if tid.is_some(){
                unimplemented!()
            }
            match mode {
                0 => jdwp_copy.deref().GarbageCollectionFinish_disable(&jvm.tracing),
                1 => jdwp_copy.deref().GarbageCollectionFinish_enable(&jvm.tracing),
                _ => unimplemented!()
            }
            jvmtiError_JVMTI_ERROR_NONE
        }
        62 => {//jvmtiEvent_JVMTI_EVENT_BREAKPOINT
            if tid.is_some(){
                unimplemented!()
            }
            match mode {
                0 => jdwp_copy.deref().Breakpoint_disable(&jvm.tracing),
                1 => jdwp_copy.deref().Breakpoint_enable(&jvm.tracing),
                _ => unimplemented!()
            }
            jvmtiError_JVMTI_ERROR_NONE
        }
        _ => {
            dbg!(event_type);
            unimplemented!();
        }
    };
    jvm.tracing.trace_jdwp_function_exit(jvm, "SetEventNotificationMode");//todo maybe there should be a macro or similar layer for this so that I don't have early return issues
    res
}

