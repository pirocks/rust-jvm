use descriptor_parser::parse_method_descriptor;
use jvmti_jni_bindings::{JNIEnv, jobject};
use slow_interpreter::instructions::invoke::virtual_::invoke_virtual_method_i;
use slow_interpreter::instructions::ldc::create_string_on_stack;
use slow_interpreter::java_values::JavaValue;
use slow_interpreter::rust_jni::native_util::{from_object, get_interpreter_state, get_state};

#[no_mangle]
unsafe extern "system" fn JVM_InitProperties(env: *mut JNIEnv, p0: jobject) -> jobject {
    //todo get rid of these  hardcoded paths
    // sun.boot.class.path
    add_prop(env, p0, "sun.boot.library.path".to_string(), "/home/francis/Clion/rust-jvm/target/debug/deps:/home/francis/build/openjdk-debug/jdk8u/build/linux-x86_64-normal-server-slowdebug/jdk/lib/amd64".to_string());
    add_prop(env, p0, "sun.boot.class.path".to_string(), "/home/francis/build/openjdk-debug/jdk8u/build/linux-x86_64-normal-server-slowdebug/jdk/lib/jce.jar:/home/francis/build/openjdk-debug/jdk8u/build/linux-x86_64-normal-server-slowdebug/jdk/classes".to_string());
    add_prop(env, p0, "java.class.path".to_string(), "/home/francis/build/openjdk-debug/jdk8u/build/linux-x86_64-normal-server-slowdebug/jdk/lib/jce.jar:/home/francis/build/openjdk-debug/jdk8u/build/linux-x86_64-normal-server-slowdebug/jdk/classes".to_string());
    add_prop(env, p0, "java.library.path".to_string(), "/usr/java/packages/lib/amd64:/usr/lib64:/lib64:/lib:/usr/lib".to_string());
    add_prop(env, p0, "java.home".to_string(), "/home/francis/build/openjdk-debug/jdk8u/build/linux-x86_64-normal-server-slowdebug/jdk/".to_string())
}

unsafe fn add_prop(env: *mut JNIEnv, p: jobject, key: String, val: String) -> jobject {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    create_string_on_stack(jvm, int_state, key);
    let key = int_state.pop_current_operand_stack();
    create_string_on_stack(jvm, int_state, val);
    let val = int_state.pop_current_operand_stack();
    let prop_obj = from_object(p).unwrap();
    let runtime_class = &prop_obj.unwrap_normal_object().class_pointer;
    let class_view = &runtime_class.view();
    let candidate_meth = class_view.lookup_method_name(&"setProperty".to_string());
    let meth = candidate_meth.get(0).unwrap();
    let md = meth.desc();
    int_state.push_current_operand_stack(JavaValue::Object(prop_obj.clone().into()));
    int_state.push_current_operand_stack(key);
    int_state.push_current_operand_stack(val);
    invoke_virtual_method_i(jvm, int_state, md, runtime_class.clone(), meth.method_i(), meth);
    int_state.pop_current_operand_stack();
    p
}

