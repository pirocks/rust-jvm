use std::ffi::CStr;

use jvmti_jni_bindings::{
    jvmtiEnv, jvmtiError, jvmtiError_JVMTI_ERROR_NONE, jvmtiError_JVMTI_ERROR_NOT_AVAILABLE,
};

use crate::jvmti::get_state;

///Get System Property
///
///     jvmtiError
///     GetSystemProperty(jvmtiEnv* env,
///                 const char* property,
///                 char** value_ptr)
///
/// Return a VM system property value given the property key.
///
/// The function GetSystemProperties returns the set of property keys which may be used.
/// The properties which can be retrieved may grow during execution.
///
/// Since this is a VM view of system properties, the values of properties may differ from that returned by java.lang.System.getProperty(String).
/// A typical VM might copy the values of the VM system properties into the Properties held by java.lang.System during the initialization of that class.
/// Thereafter any changes to the VM system properties (with SetSystemProperty) or the java.lang.System system properties (with java.lang.System.setProperty(String,String)) would cause the values to diverge.
/// JNI method invocation may be used to access java.lang.System.getProperty(String).
///
/// Phase	Callback Safe	Position	Since
/// may only be called during the OnLoad or the live phase 	No 	131	1.0
///
/// Capabilities
/// Required Functionality
///
/// Parameters
/// Name 	Type 	Description
/// property	const char*	The key of the property to retrieve, encoded as a modified UTF-8 string.
///
/// Agent passes in an array of char.
/// value_ptr	char**	On return, points to the property value, encoded as a modified UTF-8 string.
///
/// Agent passes a pointer to a char*. On return, the char* points to a newly allocated array. The array should be freed with Deallocate.
///
/// Errors
/// This function returns either a universal error or one of the following errors
/// Error 	Description
/// JVMTI_ERROR_NOT_AVAILABLE	This property is not available. Use GetSystemProperties to find available properties.
/// JVMTI_ERROR_NULL_POINTER	property is NULL.
/// JVMTI_ERROR_NULL_POINTER	value_ptr is NULL.
pub unsafe extern "C" fn get_system_property(
    env: *mut jvmtiEnv,
    property: *const ::std::os::raw::c_char,
    value_ptr: *mut *mut ::std::os::raw::c_char,
) -> jvmtiError {
    let jvm = get_state(env);
    let tracing_guard = jvm
        .config
        .tracing
        .trace_jdwp_function_enter(jvm, "GetSystemProperty");
    null_check!(property);
    null_check!(value_ptr);
    //todo figure out how to assert OnLoad or live
    let property_name = CStr::from_ptr(property).to_str().unwrap();
    //apparently different from System.getProperty()?
    if property_name == "java.vm.vendor" {
        unimplemented!()
    }
    if property_name == "java.vm.version" {
        unimplemented!()
    }
    if property_name == "java.vm.name" {
        let leaked_name = jvm
            .native
            .native_interface_allocations
            .allocate_string("TODO: Get a better VM Name".to_string()); //todo get better name
        value_ptr.write(leaked_name);
        return jvm
            .config
            .tracing
            .trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE);
    }
    if property_name == "java.vm.info" {
        let leaked_name = jvm
            .native
            .native_interface_allocations
            .allocate_string("TODO: Get better VM Info".to_string()); //todo
        value_ptr.write(leaked_name);
        return jvm
            .config
            .tracing
            .trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE);
    }
    if property_name == "java.library.path" || property_name == "sun.boot.library.path" {
        let leaked_name = jvm.native.native_interface_allocations.allocate_string("/home/francis/build/openjdk-jdk8u/build/linux-x86_64-normal-server-release/jdk/lib/amd64/".to_string()); //todo in future don't hardcode this
        value_ptr.write(leaked_name);
        return jvm
            .config
            .tracing
            .trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE);
    }
    if property_name == "java.class.path" || property_name == "sun.boot.class.path" {
        let jvm = get_state(env);
        let leaked_str = jvm
            .native
            .native_interface_allocations
            .allocate_string(jvm.classpath.classpath_string());
        value_ptr.write(leaked_str);
        return jvm
            .config
            .tracing
            .trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE);
        //todo duplication
    }

    if property_name == "java.version" {
        let leaked_str = jvm
            .native
            .native_interface_allocations
            .allocate_string("1.8".to_string());
        value_ptr.write(leaked_str);
        return jvm
            .config
            .tracing
            .trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE);
        //todo duplication
    }

    if property_name == "path.separator" {
        let leaked_str = jvm
            .native
            .native_interface_allocations
            .allocate_string(":".to_string());
        value_ptr.write(leaked_str);
        return jvm
            .config
            .tracing
            .trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE);
        //todo duplication
    }

    if property_name == "user.dir" {
        if let Ok(dir) = std::env::current_dir() {
            let leaked_str = jvm
                .native
                .native_interface_allocations
                .allocate_string(dir.to_string_lossy().to_string());
            value_ptr.write(leaked_str);
            return jvm
                .config
                .tracing
                .trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE);
            //todo duplication
        };
    }

    jvm.config
        .tracing
        .trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NOT_AVAILABLE)
}
