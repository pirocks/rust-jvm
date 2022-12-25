use java_lang_invoke_method_handle_natives::Java_java_lang_invoke_MethodHandleNatives_registerNatives;
use sun_misc_perf::Java_sun_misc_Perf_registerNatives;
use sun_misc_unsafe::Java_sun_misc_Unsafe_registerNatives;

#[no_mangle]
pub fn __rust_jvm_use_deps(){
    std::hint::black_box(Java_sun_misc_Unsafe_registerNatives);
    std::hint::black_box(Java_sun_misc_Perf_registerNatives);
    std::hint::black_box(Java_java_lang_invoke_MethodHandleNatives_registerNatives);
}
