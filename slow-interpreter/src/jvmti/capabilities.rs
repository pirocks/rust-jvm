use jvmti_bindings::{jvmtiEnv, jvmtiCapabilities, jvmtiError, jvmtiError_JVMTI_ERROR_NONE, jvmtiError_JVMTI_ERROR_MUST_POSSESS_CAPABILITY};
use std::os::raw::c_void;
use std::mem::{size_of, transmute};
use crate::jvmti::get_state;

// can_access_local_variables              = 1
// can_generate_single_step_events         = 1
// can_generate_exception_events           = 1
// can_generate_frame_pop_events           = 1
// can_generate_breakpoint_events          = 1
// can_suspend                             = 1
// can_generate_method_entry_events        = 1
// can_generate_method_exit_events         = 1
// can_generate_garbage_collection_events  = 1
// can_maintain_original_method_order      = 1
// can_generate_monitor_events             = 1
// can_tag_objects                         = 1


pub unsafe extern "C" fn get_potential_capabilities(env: *mut jvmtiEnv, capabilities_ptr: *mut jvmtiCapabilities) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm,"GetPotentialCapabilities");
    //    unsigned int can_tag_objects : 1;
    (*capabilities_ptr).set_can_tag_objects(1);
    //     unsigned int can_generate_field_modification_events : 1;
    (*capabilities_ptr).set_can_generate_field_modification_events(0);
    //     unsigned int can_generate_field_access_events : 1;
    (*capabilities_ptr).set_can_generate_field_access_events(0);
    //     unsigned int can_get_bytecodes : 1;
    (*capabilities_ptr).set_can_get_bytecodes(0);
    //     unsigned int can_get_synthetic_attribute : 1;
    (*capabilities_ptr).set_can_get_synthetic_attribute(0);
    //     unsigned int can_get_owned_monitor_info : 1;
    (*capabilities_ptr).set_can_get_owned_monitor_info(0);
    //     unsigned int can_get_current_contended_monitor : 1;
    (*capabilities_ptr).set_can_get_current_contended_monitor(0);
    //     unsigned int can_get_monitor_info : 1;
    (*capabilities_ptr).set_can_get_owned_monitor_info(0);
    //     unsigned int can_pop_frame : 1;
    (*capabilities_ptr).set_can_pop_frame(0);
    //     unsigned int can_redefine_classes : 1;
    (*capabilities_ptr).set_can_redefine_classes(0);
    //     unsigned int can_signal_thread : 1;
    (*capabilities_ptr).set_can_signal_thread(0);
    //     unsigned int can_get_source_file_name : 1;
    (*capabilities_ptr).set_can_get_source_file_name(1);
    //     unsigned int can_get_line_numbers : 1;
    (*capabilities_ptr).set_can_get_line_numbers(1);
    //     unsigned int can_get_source_debug_extension : 1;
    (*capabilities_ptr).set_can_get_source_debug_extension(1);
    //     unsigned int can_access_local_variables : 1;
    (*capabilities_ptr).set_can_access_local_variables(1);
    //     unsigned int can_maintain_original_method_order : 1;
    (*capabilities_ptr).set_can_maintain_original_method_order(1);
    //     unsigned int can_generate_single_step_events : 1;
    (*capabilities_ptr).set_can_generate_single_step_events(1);
    //     unsigned int can_generate_exception_events : 1;
    (*capabilities_ptr).set_can_generate_exception_events(1);
    //     unsigned int can_generate_frame_pop_events : 1;
    (*capabilities_ptr).set_can_generate_frame_pop_events(1);
    //     unsigned int can_generate_breakpoint_events : 1;
    (*capabilities_ptr).set_can_generate_breakpoint_events(1);
    //     unsigned int can_suspend : 1;
    (*capabilities_ptr).set_can_suspend(1);
    //     unsigned int can_redefine_any_class : 1;
    (*capabilities_ptr).set_can_redefine_any_class(0);
    //     unsigned int can_get_current_thread_cpu_time : 1;
    (*capabilities_ptr).set_can_get_current_thread_cpu_time(0);
    //     unsigned int can_get_thread_cpu_time : 1;
    (*capabilities_ptr).set_can_get_thread_cpu_time(0);
    //     unsigned int can_generate_method_entry_events : 1;
    (*capabilities_ptr).set_can_generate_method_entry_events(1);
    //     unsigned int can_generate_method_exit_events : 1;
    (*capabilities_ptr).set_can_generate_method_exit_events(1);
    //     unsigned int can_generate_all_class_hook_events : 1;
    (*capabilities_ptr).set_can_generate_all_class_hook_events(0);
    //     unsigned int can_generate_compiled_method_load_events : 1;
    (*capabilities_ptr).set_can_generate_compiled_method_load_events(0);
    //     unsigned int can_generate_monitor_events : 1;
    (*capabilities_ptr).set_can_generate_monitor_events(1);
    //     unsigned int can_generate_vm_object_alloc_events : 1;
    (*capabilities_ptr).set_can_generate_vm_object_alloc_events(0);
    //     unsigned int can_generate_native_method_bind_events : 1;
    (*capabilities_ptr).set_can_generate_native_method_bind_events(0);
    //     unsigned int can_generate_garbage_collection_events : 1;
    (*capabilities_ptr).set_can_generate_garbage_collection_events(1);
    //     unsigned int can_generate_object_free_events : 1;
    (*capabilities_ptr).set_can_generate_object_free_events(0);
    //     unsigned int can_force_early_return : 1;
    (*capabilities_ptr).set_can_force_early_return(0);
    //     unsigned int can_get_owned_monitor_stack_depth_info : 1;
    (*capabilities_ptr).set_can_get_owned_monitor_stack_depth_info(0);
    //     unsigned int can_get_constant_pool : 1;
    (*capabilities_ptr).set_can_get_constant_pool(0);
    //     unsigned int can_set_native_method_prefix : 1;
    (*capabilities_ptr).set_can_set_native_method_prefix(0);
    //     unsigned int can_retransform_classes : 1;
    (*capabilities_ptr).set_can_retransform_classes(0);
    //     unsigned int can_retransform_any_class : 1;
    (*capabilities_ptr).set_can_retransform_any_class(0);
    //     unsigned int can_generate_resource_exhaustion_heap_events : 1;
    (*capabilities_ptr).set_can_generate_resource_exhaustion_heap_events(0);
    //     unsigned int can_generate_resource_exhaustion_threads_events : 1;
    (*capabilities_ptr).set_can_generate_resource_exhaustion_threads_events(0);
    //     unsigned int : 7;
    //     unsigned int : 16;
    //     unsigned int : 16;
    //     unsigned int : 16;
    //     unsigned int : 16;
    //     unsigned int : 16;
    jvm.tracing.trace_jdwp_function_exit(jvm,"GetPotentialCapabilities");
    jvmtiError_JVMTI_ERROR_NONE
}

