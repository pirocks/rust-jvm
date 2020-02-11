use jni_bindings::{__va_list_tag, FILE, vsnprintf};
use log::trace;

#[no_mangle]
unsafe extern "system" fn jio_vsnprintf(
    str: *mut ::std::os::raw::c_char,
    count: usize,
    fmt: *const ::std::os::raw::c_char,
    args: *mut __va_list_tag,
) -> ::std::os::raw::c_int {
    trace!("JIO Output:");
    vsnprintf(str, count as u64, fmt, args)
}


#[no_mangle]
unsafe extern "C" fn jio_snprintf(
    str: *mut ::std::os::raw::c_char,
    count: usize,
    fmt: *const ::std::os::raw::c_char,
//    ...
) -> ::std::os::raw::c_int {
    unimplemented!()
}


#[no_mangle]
unsafe extern "C" fn jio_fprintf(
    arg1: *mut FILE,
    fmt: *const ::std::os::raw::c_char,
//    ...
) -> ::std::os::raw::c_int {
    unimplemented!()
}


#[no_mangle]
unsafe extern "system" fn jio_vfprintf(
    arg1: *mut FILE,
    fmt: *const ::std::os::raw::c_char,
    args: *mut __va_list_tag,
) -> ::std::os::raw::c_int {
    unimplemented!()
}

