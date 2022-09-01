use std::ptr::null_mut;

use jvmti_jni_bindings::{JMM_VERSION_1_2_2, jmmInterface_1_, jmmLongAttribute, jmmLongAttribute_JMM_CLASS_INIT_TOTAL_COUNT, jmmLongAttribute_JMM_CLASS_INIT_TOTAL_TIME_MS, jmmLongAttribute_JMM_CLASS_LOADED_BYTES, jmmLongAttribute_JMM_CLASS_LOADED_COUNT, jmmLongAttribute_JMM_CLASS_UNLOADED_BYTES, jmmLongAttribute_JMM_CLASS_UNLOADED_COUNT, jmmLongAttribute_JMM_CLASS_VERIFY_TOTAL_TIME_MS, jmmLongAttribute_JMM_COMPILE_TOTAL_TIME_MS, jmmLongAttribute_JMM_GC_COUNT, jmmLongAttribute_JMM_GC_EXT_ATTRIBUTE_INFO_SIZE, jmmLongAttribute_JMM_GC_TIME_MS, jmmLongAttribute_JMM_INTERNAL_ATTRIBUTE_INDEX, jmmLongAttribute_JMM_JVM_INIT_DONE_TIME_MS, jmmLongAttribute_JMM_JVM_UPTIME_MS, jmmLongAttribute_JMM_METHOD_DATA_SIZE_BYTES, jmmLongAttribute_JMM_OS_ATTRIBUTE_INDEX, jmmLongAttribute_JMM_OS_MEM_TOTAL_PHYSICAL_BYTES, jmmLongAttribute_JMM_OS_PROCESS_ID, jmmLongAttribute_JMM_SAFEPOINT_COUNT, jmmLongAttribute_JMM_SHARED_CLASS_LOADED_BYTES, jmmLongAttribute_JMM_SHARED_CLASS_LOADED_COUNT, jmmLongAttribute_JMM_SHARED_CLASS_UNLOADED_BYTES, jmmLongAttribute_JMM_SHARED_CLASS_UNLOADED_COUNT, jmmLongAttribute_JMM_THREAD_DAEMON_COUNT, jmmLongAttribute_JMM_THREAD_LIVE_COUNT, jmmLongAttribute_JMM_THREAD_PEAK_COUNT, jmmLongAttribute_JMM_THREAD_TOTAL_COUNT, jmmLongAttribute_JMM_TOTAL_APP_TIME_MS, jmmLongAttribute_JMM_TOTAL_CLASSLOAD_TIME_MS, jmmLongAttribute_JMM_TOTAL_SAFEPOINTSYNC_TIME_MS, jmmLongAttribute_JMM_TOTAL_STOPPED_TIME_MS, jmmLongAttribute_JMM_VM_GLOBAL_COUNT, jmmLongAttribute_JMM_VM_THREAD_COUNT, jmmOptionalSupport, JNI_OK, JNIEnv, jobject, jobjectArray};
use jvmti_jni_bindings::{jint, jlong};
use slow_interpreter::rust_jni::interface::{get_interpreter_state, get_state};
use slow_interpreter::rust_jni::interface::local_frame::new_local_ref_public_new;

unsafe extern "C" fn get_version(env: *mut JNIEnv) -> jint {
    JMM_VERSION_1_2_2 as i32
}

unsafe extern "C" fn GetOptionalSupport(env: *mut JNIEnv, support_ptr: *mut jmmOptionalSupport) -> jint {
    support_ptr.write(jmmOptionalSupport { _bitfield_align_1: [], _bitfield_1: Default::default() });
    JNI_OK as i32
}

