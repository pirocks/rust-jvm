use std::time::{Duration, SystemTime, UNIX_EPOCH};

use jvmti_jni_bindings::{jboolean, jlocation, jlong, JNIEnv, jobject, jthread, JVM_Available};
use slow_interpreter::java_values::JavaValue;
use slow_interpreter::rust_jni::interface::string::get_string_region;
use slow_interpreter::rust_jni::native_util::{from_object, get_interpreter_state, get_state};

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
        current_thread.park(u64::max_value())//todo this is technically wrong
    }
    if is_absolute != 0 {
        let now = SystemTime::now();
        let unix_time = now.duration_since(UNIX_EPOCH).unwrap().as_millis();
        let amount_to_wait = time as u128 - unix_time;
        current_thread.park(amount_to_wait as u64)
    } else {
        current_thread.park(time as u64)
    }
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
    let thread_obj = JavaValue::Object(from_object(thread)).cast_thread();
    let target_thread = thread_obj.get_java_thread(jvm);
    target_thread.unpark();
}