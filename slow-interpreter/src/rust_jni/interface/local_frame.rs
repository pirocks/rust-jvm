use jvmti_jni_bindings::{jint, JNI_OK, JNIEnv, jobject};

use crate::java_values::JavaValue;
use crate::rust_jni::native_util::{get_interpreter_state, to_object};

pub unsafe extern "C" fn pop_local_frame(env: *mut JNIEnv, result: jobject) -> jobject {
    assert_eq!(result, std::ptr::null_mut());
    // let jv = from_object(result);

    // let jvm = get_state(env);
    // unimplemented!();
    //todo this is wrong
    // jvm.thread_state.get_current_thread().call_stack.write().unwrap().pop();

    to_object(None)
    // unimplemented!();
}

pub unsafe extern "C" fn push_local_frame(env: *mut JNIEnv, capacity: jint) -> jint {
    // let jvm = get_state(env);
    // let int_state = get_interpreter_state(env);
    // let frame = int_state.current_frame_mut();
    // let mut new_local_vars = vec![];
    // for i in 0..capacity {
    //     match frame.local_vars.get(i as usize) {
    //         None => new_local_vars.push(JavaValue::Top),
    //         Some(jv) => new_local_vars.push(jv.clone()),
    //     }
    // }
    //todo so this what this should do. but different
    // unimplemented!("get clarity on what this actually does");
    /*jvm.thread_state.get_current_thread().call_stack.write().unwrap().push(StackEntry {
        class_pointer: frame.class_pointer.clone(),
        method_i: std::u16::MAX,
        local_vars: new_local_vars,
        operand_stack: vec![],
        pc: std::usize::MAX,
        pc_offset: std::isize::MAX,
    });*/
    JNI_OK as i32
}
