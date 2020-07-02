use jvmti_jni_bindings::{JNIEnv, jint, jobject, JNI_OK};
use crate::rust_jni::native_util::{get_state, get_frame, to_object, get_thread, get_frames};
use crate::java_values::JavaValue;
use crate::stack_entry::StackEntry;

pub unsafe extern "C" fn pop_local_frame(env: *mut JNIEnv, result: jobject) -> jobject {
    assert_eq!(result, std::ptr::null_mut());
    // let jv = from_object(result);

    let jvm = get_state(env);
    //todo this is wrong
    jvm.thread_state.get_current_thread().call_stack.write().unwrap().pop();

    to_object(None);
    unimplemented!();
}

pub unsafe extern "C" fn push_local_frame(env: *mut JNIEnv, capacity: jint) -> jint {
    // let frame = get_frame(&mut get_frames(env));
    let jvm = get_state(env);
    let mut thread = get_thread(env);
    let mut frames = get_frames(&thread);
    let frame = get_frame(&mut frames);
    let mut new_local_vars = vec![];
    for i in 0..capacity {
        match frame.local_vars.get(i as usize) {
            None => new_local_vars.push(JavaValue::Top),
            Some(jv) => new_local_vars.push(jv.clone()),
        }
    }
    //todo so this what this should do. but different
    jvm.thread_state.get_current_thread().call_stack.write().unwrap().push(StackEntry {
        class_pointer: frame.class_pointer.clone(),
        method_i: std::u16::MAX,
        local_vars: new_local_vars,
        operand_stack: vec![],
        pc: std::usize::MAX,
        pc_offset: std::isize::MAX
    });
    JNI_OK as i32
}
