use jni_bindings::{jint, sockaddr};

#[no_mangle]
unsafe extern "system" fn JVM_InitializeSocketLibrary() -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_Socket(domain: jint, type_: jint, protocol: jint) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_SocketClose(fd: jint) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_SocketShutdown(fd: jint, howto: jint) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_Recv(fd: jint, buf: *mut ::std::os::raw::c_char, nBytes: jint, flags: jint) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_Send(fd: jint, buf: *mut ::std::os::raw::c_char, nBytes: jint, flags: jint) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_Timeout(fd: ::std::os::raw::c_int, timeout: ::std::os::raw::c_long) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_Listen(fd: jint, count: jint) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_Connect(fd: jint, him: *mut sockaddr, len: jint) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_Bind(fd: jint, him: *mut sockaddr, len: jint) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_Accept(fd: jint, him: *mut sockaddr, len: *mut jint) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_SocketAvailable(fd: jint, result: *mut jint) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetSockName(fd: jint, him: *mut sockaddr, len: *mut ::std::os::raw::c_int) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetHostName(name: *mut ::std::os::raw::c_char, namelen: ::std::os::raw::c_int) -> ::std::os::raw::c_int {
    unimplemented!()
}

