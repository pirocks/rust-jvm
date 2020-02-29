use jni_bindings::{jdouble, jboolean, JNIEnv, jclass};
use rust_jvm_common::classfile::ACC_INTERFACE;
use rust_jvm_common::classnames::class_name;
use slow_interpreter::rust_jni::interface::util::runtime_class_from_object;
use slow_interpreter::rust_jni::native_util::from_object;

#[no_mangle]
unsafe extern "system" fn JVM_IsNaN(d: jdouble) -> jboolean {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_IsInterface(env: *mut JNIEnv, cls: jclass) -> jboolean {
//    get_frame(env).print_stack_trace();
    (runtime_class_from_object(cls).unwrap().classfile.access_flags & ACC_INTERFACE > 0) as jboolean
}

#[no_mangle]
unsafe extern "system" fn JVM_IsArrayClass(env: *mut JNIEnv, cls: jclass) -> jboolean {
    let object_non_null = from_object(cls).unwrap().clone();
    let object_class = object_non_null.unwrap_normal_object().array_class_object_pointer.borrow();
    object_class.is_some() as jboolean
}

#[no_mangle]
unsafe extern "system" fn JVM_IsPrimitiveClass(env: *mut JNIEnv, cls: jclass) -> jboolean {
    let class_object = runtime_class_from_object(cls);
    if class_object.is_none() {
        dbg!(&class_object);
        return false as jboolean;
    }
    let name_ = class_name(&class_object.unwrap().classfile);
    let name = name_.get_referred_name();
    dbg!(name);
    dbg!(name == &"java/lang/Integer".to_string());
    let is_primitive = name == &"java/lang/Boolean".to_string() ||
        name == &"java/lang/Character".to_string() ||
        name == &"java/lang/Byte".to_string() ||
        name == &"java/lang/Short".to_string() ||
        name == &"java/lang/Integer".to_string() ||
        name == &"java/lang/Long".to_string() ||
        name == &"java/lang/Float".to_string() ||
        name == &"java/lang/Double".to_string() ||
        name == &"java/lang/Void".to_string();

    dbg!(is_primitive);

    dbg!(is_primitive as jboolean);
    is_primitive as jboolean
}


#[no_mangle]
unsafe extern "system" fn JVM_IsConstructorIx(env: *mut JNIEnv, cb: jclass, index: ::std::os::raw::c_int) -> jboolean {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_IsVMGeneratedMethodIx(env: *mut JNIEnv, cb: jclass, index: ::std::os::raw::c_int) -> jboolean {
    unimplemented!()
}