#[allow(non_camel_case_types)]
#[allow(non_upper_case_globals)]
unsafe extern "C" fn GetLongAttribute(env: *mut JNIEnv, obj: jobject, att: jmmLongAttribute) -> jlong {
    match att {
        jmmLongAttribute_JMM_CLASS_LOADED_COUNT => todo!(),
        jmmLongAttribute_JMM_CLASS_UNLOADED_COUNT => todo!(),
        jmmLongAttribute_JMM_THREAD_TOTAL_COUNT => todo!(),
        jmmLongAttribute_JMM_THREAD_LIVE_COUNT => todo!(),
        jmmLongAttribute_JMM_THREAD_PEAK_COUNT => todo!(),
        jmmLongAttribute_JMM_THREAD_DAEMON_COUNT => todo!(),
        jmmLongAttribute_JMM_JVM_INIT_DONE_TIME_MS => {
            1 //todo have accurate numbers here.
        }
        jmmLongAttribute_JMM_COMPILE_TOTAL_TIME_MS => todo!(),
        jmmLongAttribute_JMM_GC_TIME_MS => todo!(),
        jmmLongAttribute_JMM_GC_COUNT => todo!(),
        jmmLongAttribute_JMM_JVM_UPTIME_MS => todo!(),
        jmmLongAttribute_JMM_INTERNAL_ATTRIBUTE_INDEX => todo!(),
        jmmLongAttribute_JMM_CLASS_LOADED_BYTES => todo!(),
        jmmLongAttribute_JMM_CLASS_UNLOADED_BYTES => todo!(),
        jmmLongAttribute_JMM_TOTAL_CLASSLOAD_TIME_MS => todo!(),
        jmmLongAttribute_JMM_VM_GLOBAL_COUNT => todo!(),
        jmmLongAttribute_JMM_SAFEPOINT_COUNT => todo!(),
        jmmLongAttribute_JMM_TOTAL_SAFEPOINTSYNC_TIME_MS => todo!(),
        jmmLongAttribute_JMM_TOTAL_STOPPED_TIME_MS => todo!(),
        jmmLongAttribute_JMM_TOTAL_APP_TIME_MS => todo!(),
        jmmLongAttribute_JMM_VM_THREAD_COUNT => todo!(),
        jmmLongAttribute_JMM_CLASS_INIT_TOTAL_COUNT => todo!(),
        jmmLongAttribute_JMM_CLASS_INIT_TOTAL_TIME_MS => todo!(),
        jmmLongAttribute_JMM_METHOD_DATA_SIZE_BYTES => todo!(),
        jmmLongAttribute_JMM_CLASS_VERIFY_TOTAL_TIME_MS => todo!(),
        jmmLongAttribute_JMM_SHARED_CLASS_LOADED_COUNT => todo!(),
        jmmLongAttribute_JMM_SHARED_CLASS_UNLOADED_COUNT => todo!(),
        jmmLongAttribute_JMM_SHARED_CLASS_LOADED_BYTES => todo!(),
        jmmLongAttribute_JMM_SHARED_CLASS_UNLOADED_BYTES => todo!(),
        jmmLongAttribute_JMM_OS_ATTRIBUTE_INDEX => todo!(),
        jmmLongAttribute_JMM_OS_PROCESS_ID => todo!(),
        jmmLongAttribute_JMM_OS_MEM_TOTAL_PHYSICAL_BYTES => todo!(),
        jmmLongAttribute_JMM_GC_EXT_ATTRIBUTE_INFO_SIZE => todo!(),
        _ => panic!(),
    }
}

unsafe extern "C" fn GetInputArgumentArray(env: *mut JNIEnv) -> jobjectArray{
    let jvm = get_state(env as *mut jvmti_jni_bindings::JNIEnv);
    let int_state = get_interpreter_state(env as *mut jvmti_jni_bindings::JNIEnv);
    new_local_ref_public_new(Some(jvm.local_var_array.get().unwrap().as_allocated_obj()),todo!()/*int_state*/) as jobject
}

pub fn get_jmm_interface() -> jmmInterface_1_ {
    jmmInterface_1_ {
        reserved1: null_mut(),
        GetOneThreadAllocatedMemory: None,
        GetVersion: Some(get_version),
        GetOptionalSupport: Some(GetOptionalSupport),
        GetInputArguments: None,
        GetThreadInfo: None,
        GetInputArgumentArray: Some(GetInputArgumentArray),
        GetMemoryPools: None,
        GetMemoryManagers: None,
        GetMemoryPoolUsage: None,
        GetPeakMemoryPoolUsage: None,
        GetThreadAllocatedMemory: None,
        GetMemoryUsage: None,
        GetLongAttribute: Some(GetLongAttribute),
        GetBoolAttribute: None,
        SetBoolAttribute: None,
        GetLongAttributes: None,
        FindCircularBlockedThreads: None,
        GetThreadCpuTime: None,
        GetVMGlobalNames: None,
        GetVMGlobals: None,
        GetInternalThreadTimes: None,
        ResetStatistic: None,
        SetPoolSensor: None,
        SetPoolThreshold: None,
        GetPoolCollectionUsage: None,
        GetGCExtAttributeInfo: None,
        GetLastGCStat: None,
        GetThreadCpuTimeWithKind: None,
        GetThreadCpuTimesWithKind: None,
        DumpHeap0: None,
        FindDeadlocks: None,
        SetVMGlobal: None,
        DumpThreadsMaxDepth: None,
        DumpThreads: None,
        SetGCNotificationEnabled: None,
        GetDiagnosticCommands: None,
        GetDiagnosticCommandInfo: None,
        GetDiagnosticCommandArgumentsInfo: None,
        ExecuteDiagnosticCommand: None,
        SetDiagnosticFrameworkNotificationEnabled: None,
    }
}
