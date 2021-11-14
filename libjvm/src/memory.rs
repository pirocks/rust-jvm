use jvmti_jni_bindings::jlong;

#[no_mangle]
unsafe extern "system" fn JVM_TotalMemory() -> jlong {
    //todo this is hard to implement , so for now make it up
    //so far this seems only used in rng.
    100000
}

#[no_mangle]
unsafe extern "system" fn JVM_FreeMemory() -> jlong {
    //todo this is hard to implement , so for now make it up
    //so far this seems only used in rng.
    //todo in future will need to implement this for reals
    1000000000000 - 100000
}

#[no_mangle]
unsafe extern "system" fn JVM_MaxMemory() -> jlong {
    //todo this is hard to implement , so for now make it up
    //so far this seems only used in rng.
    1000000000000
}
