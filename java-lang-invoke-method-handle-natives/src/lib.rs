use jvmti_jni_bindings::{jclass, jint, JNIEnv};



#[no_mangle]
pub unsafe extern "system" fn Java_java_lang_invoke_MethodHandleNatives_registerNatives(_env: *mut JNIEnv, _cb: jclass) {
    //todo for now register nothing, register later as needed.
}

#[no_mangle]
pub unsafe extern "system" fn Java_java_lang_invoke_MethodHandleNatives_getConstant(_env: *mut JNIEnv, _c: jclass, _i: jint) -> jint {
//pub fn MHN_getConstant<'gc>() -> Result<NewJavaValueHandle<'gc>, WasException<'gc>> {
//     //so I have no idea what this is for, but openjdk does approx this so it should be fine.
//     Ok(NewJavaValueHandle::Int(0))
// }
    0
}


pub mod resolve;
pub mod init;
pub mod get_members;
pub mod offsets;
