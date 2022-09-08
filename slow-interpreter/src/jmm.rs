use jvmti_jni_bindings::{jint, jlong, JMM_VERSION_1_2_2, jmmLongAttribute, jmmLongAttribute_JMM_CLASS_INIT_TOTAL_COUNT, jmmLongAttribute_JMM_CLASS_INIT_TOTAL_TIME_MS, jmmLongAttribute_JMM_CLASS_LOADED_BYTES, jmmLongAttribute_JMM_CLASS_LOADED_COUNT, jmmLongAttribute_JMM_CLASS_UNLOADED_BYTES, jmmLongAttribute_JMM_CLASS_UNLOADED_COUNT, jmmLongAttribute_JMM_CLASS_VERIFY_TOTAL_TIME_MS, jmmLongAttribute_JMM_COMPILE_TOTAL_TIME_MS, jmmLongAttribute_JMM_GC_COUNT, jmmLongAttribute_JMM_GC_EXT_ATTRIBUTE_INFO_SIZE, jmmLongAttribute_JMM_GC_TIME_MS, jmmLongAttribute_JMM_INTERNAL_ATTRIBUTE_INDEX, jmmLongAttribute_JMM_JVM_INIT_DONE_TIME_MS, jmmLongAttribute_JMM_JVM_UPTIME_MS, jmmLongAttribute_JMM_METHOD_DATA_SIZE_BYTES, jmmLongAttribute_JMM_OS_ATTRIBUTE_INDEX, jmmLongAttribute_JMM_OS_MEM_TOTAL_PHYSICAL_BYTES, jmmLongAttribute_JMM_OS_PROCESS_ID, jmmLongAttribute_JMM_SAFEPOINT_COUNT, jmmLongAttribute_JMM_SHARED_CLASS_LOADED_BYTES, jmmLongAttribute_JMM_SHARED_CLASS_LOADED_COUNT, jmmLongAttribute_JMM_SHARED_CLASS_UNLOADED_BYTES, jmmLongAttribute_JMM_SHARED_CLASS_UNLOADED_COUNT, jmmLongAttribute_JMM_THREAD_DAEMON_COUNT, jmmLongAttribute_JMM_THREAD_LIVE_COUNT, jmmLongAttribute_JMM_THREAD_PEAK_COUNT, jmmLongAttribute_JMM_THREAD_TOTAL_COUNT, jmmLongAttribute_JMM_TOTAL_APP_TIME_MS, jmmLongAttribute_JMM_TOTAL_CLASSLOAD_TIME_MS, jmmLongAttribute_JMM_TOTAL_SAFEPOINTSYNC_TIME_MS, jmmLongAttribute_JMM_TOTAL_STOPPED_TIME_MS, jmmLongAttribute_JMM_VM_GLOBAL_COUNT, jmmLongAttribute_JMM_VM_THREAD_COUNT, jmmOptionalSupport, JNI_OK, JNIEnv, jobject, jobjectArray};

use crate::rust_jni::interface::jni::{get_interpreter_state, get_state};
use crate::rust_jni::interface::local_frame::new_local_ref_public_new;

pub unsafe extern "C" fn get_version(env: *mut JNIEnv) -> jint {
    JMM_VERSION_1_2_2 as i32
}

pub unsafe extern "C" fn get_optional_support(env: *mut JNIEnv, support_ptr: *mut jmmOptionalSupport) -> jint {
    support_ptr.write(jmmOptionalSupport { _bitfield_align_1: [], _bitfield_1: Default::default() });
    JNI_OK as i32
}

#[allow(non_upper_case_globals)]
pub unsafe extern "C" fn get_long_attribute(env: *mut JNIEnv, obj: jobject, att: jmmLongAttribute) -> jlong {
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

pub unsafe extern "C" fn get_input_argument_array(env: *mut JNIEnv) -> jobjectArray {
    let jvm = get_state(env as *mut JNIEnv);
    let int_state = get_interpreter_state(env as *mut jvmti_jni_bindings::JNIEnv);
    new_local_ref_public_new(Some(jvm.local_var_array.get().unwrap().as_allocated_obj()), todo!()/*int_state*/) as jobject
}

