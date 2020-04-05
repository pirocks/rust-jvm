use jvmti_bindings::{jvmtiEnv, jvmtiError, jvmtiError_JVMTI_ERROR_NOT_AVAILABLE};

pub unsafe extern "C" fn get_system_property(
    env: *mut jvmtiEnv,
    property: *const ::std::os::raw::c_char,
    value_ptr: *mut *mut ::std::os::raw::c_char
) -> jvmtiError{
    jvmtiError_JVMTI_ERROR_NOT_AVAILABLE
}