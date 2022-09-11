use std::ptr::null_mut;

use jvmti_jni_bindings::jmmInterface_1_;

use crate::rust_jni::jmm_interface::{get_input_argument_array, get_long_attribute, get_optional_support, get_version};

pub fn initial_jmm() -> jmmInterface_1_ {
    jmmInterface_1_ {
        reserved1: null_mut(),
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
