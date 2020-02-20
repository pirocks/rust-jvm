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
/**
    * Determines if the specified {@code Class} object represents a
    * primitive type.
    *
    * <p> There are nine predefined {@code Class} objects to represent
    * the eight primitive types and void.  These are created by the Java
    * Virtual Machine, and have the same names as the primitive types that
    * they represent, namely {@code boolean}, {@code byte},
    * {@code char}, {@code short}, {@code int},
    * {@code long}, {@code float}, and {@code double}.
    *
    * <p> These objects may only be accessed via the following public static
    * final variables, and are the only {@code Class} objects for which
    * this method returns {@code true}.
    *
    * @return true if and only if this class represents a primitive type
    *
    * @see     java.lang.Boolean#TYPE
    * @see     java.lang.Character#TYPE
    * @see     java.lang.Byte#TYPE
    * @see     java.lang.Short#TYPE
    * @see     java.lang.Integer#TYPE
    * @see     java.lang.Long#TYPE
    * @see     java.lang.Float#TYPE
    * @see     java.lang.Double#TYPE
    * @see     java.lang.Void#TYPE
    * @since JDK1.1
    */
unsafe extern "system" fn JVM_IsPrimitiveClass(env: *mut JNIEnv, cls: jclass) -> jboolean {
//    get_frame(env).print_stack_trace();
    let class_object = runtime_class_from_object(cls);
    if class_object.is_none() {
        return false as jboolean;
    }
    let name_ = class_name(&class_object.unwrap().classfile);
    let name = name_.get_referred_name();
    dbg!(&name);
    let is_primitive = name == &"java/lang/Boolean".to_string() ||
        name == &"java/lang/Character".to_string() ||
        name == &"java/lang/Byte".to_string() ||
        name == &"java/lang/Short".to_string() ||
        name == &"java/lang/Integer".to_string() ||
        name == &"java/lang/Long".to_string() ||
        name == &"java/lang/Float".to_string() ||
        name == &"java/lang/Double".to_string() ||
        name == &"java/lang/Void".to_string();

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

