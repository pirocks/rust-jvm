use jvmti_bindings::{jvmtiEnv, jvmtiError, jint, jvmtiError_JVMTI_ERROR_NONE};

pub unsafe extern "C" fn get_version_number(_env: *mut jvmtiEnv, version_ptr: *mut jint) -> jvmtiError{
    //JVMTI_VERSION_MASK_MAJOR	0x0FFF0000	Mask to extract major version number.
    // JVMTI_VERSION_MASK_MINOR	0x0000FF00	Mask to extract minor version number.
    // JVMTI_VERSION_MASK_MICRO	0x000000FF	Mask to extract micro version number.
    version_ptr.write(0x00010200 as jint);//34 is java major version in hex. Not quite sure which version number to use todo
    jvmtiError_JVMTI_ERROR_NONE
}