pub unsafe extern "C" fn add_capabilities(
    env: *mut jvmtiEnv,
    capabilities_ptr: *const jvmtiCapabilities,
) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm,"AddCapabilities");
    let res = if (*capabilities_ptr).can_generate_field_modification_events() > 0 ||
        (*capabilities_ptr).can_generate_field_access_events() > 0 ||
        (*capabilities_ptr).can_get_bytecodes() > 0 ||
        (*capabilities_ptr).can_get_synthetic_attribute() > 0 ||
        (*capabilities_ptr).can_get_owned_monitor_info() > 0 ||
        (*capabilities_ptr).can_get_current_contended_monitor() > 0 ||
        (*capabilities_ptr).can_get_owned_monitor_info() > 0 ||
        (*capabilities_ptr).can_pop_frame() > 0 ||
        (*capabilities_ptr).can_redefine_classes() > 0 ||
        (*capabilities_ptr).can_signal_thread() > 0 ||
        (*capabilities_ptr).can_redefine_any_class() > 0 ||
        (*capabilities_ptr).can_get_current_thread_cpu_time() > 0 ||
        (*capabilities_ptr).can_get_thread_cpu_time() > 0 ||
        (*capabilities_ptr).can_generate_all_class_hook_events() > 0 ||
        (*capabilities_ptr).can_generate_compiled_method_load_events() > 0 ||
        (*capabilities_ptr).can_generate_vm_object_alloc_events() > 0 ||
        (*capabilities_ptr).can_generate_native_method_bind_events() > 0 ||
        (*capabilities_ptr).can_generate_object_free_events() > 0 ||
        (*capabilities_ptr).can_force_early_return() > 0 ||
        (*capabilities_ptr).can_get_owned_monitor_stack_depth_info() > 0 ||
        (*capabilities_ptr).can_get_constant_pool() > 0 ||
        (*capabilities_ptr).can_set_native_method_prefix() > 0 ||
        (*capabilities_ptr).can_retransform_classes() > 0 ||
        (*capabilities_ptr).can_retransform_any_class() > 0 ||
        (*capabilities_ptr).can_generate_resource_exhaustion_heap_events() > 0 ||
        (*capabilities_ptr).can_generate_resource_exhaustion_threads_events() > 0 {
        jvmtiError_JVMTI_ERROR_MUST_POSSESS_CAPABILITY//todo is this the right error? Does it matter.
    } else {
        jvmtiError_JVMTI_ERROR_NONE
    };
    jvm.tracing.trace_jdwp_function_exit(jvm,"AddCapabilities");
    res
}
// can_access_local_variables              = 1
// can_generate_single_step_events         = 1
// can_generate_exception_events           = 1
// can_generate_frame_pop_events           = 1
// can_generate_breakpoint_events          = 1
// can_suspend                             = 1
// can_generate_method_entry_events        = 1
// can_generate_method_exit_events         = 1
// can_generate_garbage_collection_events  = 1
// can_maintain_original_method_order      = 1
// can_generate_monitor_events             = 1
// can_tag_objects                         = 1
pub unsafe extern "C" fn get_capabilities(env: *mut jvmtiEnv, capabilities_ptr: *mut jvmtiCapabilities) -> jvmtiError{
    let jvm  =  get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm,"GetCapabilities");
    libc::memset(capabilities_ptr as *mut c_void,0,size_of::<jvmtiCapabilities>());
    let mut_borrow: &mut jvmtiCapabilities = transmute(capabilities_ptr);//todo what is the correct way to do this?
    mut_borrow.set_can_access_local_variables(1);
    mut_borrow.set_can_generate_single_step_events(1);
    mut_borrow.set_can_generate_exception_events(1);
    mut_borrow.set_can_generate_frame_pop_events(1);
    mut_borrow.set_can_generate_breakpoint_events(1);
    mut_borrow.set_can_suspend(1);
    mut_borrow.set_can_generate_method_entry_events(1);
    mut_borrow.set_can_generate_method_exit_events(1);
    mut_borrow.set_can_generate_garbage_collection_events(1);
    mut_borrow.set_can_maintain_original_method_order(1);
    mut_borrow.set_can_generate_monitor_events(1);
    mut_borrow.set_can_tag_objects(1);
    jvm.tracing.trace_jdwp_function_exit(jvm,"GetCapabilities");
    jvmtiError_JVMTI_ERROR_NONE

}