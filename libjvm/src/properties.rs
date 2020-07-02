use jvmti_jni_bindings::{jobject, JNIEnv};
use slow_interpreter::instructions::ldc::create_string_on_stack;
use slow_interpreter::rust_jni::native_util::{get_state, get_frame, from_object};

use slow_interpreter::instructions::invoke::virtual_::invoke_virtual_method_i;
use slow_interpreter::java_values::JavaValue;
use descriptor_parser::parse_method_descriptor;

#[no_mangle]
unsafe extern "system" fn JVM_InitProperties(env: *mut JNIEnv, p0: jobject) -> jobject {
    //todo get rid of these  hardcoded paths
    let p1 = add_prop(env, p0, "sun.boot.library.path".to_string(), "/home/francis/Clion/rust-jvm/target/debug/deps:/home/francis/Desktop/jdk8u232-b09/jre/lib/amd64".to_string());
    let p2 = add_prop(env, p1, "java.library.path".to_string(), "/usr/java/packages/lib/amd64:/usr/lib64:/lib64:/lib:/usr/lib".to_string());
    p2
}

unsafe fn add_prop(env: *mut JNIEnv, p: jobject, key: String, val: String) -> jobject {
    let frame = get_frame(&mut get_frames(env));
    let state = get_state(env);
    create_string_on_stack(state, key);
    let key = frame.pop();
    create_string_on_stack(state, val);
    let val = frame.pop();
    let prop_obj = from_object(p).unwrap();
    let runtime_class = &prop_obj.unwrap_normal_object().class_pointer;
    let class_view = &runtime_class.view();
    let candidate_meth = class_view.lookup_method_name(&"setProperty".to_string());
    let meth = candidate_meth.iter().next().unwrap();
    let md = meth.desc();
    frame.push(JavaValue::Object(prop_obj.clone().into()));
    frame.push(key);
    frame.push(val);
    invoke_virtual_method_i(state,  md, runtime_class.clone(), meth.method_i(), meth, false);
    frame.pop();
    p
}

