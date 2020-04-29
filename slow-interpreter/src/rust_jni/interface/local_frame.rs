use jvmti_jni_bindings::{JNIEnv, jint, jobject, JNI_OK};
use crate::rust_jni::native_util::{get_state, get_frame, to_object};
use crate::java_values::JavaValue;
use std::rc::Rc;
use crate::stack_entry::StackEntry;
use std::cell::RefCell;

pub unsafe extern "C" fn pop_local_frame(env: *mut JNIEnv, result: jobject) -> jobject {
    assert_eq!(result, std::ptr::null_mut());
    // let jv = from_object(result);

    let state = get_state(env);
    state.get_current_thread().call_stack.borrow_mut().pop();

    to_object(None)
}

pub unsafe extern "C" fn push_local_frame(env: *mut JNIEnv, capacity: jint) -> jint {
    // let frame = get_frame(env);
    let state = get_state(env);
    let frame = get_frame(env);
    let mut new_local_vars = vec![];
    for i in 0..capacity {
        match frame.local_vars.borrow().get(i as usize) {
            None => new_local_vars.push(JavaValue::Top),
            Some(jv) => new_local_vars.push(jv.clone()),
        }
    }
    //todo so this what this should do. but different
    state.get_current_thread().call_stack.borrow_mut().push(Rc::new(StackEntry {
        class_pointer: frame.class_pointer.clone(),
        method_i: std::u16::MAX,
        local_vars: RefCell::new(new_local_vars),
        operand_stack: RefCell::new(vec![]),
        pc: RefCell::new(std::usize::MAX),
        pc_offset: RefCell::new(std::isize::MAX)
    }));
    JNI_OK as i32
}
