use std::time::{Duration, SystemTime, UNIX_EPOCH};

use jvmti_jni_bindings::{jboolean, jlocation, jlong, JNIEnv, jobject, jthread, JVM_Available};
use slow_interpreter::better_java_stack::frames::HasFrame;
use slow_interpreter::java_values::JavaValue;


use slow_interpreter::rust_jni::native_util::{from_object, from_object_new};
use slow_interpreter::utils::pushable_frame_todo;use slow_interpreter::rust_jni::jni_utils::{get_interpreter_state, get_state};

///Blocks current thread, returning when a balancing unpark occurs, or a balancing unpark has already occurred,
/// or the thread is interrupted, or, if not absolute and time is not zero, the given time nanoseconds have
/// elapsed, or if absolute, the given deadline in milliseconds since Epoch has passed, or spuriously
/// (i.e., returning for no "reason").
/// Note: This operation is in the Unsafe class only because unpark is, so it would be strange to place it elsewhere.
#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_park(env: *mut JNIEnv, _unsafe: jobject, is_absolute: jboolean, time: jlong) {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let current_thread = &jvm.thread_state.get_current_thread();
    if time == 0 {
        let _ = current_thread.park(jvm, int_state, None);
        return;
    }
    let _ = if is_absolute != 0 {
        let now = SystemTime::now();
        let unix_time = now.duration_since(UNIX_EPOCH).unwrap().as_millis(); //todo maybe we should handle being in the past
        let amount_to_wait = time as u128 - unix_time;
        current_thread.park(jvm, int_state, Some(amount_to_wait))
    } else {
        // int_state.debug_print_stack_trace(jvm);
        // dbg!(current_thread.thread_object().name(jvm).to_rust_string(jvm));
        current_thread.park(jvm, int_state, Some(time as u128))
    };
}

///Unblocks the given thread blocked on park, or, if it is not blocked, causes the subsequent call to park
/// not to block. Note: this operation is "unsafe" solely because the caller must somehow ensure that the
/// thread has not been destroyed. Nothing special is usually required to ensure this when called from
/// Java (in which there will ordinarily be a live reference to the thread) but this is not
/// nearly-automatically so when calling from native code.
// Params:
// thread – the thread to unpark.
#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_unpark(env: *mut JNIEnv, _unsafe: jobject, thread: jthread) {
    let jvm = get_state(env);
    let thread_obj = from_object_new(jvm, thread).unwrap().new_java_value_handle().cast_thread(jvm);
    let target_thread = thread_obj.get_java_thread(jvm);
    let interpreter_state = get_interpreter_state(env);
    // interpreter_state.debug_print_stack_trace(jvm);
    // dbg!(target_thread.thread_object().name(jvm).to_rust_string(jvm));
    target_thread.unpark(jvm, interpreter_state);
}