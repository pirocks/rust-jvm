use std::time::Instant;

use nix::errno::Errno::EINTR;
use nix::sys::socket::setsockopt;

use classfile_parser::code::InstructionTypeNum::{lookupswitch, ret};
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

fn retry_on_eintr(to_retry: impl Fn() -> i32) -> i32 {
    loop {
        let err = to_retry();
        if nix::errno::errno() != EINTR || err != -1 {
            return err;
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_Recv(fd: jint, buf: *mut ::std::os::raw::c_char, nBytes: jint, flags: jint) -> jint {
    retry_on_eintr(|| libc::recv(fd, buf, nBytes as usize, flags) as i32)
}

#[no_mangle]
unsafe extern "system" fn JVM_Send(fd: jint, buf: *mut ::std::os::raw::c_char, nBytes: jint, flags: jint) -> jint {
    retry_on_eintr(|| libc::send(fd, buf, nBytes as usize, flags) as i32)
}

#[no_mangle]
unsafe extern "system" fn JVM_Timeout(fd: ::std::os::raw::c_int, timeout: ::std::os::raw::c_long) -> jint {
    let start = Instant::now();
    loop {
        let mut pollfd = libc::pollfd {
            fd,
            events: libc::POLLIN | libc::POLLERR,
            revents: 0,
        };
        let err = libc::poll(&mut pollfd as *mut libc::pollfd, 1, timeout as i32);
        if nix::errno::errno() == EINTR && err == -1 {
            if timeout >= 0 {
                if Instant::now().duration_since(start).as_millis() >= timeout as u128 {
                    return 0;
                }
            }
        } else {
            return err
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_Listen(fd: jint, count: jint) -> jint {
    libc::listen(fd, count)
}

#[no_mangle]
unsafe extern "system" fn JVM_Connect(fd: jint, him: *const sockaddr, len: jint) -> jint {
    retry_on_eintr(|| libc::connect(fd, him as *const libc::sockaddr, len as u32))
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
    //mostly stolen from os::socket_available in hotspot
    if libc::ioctl(fd, FIONREAD, result) < 0 {
        1
    } else {
        0
    }
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

