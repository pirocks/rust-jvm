use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::ffi::c_void;
use std::ops::Deref;
use std::process::exit;
use std::ptr::null_mut;
use std::sync::RwLock;

use lazy_static::lazy_static;
use wtf8::Wtf8Buf;

use another_jit_vm_ir::WasException;
use jmm_bindings::jmmInterface_1_;
use jvmti_jni_bindings::{_jobject, jboolean, jint, JNIEnv, jobject, JVM_INTERFACE_VERSION, jvm_version_info};
use slow_interpreter::java::lang::string::JString;
use slow_interpreter::java::util::properties::Properties;
use slow_interpreter::java_values::JavaValue;
use slow_interpreter::rust_jni::native_util::{from_object, get_interpreter_state, get_state, to_object};

use crate::jvm_management::management_impl::get_jmm_interface;

#[no_mangle]
unsafe extern "system" fn JVM_GetInterfaceVersion() -> jint {
    JVM_INTERFACE_VERSION as jint
}

lazy_static! {
    static ref ON_EXIT: RwLock<Vec<Option<unsafe extern "C" fn()>>> = RwLock::new(Vec::new());
}

#[no_mangle]
unsafe extern "system" fn JVM_OnExit(func: Option<unsafe extern "C" fn()>) {
    ON_EXIT.write().unwrap().push(func);
}

#[no_mangle]
unsafe extern "system" fn JVM_Exit(code: jint) {
    //todo run finalizers blocking on gc
    for func in ON_EXIT.read().unwrap().iter() {
        if let Some(func) = func.as_ref() {
            func();
        };
    }
    exit(code);
}

#[no_mangle]
unsafe extern "system" fn JVM_Halt(code: jint) {
    exit(code);
    // halt means that no cleanup is desired
}

#[no_mangle]
unsafe extern "system" fn JVM_ActiveProcessorCount() -> jint {
    num_cpus::get() as jint
}

#[no_mangle]
unsafe extern "system" fn JVM_IsSupportedJNIVersion(version: jint) -> jboolean {
    //todo for now we support everything?
    true as jboolean
}

thread_local! {
    static JMM: RefCell<Option<*const jmmInterface_1_>> = RefCell::new(None)
}

#[no_mangle]
unsafe extern "system" fn JVM_GetManagement(version: jint) -> *mut ::std::os::raw::c_void {
    eprintln!("Attempt to get jmm which is unsupported");
    JMM.with(|refcell: &RefCell<Option<*const jmmInterface_1_>>| {
        if refcell.borrow().is_none() {
            *refcell.borrow_mut() = Some(Box::leak(box get_jmm_interface()));
        }
        *refcell.borrow().as_ref().unwrap()
    }) as *mut c_void
}

pub mod management_impl {
    use std::ptr::null_mut;

