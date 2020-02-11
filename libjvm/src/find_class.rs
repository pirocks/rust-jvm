use jni_bindings::{jclass, jstring, jobject, JNIEnv, jboolean};
use rust_jvm_common::classnames::ClassName;
use slow_interpreter::get_or_create_class_object;
use slow_interpreter::rust_jni::native_util::{to_object, get_state, get_frame};

#[no_mangle]
unsafe extern "system" fn JVM_FindClassFromBootLoader(env: *mut JNIEnv, name: *const ::std::os::raw::c_char) -> jclass {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_FindClassFromClassLoader(env: *mut JNIEnv, name: *const ::std::os::raw::c_char, init: jboolean, loader: jobject, throwError: jboolean) -> jclass {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_FindClassFromClass(env: *mut JNIEnv, name: *const ::std::os::raw::c_char, init: jboolean, from: jclass) -> jclass {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_FindLoadedClass(env: *mut JNIEnv, loader: jobject, name: jstring) -> jclass {
    unimplemented!()
}


#[no_mangle]
unsafe extern "system" fn JVM_FindPrimitiveClass(env: *mut JNIEnv, utf: *const ::std::os::raw::c_char) -> jclass {
    // need to perform not equal to 0 check
    if *utf.offset(0) == 'f' as i8 &&
        *utf.offset(1) == 'l' as i8 &&
        *utf.offset(2) == 'o' as i8 &&
        *utf.offset(3) == 'a' as i8 &&
        *utf.offset(4) == 't' as i8 &&
        *utf.offset(5) == 0 {
        let state = get_state(env);
        let frame = get_frame(env);
        let res = get_or_create_class_object(state, &ClassName::new("java/lang/Float"), frame, state.bootstrap_loader.clone());//todo what if not using bootstap loader
        return to_object(res.into());
    }
    if *utf.offset(0) == 'd' as i8 &&
        *utf.offset(1) == 'o' as i8 &&
        *utf.offset(2) == 'u' as i8 &&
        *utf.offset(3) == 'b' as i8 &&
        *utf.offset(4) == 'l' as i8 &&
        *utf.offset(5) == 'e' as i8 &&
        *utf.offset(6) == 0 {
        let state = get_state(env);
        let frame = get_frame(env);
        let res = get_or_create_class_object(state, &ClassName::new("java/lang/Double"), frame, state.bootstrap_loader.clone());//todo what if not using bootstap loader
        return to_object(res.into());
    }
    if *utf.offset(0) == 'i' as i8 &&
        *utf.offset(1) == 'n' as i8 &&
        *utf.offset(2) == 't' as i8 &&
        *utf.offset(3) == 0 as i8 {
        let state = get_state(env);
        let frame = get_frame(env);
        let res = get_or_create_class_object(state, &ClassName::new("java/lang/Integer"), frame, state.bootstrap_loader.clone());//todo what if not using bootstap loader
        return to_object(res.into());
    }
    if *utf.offset(0) == 'b' as i8 &&
        *utf.offset(1) == 'o' as i8 &&
        *utf.offset(2) == 'o' as i8 &&
        *utf.offset(3) == 'l' as i8 &&
        *utf.offset(4) == 'e' as i8 &&
        *utf.offset(5) == 'a' as i8 &&
        *utf.offset(6) == 'n' as i8 &&
        *utf.offset(7) == 0 {
        let state = get_state(env);
        let frame = get_frame(env);
        let res = get_or_create_class_object(state, &ClassName::new("java/lang/Boolean"), frame, state.bootstrap_loader.clone());//todo what if not using bootstap loader
        return to_object(res.into());
    }
    if *utf.offset(0) == 'c' as i8 &&
        *utf.offset(1) == 'h' as i8 &&
        *utf.offset(2) == 'a' as i8 &&
        *utf.offset(3) == 'r' as i8 &&
        *utf.offset(4) == 0 {
        let state = get_state(env);
        let frame = get_frame(env);
        let res = get_or_create_class_object(state, &ClassName::new("java/lang/Character"), frame, state.bootstrap_loader.clone());//todo what if not using bootstap loader
        return to_object(res.into());
    }

    if *utf.offset(0) == 'l' as i8 &&
        *utf.offset(1) == 'o' as i8 &&
        *utf.offset(2) == 'n' as i8 &&
        *utf.offset(3) == 'g' as i8 &&
        *utf.offset(4) == 0 {
        let state = get_state(env);
        let frame = get_frame(env);
        let res = get_or_create_class_object(state, &ClassName::new("java/lang/Long"), frame, state.bootstrap_loader.clone());//todo what if not using bootstap loader
        return to_object(res.into());
    }

    dbg!((*utf) as u8 as char);
    unimplemented!()
}
