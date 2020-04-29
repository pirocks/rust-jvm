use jvmti_jni_bindings::{jvmtiEnv, jvmtiError, jvmtiError_JVMTI_ERROR_NOT_AVAILABLE, jvmtiError_JVMTI_ERROR_NONE};
use std::ffi::{CStr, CString};
use crate::jvmti::get_state;

pub unsafe extern "C" fn get_system_property(
    env: *mut jvmtiEnv,
    property: *const ::std::os::raw::c_char,
    value_ptr: *mut *mut ::std::os::raw::c_char
) -> jvmtiError{
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm,"GetSystemProperty");
    let property_name = CStr::from_ptr(property).to_str().unwrap();

    //apparently different from System.getProperty()?
    if property_name == "java.vm.vendor"{
        unimplemented!()
    }
    if property_name == "java.vm.version"{
        unimplemented!()
    }
    if property_name == "java.vm.name"{
        let leaked_name = CString::new("TODO: Get a better VM Name").unwrap().into_raw();//todo name and avoid all this leaking
        value_ptr.write(leaked_name);
        return  jvmtiError_JVMTI_ERROR_NONE
    }
    if property_name == "java.vm.info"{
        let leaked_name = CString::new("TODO: Get better VM Info").unwrap().into_raw();//todo
        value_ptr.write(leaked_name);
        return  jvmtiError_JVMTI_ERROR_NONE
    }
    if property_name == "java.library.path"{
        unimplemented!()
    }
    if property_name == "java.class.path" || property_name == "sun.boot.class.path"{
        let jvm = get_state(env);
        let leaked_str = CString::new(jvm.classpath.classpath_string()).unwrap().into_raw();
        value_ptr.write(leaked_str);
        return jvmtiError_JVMTI_ERROR_NONE//todo duplication
    }

    jvm.tracing.trace_jdwp_function_exit(jvm,"GetSystemProperty");
    jvmtiError_JVMTI_ERROR_NOT_AVAILABLE
}