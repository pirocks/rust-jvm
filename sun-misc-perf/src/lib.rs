use jvmti_jni_bindings::{jclass, JNIEnv, jobject};
use slow_interpreter::exceptions::WasException;
use slow_interpreter::java_values::ExceptionReturn;
use slow_interpreter::rust_jni::jni_utils::{get_interpreter_state, get_state, get_throw, new_local_ref_internal_new};
use slow_interpreter::stdlib::java::NewAsObjectOrJavaValue;
use slow_interpreter::stdlib::java::nio::heap_byte_buffer::HeapByteBuffer;

#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Perf_registerNatives(_env: *mut JNIEnv, _cb: jclass) {
    //todo for now register nothing, register later as needed.
}

#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Perf_createLong(env: *mut JNIEnv) -> jobject {
    // todo this is incorrect and should be implemented properly.
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    match HeapByteBuffer::new(jvm, int_state, vec![0, 0, 0, 0, 0, 0, 0, 0], 0, 8){
        Ok(res) => {
            new_local_ref_internal_new(res.full_object_ref(),int_state)
        }
        Err(WasException{ exception_obj }) => {
            *get_throw(env) = Some(WasException{ exception_obj });
            jobject::invalid_default()
        }
    }
}