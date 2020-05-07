use nix::sys::signal::{SigAction, SigHandler, SigSet, sigaction};
use crate::{JVMState, JavaThread};
use nix::sys::signal::Signal;
use std::convert::TryFrom;
use std::mem::transmute;
use crate::signal::{sigqueue, sigval, SA_SIGINFO};
use std::ffi::c_void;
use crate::jvmti::event_callbacks::{JVMTIEvent, DebuggerEventConsumer};
use nix::errno::errno;

pub struct JVMTIEventData<'l> {
    pub event: JVMTIEvent,
    pub jvm: &'l JVMState,
}

pub enum SignalReason<'l> {
    JVMTIEvent(JVMTIEventData<'l>),
}


impl JVMState {
    pub fn init_signal_handler(&self) {
        unsafe {
            let sa = SigAction::new(SigHandler::SigAction(handler), transmute(SA_SIGINFO as libc::c_int), SigSet::all());
            sigaction(Signal::SIGUSR1, &sa).unwrap();
        };
    }

    pub fn trigger_jvmti_event(&self, t: &JavaThread, event: JVMTIEvent) {
        let reason = SignalReason::JVMTIEvent(JVMTIEventData { event, jvm: &self });//todo lifetime during vm death?
        unsafe { self.trigger_signal(t, reason) }
    }

    pub unsafe fn trigger_signal(&self, t: &JavaThread, reason: SignalReason) {
        let metadata_void_ptr = Box::leak(box reason) as *mut SignalReason as *mut c_void;
        let sigval_ = sigval { sival_ptr: metadata_void_ptr };
        let res = sigqueue(t.unix_tid.as_raw(), transmute(Signal::SIGUSR1), sigval_);
        if res != 0 {
            dbg!(errno());
            panic!()
        }
    }
}

extern fn handler(signal_number: libc::c_int, _siginfo: *mut libc::siginfo_t, data: *mut libc::c_void) {
    assert_eq!(Signal::try_from(signal_number).unwrap(), Signal::SIGUSR1);
    let reason = unsafe { (data as *mut SignalReason).read() };
    match reason {
        SignalReason::JVMTIEvent(jvmti_data) => {
            let JVMTIEventData { event, jvm } = jvmti_data;
            match event {
                JVMTIEvent::VMInit(init) => {
                    unsafe { jvm.jvmti_state.built_in_jdwp.VMInit(jvm, init) }
                }
                JVMTIEvent::ThreadStart(thread_start) => {
                    unsafe { jvm.jvmti_state.built_in_jdwp.ThreadStart(jvm, thread_start) }
                }
                JVMTIEvent::Breakpoint(breakpoint) => {
                    unsafe { jvm.jvmti_state.built_in_jdwp.Breakpoint(jvm, breakpoint) }
                }
                JVMTIEvent::ClassPrepare(classprepare) => {
                    unsafe { jvm.jvmti_state.built_in_jdwp.ClassPrepare(jvm, classprepare) }
                }
            }
        }
    }
    unimplemented!()
}
