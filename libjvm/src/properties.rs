use std::ptr::null_mut;
use wtf8::Wtf8Buf;

use classfile_view::view::ClassView;
use jvmti_jni_bindings::{JNIEnv, jobject};
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::compressed_classfile::names::{CClassName, MethodName};
use slow_interpreter::instructions::invoke::virtual_::invoke_virtual_method_i;
use slow_interpreter::instructions::ldc::create_string_on_stack;
use slow_interpreter::interpreter::WasException;
use slow_interpreter::java::lang::string::JString;
use slow_interpreter::java::NewAsObjectOrJavaValue;
use slow_interpreter::java_values::JavaValue;
use slow_interpreter::new_java_values::NewJavaValue;
use slow_interpreter::rust_jni::interface::local_frame::new_local_ref_public_new;
use slow_interpreter::rust_jni::native_util::{from_object, from_object_new, get_interpreter_state, get_state};
use slow_interpreter::utils::throw_npe_res;

#[no_mangle]
unsafe extern "system" fn JVM_InitProperties(env: *mut JNIEnv, p0: jobject) -> jobject {
    //todo get rid of these  hardcoded paths
    // sun.boot.class.path
    let res = match (|| {
        add_prop(env, p0, "sun.boot.library.path".to_string(), "/home/francis/Clion/rust-jvm/target/debug/deps:/home/francis/build/openjdk-debug/jdk8u/build/linux-x86_64-normal-server-slowdebug/jdk/lib/amd64".to_string())?;
        add_prop(env, p0, "sun.boot.class.path".to_string(), "/home/francis/build/openjdk-debug/jdk8u/build/linux-x86_64-normal-server-slowdebug/jdk/lib/jce.jar:/home/francis/build/openjdk-debug/jdk8u/build/linux-x86_64-normal-server-slowdebug/jdk/classes:/home/francis/Desktop/test/unzipped-jar".to_string())?;
        add_prop(env, p0, "java.class.path".to_string(), "/home/francis/build/openjdk-debug/jdk8u/build/linux-x86_64-normal-server-slowdebug/jdk/lib/jce.jar:/home/francis/build/openjdk-debug/jdk8u/build/linux-x86_64-normal-server-slowdebug/jdk/classes:/home/francis/Desktop/test/unzipped-jar".to_string())?;
        add_prop(env, p0, "java.library.path".to_string(), "/usr/java/packages/lib/amd64:/usr/lib64:/lib64:/lib:/usr/lib".to_string())?;
        // add_prop(env, p0, "org.slf4j.simpleLogger.defaultLogLevel ".to_string(), "off".to_string())?;
        add_prop(env, p0, "log4j2.disable.jmx".to_string(), "true".to_string());
        Ok(add_prop(env, p0, "java.home".to_string(), "/home/francis/build/openjdk-debug/jdk8u/build/linux-x86_64-normal-server-slowdebug/jdk/".to_string())?)
    })() {
        Err(WasException {}) => return null_mut(),
        Ok(res) => res,
    };
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let prop_obj = from_object_new(jvm,p0).unwrap();
    let key = JString::from_rust(jvm, int_state, Wtf8Buf::from_string("user.dir".to_string())).unwrap();
    let properties = prop_obj.cast_properties();
    let table = properties.table(jvm);
    let table_array = table.unwrap_array(jvm);
    for value in table_array.array_iterator() {
        if let Some(notnull) = value.unwrap_object(){
            let entry = notnull.cast_entry();
            dbg!(entry.hash(jvm));
        }else {
            dbg!("null");
        }
    }
    // let _ = properties.get_property(jvm, int_state, key).unwrap().unwrap().new_java_value_handle().unwrap_object().unwrap();
    /*let key = key.new_java_value();
    let handle = invoke_virtual_method_i(
        jvm,
        int_state,
        md,
        runtime_class.clone(),
        meth,
        vec![NewJavaValue::AllocObject(prop_obj.as_allocated_obj()), key]
    ).unwrap().unwrap();
    handle.unwrap_object_nonnull();
    res*/
    res
}

unsafe fn add_prop(env: *mut JNIEnv, p: jobject, key: String, val: String) -> Result<jobject, WasException> {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let key = JString::from_rust(jvm, int_state, Wtf8Buf::from_string(key))?.intern(jvm,int_state)?;
    let val = JString::from_rust(jvm,int_state,Wtf8Buf::from_string(val))?.intern(jvm,int_state)?;
    let prop_obj = match from_object_new(jvm, p) {
        Some(x) => x,
        None => return throw_npe_res(jvm, int_state),
    };
    let runtime_class = &prop_obj.as_allocated_obj().runtime_class(jvm);
    let class_view = &runtime_class.view();
    let candidate_meth = class_view.lookup_method_name(MethodName::method_setProperty());
    let meth = candidate_meth.get(0).unwrap();
    let md = meth.desc();

    let p = invoke_virtual_method_i(
        jvm,
        int_state,
        md,
        runtime_class.clone(),
        meth,
        vec![NewJavaValue::AllocObject(prop_obj.as_allocated_obj()),key.new_java_value_handle().as_njv(),val.new_java_value_handle().as_njv()]
    )?.unwrap().unwrap_object();
    Ok(new_local_ref_public_new(p.as_ref().map(|handle|handle.as_allocated_obj()),int_state))
}