use jvmti_jni_bindings::{__va_list_tag, FILE, vfprintf, vsnprintf};

#[no_mangle]
unsafe extern "system" fn jio_vsnprintf(
    str: *mut ::std::os::raw::c_char,
    count: usize,
    fmt: *const ::std::os::raw::c_char,
    args: *mut __va_list_tag,
) -> ::std::os::raw::c_int {
    vsnprintf(str, count as u64, fmt, args)
}

#[no_mangle]
unsafe extern "C" fn jio_snprintf(
    str: *mut ::std::os::raw::c_char,
    count: usize,
    fmt: *const ::std::os::raw::c_char,
    args: *mut __va_list_tag,
) -> ::std::os::raw::c_int {
    libc::snprintf(str, count, fmt, args)
}

#[no_mangle]
unsafe extern "C" fn jio_fprintf(
    stream: *mut FILE,
    fmt: *const ::std::os::raw::c_char,
    args: *mut __va_list_tag,
) -> ::std::os::raw::c_int {
    libc::fprintf(stream as *mut libc::FILE, fmt, args)
}

#[no_mangle]
unsafe extern "system" fn jio_vfprintf(
    arg1: *mut FILE,
    fmt: *const ::std::os::raw::c_char,
    args: *mut __va_list_tag,
) -> ::std::os::raw::c_int {
    vfprintf(arg1, fmt, args)
}