    use jmm_bindings::{JMM_VERSION_1_2_2, jmmInterface_1_, jmmLongAttribute, jmmLongAttribute_JMM_CLASS_INIT_TOTAL_COUNT, jmmLongAttribute_JMM_CLASS_INIT_TOTAL_TIME_MS, jmmLongAttribute_JMM_CLASS_LOADED_BYTES, jmmLongAttribute_JMM_CLASS_LOADED_COUNT, jmmLongAttribute_JMM_CLASS_UNLOADED_BYTES, jmmLongAttribute_JMM_CLASS_UNLOADED_COUNT, jmmLongAttribute_JMM_CLASS_VERIFY_TOTAL_TIME_MS, jmmLongAttribute_JMM_COMPILE_TOTAL_TIME_MS, jmmLongAttribute_JMM_GC_COUNT, jmmLongAttribute_JMM_GC_EXT_ATTRIBUTE_INFO_SIZE, jmmLongAttribute_JMM_GC_TIME_MS, jmmLongAttribute_JMM_INTERNAL_ATTRIBUTE_INDEX, jmmLongAttribute_JMM_JVM_INIT_DONE_TIME_MS, jmmLongAttribute_JMM_JVM_UPTIME_MS, jmmLongAttribute_JMM_METHOD_DATA_SIZE_BYTES, jmmLongAttribute_JMM_OS_ATTRIBUTE_INDEX, jmmLongAttribute_JMM_OS_MEM_TOTAL_PHYSICAL_BYTES, jmmLongAttribute_JMM_OS_PROCESS_ID, jmmLongAttribute_JMM_SAFEPOINT_COUNT, jmmLongAttribute_JMM_SHARED_CLASS_LOADED_BYTES, jmmLongAttribute_JMM_SHARED_CLASS_LOADED_COUNT, jmmLongAttribute_JMM_SHARED_CLASS_UNLOADED_BYTES, jmmLongAttribute_JMM_SHARED_CLASS_UNLOADED_COUNT, jmmLongAttribute_JMM_THREAD_DAEMON_COUNT, jmmLongAttribute_JMM_THREAD_LIVE_COUNT, jmmLongAttribute_JMM_THREAD_PEAK_COUNT, jmmLongAttribute_JMM_THREAD_TOTAL_COUNT, jmmLongAttribute_JMM_TOTAL_APP_TIME_MS, jmmLongAttribute_JMM_TOTAL_CLASSLOAD_TIME_MS, jmmLongAttribute_JMM_TOTAL_SAFEPOINTSYNC_TIME_MS, jmmLongAttribute_JMM_TOTAL_STOPPED_TIME_MS, jmmLongAttribute_JMM_VM_GLOBAL_COUNT, jmmLongAttribute_JMM_VM_THREAD_COUNT, jmmOptionalSupport, JNI_OK, JNIEnv, jobject, jobjectArray};
    use jvmti_jni_bindings::{jint, jlong};
    use slow_interpreter::rust_jni::interface::local_frame::new_local_ref_public_new;
    use slow_interpreter::rust_jni::native_util::{get_interpreter_state, get_state};

    unsafe extern "C" fn get_version(env: *mut JNIEnv) -> jint {
        JMM_VERSION_1_2_2 as i32
    }

    unsafe extern "C" fn GetOptionalSupport(env: *mut JNIEnv, support_ptr: *mut jmmOptionalSupport) -> jint {
        support_ptr.write(jmmOptionalSupport { _bitfield_1: Default::default() });
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
        new_local_ref_public_new(Some(jvm.local_var_array.get().unwrap().as_allocated_obj()),int_state) as jobject
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
}

#[no_mangle]
unsafe extern "system" fn JVM_InitAgentProperties(env: *mut JNIEnv, agent_props: jobject) -> jobject {
    match InitAgentProperties(env, agent_props) {
        Ok(res) => res,
        Err(_) => null_mut(),
    }
}

unsafe fn InitAgentProperties(env: *mut JNIEnv, agent_props: jobject) -> Result<jobject, WasException> {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let props = JavaValue::Object(todo!() /*from_jclass(jvm,agent_props)*/).cast_properties();

    let sun_java_command = JString::from_rust(jvm, int_state, Wtf8Buf::from_str("sun.java.command"))?;
    let sun_java_command_val = JString::from_rust(jvm, int_state, Wtf8Buf::from_str("command line not currently compatible todo"))?;
    props.set_property(jvm, int_state, sun_java_command, sun_java_command_val);

    let sun_java_command = JString::from_rust(jvm, int_state, Wtf8Buf::from_str("sun.jvm.flags"))?;
    let sun_java_command_val = JString::from_rust(jvm, int_state, Wtf8Buf::from_str("command line not currently compatible todo"))?;
    props.set_property(jvm, int_state, sun_java_command, sun_java_command_val);

    let sun_java_command = JString::from_rust(jvm, int_state, Wtf8Buf::from_str("sun.jvm.args"))?;
    let sun_java_command_val = JString::from_rust(jvm, int_state, Wtf8Buf::from_str("command line not currently compatible todo"))?;
    props.set_property(jvm, int_state, sun_java_command, sun_java_command_val);

    Ok(agent_props)
}

#[no_mangle]
unsafe extern "system" fn JVM_GetVersionInfo(env: *mut JNIEnv, info: *mut jvm_version_info, info_size: usize) {
    (*info).jvm_version = 8;
    (*info).set_is_attach_supported(0);
    (*info).set_update_version(0);
    (*info).set_special_update_version(0);
    (*info).set_reserved1(0);
}

#[no_mangle]
unsafe extern "system" fn JVM_SupportsCX8() -> jboolean {
    false as jboolean
}

#[no_mangle]
unsafe extern "system" fn JVM_BeforeHalt() {}