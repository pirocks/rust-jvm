use jvmti_jni_bindings::jboolean;
use slow_interpreter::jvm_state::JVM;

#[no_mangle]
unsafe extern "system" fn JVM_TraceInstructions(on: jboolean) {
    eprintln!("Instruction Tracing not supported");
}

#[no_mangle]
unsafe extern "system" fn JVM_TraceMethodCalls(on: jboolean) {
    //todo make sure JVM actually gets set b/c
    *JVM.as_ref()
        .unwrap()
        .config
        .tracing
        .trace_function_start
        .write()
        .unwrap() = on != 0;
    *JVM.as_ref()
        .unwrap()
        .config
        .tracing
        .trace_function_end
        .write()
        .unwrap() = on != 0;
}
