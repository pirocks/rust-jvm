use nix::sys::signal::{SigAction, SigHandler, SigSet, kill};
use crate::{JVMState, JavaThread, ThreadId};
use nix::sys::signal::Signal;
use nix::sys::signal::SaFlags;
use std::convert::{TryInto, TryFrom};
use std::mem::transmute;
use std::sync::atomic::AtomicPtr;
use std::ptr::null_mut;
use crate::signal::{sigqueue, sigval};
use std::ffi::c_void;

pub enum SignalReason<'l> {
    JVMTIEvent(&'l JVMState),
}


impl JVMState {
    pub fn init_signal_handler(&self) {
        SigAction::new(SigHandler::SigAction(handler), unsafe { transmute(0 as libc::c_int) }, SigSet::all());
    }

    pub unsafe fn trigger_signal(&self, jvm: &JVMState, t: JavaThread) {
        let metadata_void_ptr = Box::leak(box SignalReason::JVMTIEvent(jvm)) as *mut c_void;
        let mut sigval_ = sigval {};
        sigval_.sival_ptr = metadata_void_ptr;
        let res = sigqueue(t.unix_tid.as_raw(), transmute(Signal::SIGUSR1), sigval_);
        if res != 0 {
            panic!()
        }
    }
}

unsafe extern fn handler(signal_number: libc::c_int, siginfo: *mut libc::siginfo_t, data: *mut libc::c_void) {
    assert_eq!(Signal::try_from(signal_number).unwrap(), Signal::SIGUSR1);
    let reason = (data as *mut SignalReason).read();
    unimplemented!()
}
