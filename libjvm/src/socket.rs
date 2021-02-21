use nix::sys::socket::setsockopt;

use jvmti_jni_bindings::{jint, sockaddr};

#[no_mangle]
unsafe extern "system" fn JVM_InitializeSocketLibrary() -> jint {
    0
}

#[no_mangle]
unsafe extern "system" fn JVM_Socket(domain: jint, type_: jint, protocol: jint) -> jint {
    libc::socket(domain, type_, protocol)
}

#[no_mangle]
unsafe extern "system" fn JVM_SocketClose(fd: jint) -> jint {
    libc::close(fd)
}

#[no_mangle]
unsafe extern "system" fn JVM_SocketShutdown(fd: jint, howto: jint) -> jint {
    libc::shutdown(fd, howto)
}

#[no_mangle]
unsafe extern "system" fn JVM_Recv(fd: jint, buf: *mut ::std::os::raw::c_char, nBytes: jint, flags: jint) -> jint {
    libc::recv(fd, buf, nBytes as usize, flags) as i32 //todo these need to restart and repeat if not all read
}

#[no_mangle]
unsafe extern "system" fn JVM_Send(fd: jint, buf: *mut ::std::os::raw::c_char, nBytes: jint, flags: jint) -> jint {
    libc::send(fd, buf, nBytes as usize, flags) as i32//todo these need to restart and repeat if not all read
}

#[no_mangle]
unsafe extern "system" fn JVM_Timeout(fd: ::std::os::raw::c_int, timeout: ::std::os::raw::c_long) -> jint {
    todo!()
}

#[no_mangle]
unsafe extern "system" fn JVM_Listen(fd: jint, count: jint) -> jint {
    libc::listen(fd, count)//todo these need to restart and repeat if not all read
}

#[no_mangle]
unsafe extern "system" fn JVM_Connect(fd: jint, him: *const sockaddr, len: jint) -> jint {
    libc::connect(fd, him as *const libc::sockaddr, len as u32)
}

#[no_mangle]
unsafe extern "system" fn JVM_Bind(fd: jint, him: *mut sockaddr, len: jint) -> jint {
    libc::bind(fd, him as *const libc::sockaddr, len as u32)
}

#[no_mangle]
unsafe extern "system" fn JVM_Accept(fd: jint, him: *mut sockaddr, len: *mut jint) -> jint {
    libc::accept(fd, him as *mut libc::sockaddr, len as *mut libc::socklen_t)
}

#[no_mangle]
unsafe extern "system" fn JVM_SocketAvailable(fd: jint, result: *mut jint) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetSockName(fd: jint, him: *mut sockaddr, len: *mut ::std::os::raw::c_int) -> jint {
    libc::getsockname(fd, him as *mut libc::sockaddr, len as *mut libc::socklen_t)
}

#[no_mangle]
unsafe extern "system" fn JVM_GetHostName(name: *mut ::std::os::raw::c_char, namelen: ::std::os::raw::c_int) -> ::std::os::raw::c_int {
    libc::gethostname(name, namelen as usize)
}

#[no_mangle]
unsafe extern "system" fn JVM_GetSockOpt(
    fd: jint,
    level: ::std::os::raw::c_int,
    optname: ::std::os::raw::c_int,
    optval: *mut ::std::os::raw::c_char,
    optlen: *mut ::std::os::raw::c_int,
) -> jint {
    libc::getsockopt(fd, level, optname, optval, optlen as *mut libc::socklen_t)
}

#[no_mangle]
unsafe extern "system" fn JVM_SetSockOpt(
    fd: jint,
    level: ::std::os::raw::c_int,
    optname: ::std::os::raw::c_int,
    optval: *const ::std::os::raw::c_char,
    optlen: ::std::os::raw::c_int,
) -> jint {
    libc::setsockopt(fd, level, optname, optval, optlen as u32)
}

