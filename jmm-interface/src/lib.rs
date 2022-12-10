#![feature(once_cell)]
#![allow(unused_variables)]
#![allow(unreachable_code)]

use std::ptr::null_mut;
use jvmti_jni_bindings::{jboolean, jint, jlong, JMM_VERSION_1_2_2, jmmBoolAttribute, jmmLongAttribute, jmmLongAttribute_JMM_CLASS_INIT_TOTAL_COUNT, jmmLongAttribute_JMM_CLASS_INIT_TOTAL_TIME_MS, jmmLongAttribute_JMM_CLASS_LOADED_BYTES, jmmLongAttribute_JMM_CLASS_LOADED_COUNT, jmmLongAttribute_JMM_CLASS_UNLOADED_BYTES, jmmLongAttribute_JMM_CLASS_UNLOADED_COUNT, jmmLongAttribute_JMM_CLASS_VERIFY_TOTAL_TIME_MS, jmmLongAttribute_JMM_COMPILE_TOTAL_TIME_MS, jmmLongAttribute_JMM_GC_COUNT, jmmLongAttribute_JMM_GC_EXT_ATTRIBUTE_INFO_SIZE, jmmLongAttribute_JMM_GC_TIME_MS, jmmLongAttribute_JMM_INTERNAL_ATTRIBUTE_INDEX, jmmLongAttribute_JMM_JVM_INIT_DONE_TIME_MS, jmmLongAttribute_JMM_JVM_UPTIME_MS, jmmLongAttribute_JMM_METHOD_DATA_SIZE_BYTES, jmmLongAttribute_JMM_OS_ATTRIBUTE_INDEX, jmmLongAttribute_JMM_OS_MEM_TOTAL_PHYSICAL_BYTES, jmmLongAttribute_JMM_OS_PROCESS_ID, jmmLongAttribute_JMM_SAFEPOINT_COUNT, jmmLongAttribute_JMM_SHARED_CLASS_LOADED_BYTES, jmmLongAttribute_JMM_SHARED_CLASS_LOADED_COUNT, jmmLongAttribute_JMM_SHARED_CLASS_UNLOADED_BYTES, jmmLongAttribute_JMM_SHARED_CLASS_UNLOADED_COUNT, jmmLongAttribute_JMM_THREAD_DAEMON_COUNT, jmmLongAttribute_JMM_THREAD_LIVE_COUNT, jmmLongAttribute_JMM_THREAD_PEAK_COUNT, jmmLongAttribute_JMM_THREAD_TOTAL_COUNT, jmmLongAttribute_JMM_TOTAL_APP_TIME_MS, jmmLongAttribute_JMM_TOTAL_CLASSLOAD_TIME_MS, jmmLongAttribute_JMM_TOTAL_SAFEPOINTSYNC_TIME_MS, jmmLongAttribute_JMM_TOTAL_STOPPED_TIME_MS, jmmLongAttribute_JMM_VM_GLOBAL_COUNT, jmmLongAttribute_JMM_VM_THREAD_COUNT, jmmOptionalSupport, JNI_OK, JNIEnv, jobject, jobjectArray};
use jvmti_jni_bindings::{jmmBoolAttribute_JMM_VERBOSE_GC, jmmBoolAttribute_JMM_VERBOSE_CLASS, jmmBoolAttribute_JMM_THREAD_CONTENTION_MONITORING, jmmBoolAttribute_JMM_THREAD_CPU_TIME, jmmBoolAttribute_JMM_THREAD_ALLOCATED_MEMORY};
use jvmti_jni_bindings::jmm_interface::JMMInterfaceNamedReservedPointers;

use slow_interpreter::rust_jni::jni_utils::{get_interpreter_state, get_state, new_local_ref_public_new};

pub unsafe extern "C" fn get_version(_env: *mut JNIEnv) -> jint {
    JMM_VERSION_1_2_2 as i32
}

pub unsafe extern "C" fn get_optional_support(_env: *mut JNIEnv, support_ptr: *mut jmmOptionalSupport) -> jint {
    support_ptr.write(jmmOptionalSupport { _bitfield_align_1: [], _bitfield_1: Default::default() });
    JNI_OK as i32
}

