use jni_bindings::{jobject, JNIEnv};
use runtime_common::java_values::JavaValue;
use slow_interpreter::instructions::invoke::invoke_virtual_method_i;
use slow_interpreter::instructions::ldc::create_string_on_stack;
use slow_interpreter::rust_jni::native_util::{get_state, get_frame, from_object};
use descriptor_parser::parse_method_descriptor;

#[no_mangle]
unsafe extern "system" fn JVM_InitProperties(env: *mut JNIEnv, p0: jobject) -> jobject {
//sun.boot.library.path
    let p1 = add_prop(env, p0, "sun.boot.library.path".to_string(), "/home/francis/Clion/rust-jvm/target/debug/deps:/home/francis/Desktop/jdk8u232-b09/jre/lib/amd64".to_string());
    let p2 = add_prop(env, p1, "java.library.path".to_string(), "/usr/java/packages/lib/amd64:/usr/lib64:/lib64:/lib:/usr/lib".to_string());
//    dbg!(from_object(p2).unwrap().unwrap_normal_object().fields.borrow().deref().get("table").unwrap());
    p2
}

unsafe fn add_prop(env: *mut JNIEnv, p: jobject, key: String, val: String) -> jobject {
    let frame = get_frame(env);
    let state = get_state(env);
    create_string_on_stack(state, &frame, key);
    let key = frame.pop();
    create_string_on_stack(state, &frame, val);
    let val = frame.pop();
    let prop_obj = from_object(p).unwrap();
    let runtime_class = &prop_obj.unwrap_normal_object().class_pointer;
    let classfile = &runtime_class.classfile;
    let candidate_meth = classfile.lookup_method_name(&"setProperty".to_string());
    let (meth_i, meth) = candidate_meth.iter().next().unwrap();
    let md = parse_method_descriptor(meth.descriptor_str(classfile).as_str()).unwrap();
    frame.push(JavaValue::Object(prop_obj.clone().into()));
    frame.push(key);
    frame.push(val);
    invoke_virtual_method_i(state, frame.clone(), md, runtime_class.clone(), *meth_i, meth);
    frame.pop();
    p
}

