use jvmti_jni_bindings::*;
use crate::rust_jni::jni_interface::jvmti::get_state;


pub const JVMTI_INTERFACE_MAJOR_VERSION: u32 = 1;
pub const JVMTI_INTERFACE_MINOR_VERSION: u32 = 2;

///
/// Get Version Number
//
//     jvmtiError
//     GetVersionNumber(jvmtiEnv* env,
//                 jint* version_ptr)
//
// Return the JVM TI version via version_ptr. The return value is the version identifier. The version identifier includes major, minor and micro version as well as the jni_interface type.
//
//     Version Interface Types
//     Constant 	Value 	Description
//     JVMTI_VERSION_INTERFACE_JNI	0x00000000	Value of JVMTI_VERSION_MASK_INTERFACE_TYPE for JNI.
//     JVMTI_VERSION_INTERFACE_JVMTI	0x30000000	Value of JVMTI_VERSION_MASK_INTERFACE_TYPE for JVM TI.
//
//     Version Masks
//     Constant 	Value 	Description
//     JVMTI_VERSION_MASK_INTERFACE_TYPE	0x70000000	Mask to extract jni_interface type. The value of the version returned by this function masked with JVMTI_VERSION_MASK_INTERFACE_TYPE is always JVMTI_VERSION_INTERFACE_JVMTI since this is a JVM TI function.
//     JVMTI_VERSION_MASK_MAJOR	0x0FFF0000	Mask to extract major version number.
//     JVMTI_VERSION_MASK_MINOR	0x0000FF00	Mask to extract minor version number.
//     JVMTI_VERSION_MASK_MICRO	0x000000FF	Mask to extract micro version number.
//
//     Version Shifts
//     Constant 	Value 	Description
//     JVMTI_VERSION_SHIFT_MAJOR	16	Shift to extract major version number.
//     JVMTI_VERSION_SHIFT_MINOR	8	Shift to extract minor version number.
//     JVMTI_VERSION_SHIFT_MICRO	0	Shift to extract micro version number.
//
// Phase	Callback Safe	Position	Since
// may be called during any phase 	No 	88	1.0
//
// Capabilities
// Required Functionality
//
// Parameters
// Name 	Type 	Description
// version_ptr	jint*	On return, points to the JVM TI version.
//
// Agent passes a pointer to a jint. On return, the jint has been set.
//
// Errors
// This function returns either a universal error or one of the following errors
// Error 	Description
// JVMTI_ERROR_NULL_POINTER	version_ptr is NULL.
pub unsafe extern "C" fn get_version_number(env: *mut jvmtiEnv, version_ptr: *mut jint) -> jvmtiError {
    let jvm = get_state(env);
    let tracing_guard = jvm.config.tracing.trace_jdwp_function_enter(jvm, "GetVersionNumber");
    null_check!(version_ptr);
    let version = (JVMTI_VERSION_INTERFACE_JVMTI | (JVMTI_VERSION_MASK_MAJOR & (JVMTI_INTERFACE_MAJOR_VERSION << 16)) | (JVMTI_VERSION_MASK_MINOR & (JVMTI_INTERFACE_MINOR_VERSION << 8))) as u32;
    version_ptr.write(version as jint);
    jvm.config.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}