#[allow(non_upper_case_globals)]
pub unsafe extern "C" fn get_long_attribute(env: *mut JNIEnv, _obj: jobject, att: jmmLongAttribute) -> jlong {
    let jvm = get_state(env);
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
        jmmLongAttribute_JMM_SAFEPOINT_COUNT => {
            0
        },
        jmmLongAttribute_JMM_TOTAL_SAFEPOINTSYNC_TIME_MS => todo!(),
        jmmLongAttribute_JMM_TOTAL_STOPPED_TIME_MS => todo!(),
        jmmLongAttribute_JMM_TOTAL_APP_TIME_MS => todo!(),
        jmmLongAttribute_JMM_VM_THREAD_COUNT => todo!(),
        jmmLongAttribute_JMM_CLASS_INIT_TOTAL_COUNT => todo!(),
        jmmLongAttribute_JMM_CLASS_INIT_TOTAL_TIME_MS => todo!(),
        jmmLongAttribute_JMM_METHOD_DATA_SIZE_BYTES => todo!(),
        jmmLongAttribute_JMM_CLASS_VERIFY_TOTAL_TIME_MS => todo!(),
        jmmLongAttribute_JMM_SHARED_CLASS_LOADED_COUNT => todo!(),
        jmmLongAttribute_JMM_SHARED_CLASS_UNLOADED_COUNT => {
            0
        },
        jmmLongAttribute_JMM_SHARED_CLASS_LOADED_BYTES => todo!(),
        jmmLongAttribute_JMM_SHARED_CLASS_UNLOADED_BYTES => todo!(),
        jmmLongAttribute_JMM_OS_ATTRIBUTE_INDEX => todo!(),
        jmmLongAttribute_JMM_OS_PROCESS_ID => todo!(),
        jmmLongAttribute_JMM_OS_MEM_TOTAL_PHYSICAL_BYTES => todo!(),
        jmmLongAttribute_JMM_GC_EXT_ATTRIBUTE_INFO_SIZE => todo!(),
        _ => panic!(),
    }
}

#[allow(non_upper_case_globals)]
pub unsafe extern "C" fn get_bool_attribute(_env: *mut JNIEnv, att: jmmBoolAttribute) -> jboolean {
    match att {
        jmmBoolAttribute_JMM_VERBOSE_GC => false as jboolean,
        jmmBoolAttribute_JMM_VERBOSE_CLASS => false as jboolean,
        jmmBoolAttribute_JMM_THREAD_CONTENTION_MONITORING => false as jboolean,
        jmmBoolAttribute_JMM_THREAD_CPU_TIME => false as jboolean,
        jmmBoolAttribute_JMM_THREAD_ALLOCATED_MEMORY => false as jboolean,
        _ => {
            panic!()
        }
    }
}


pub unsafe extern "C" fn get_input_argument_array(env: *mut JNIEnv) -> jobjectArray {
    let jvm = get_state(env as *mut JNIEnv);
    let int_state = get_interpreter_state(env as *mut JNIEnv);
    new_local_ref_public_new(Some(jvm.program_args_array.get().unwrap().as_allocated_obj()), int_state) as jobject
}

pub fn initial_jmm() -> JMMInterfaceNamedReservedPointers {
    JMMInterfaceNamedReservedPointers {
        jvm_state: null_mut(),
        GetOneThreadAllocatedMemory: None,
        GetVersion: Some(get_version),
        GetOptionalSupport: Some(get_optional_support),
        GetInputArguments: None,
        GetThreadInfo: None,
        GetInputArgumentArray: Some(get_input_argument_array),
        GetMemoryPools: None,
        GetMemoryManagers: None,
        GetMemoryPoolUsage: None,
        GetPeakMemoryPoolUsage: None,
        GetThreadAllocatedMemory: None,
        GetMemoryUsage: None,
        GetLongAttribute: Some(get_long_attribute),
        GetBoolAttribute: Some(get_bool_attribute),
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
