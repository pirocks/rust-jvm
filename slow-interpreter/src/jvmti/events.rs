use jvmti_jni_bindings::*;
use crate::jvmti::get_state;
use std::ops::Deref;
use crate::jvmti::event_callbacks::DebuggerEventConsumer;


pub unsafe extern "C" fn set_event_notification_mode(env: *mut jvmtiEnv, mode: jvmtiEventMode, event_type: jvmtiEvent, event_thread: jthread,
    ...) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm,"SetEventNotificationMode");
    let jdwp_copy = jvm.jvmti_state.built_in_jdwp.clone();
    let res= match event_type {
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
        62 => {//jvmtiEvent_JVMTI_EVENT_BREAKPOINT
            match mode{
                0 => jdwp_copy.deref().Breakpoint_disable(),
                1 => jdwp_copy.deref().Breakpoint_enable(),
                _ => unimplemented!()
            }
            jvmtiError_JVMTI_ERROR_NONE
        }
        _ => {
            dbg!(event_type);
            unimplemented!();
        }
    };
    jvm.tracing.trace_jdwp_function_exit(jvm,"SetEventNotificationMode");//todo maybe there should be a macro or similar layer for this so that I don't have early return issues
    res
}

