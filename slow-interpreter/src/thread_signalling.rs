use nix::sys::signal::{SigAction, SigHandler, SigSet, sigaction};
use crate::{JVMState, JavaThread, signal};
use nix::sys::signal::Signal;
use std::convert::TryFrom;
use std::mem::transmute;
use crate::signal::{sigval, siginfo_t, SI_QUEUE, siginfo_t__bindgen_ty_1, siginfo_t__bindgen_ty_1__bindgen_ty_3, pthread_sigqueue, pthread_self};
use std::ffi::c_void;
use crate::jvmti::event_callbacks::{JVMTIEvent, DebuggerEventConsumer};
use nix::errno::errno;
use nix::unistd::{gettid, getuid};
use std::ptr::{null_mut};

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
            let sa = SigAction::new(SigHandler::SigAction(handler), transmute(0 as libc::c_int), SigSet::empty());
            // println!("sigaction");
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
        // let pid = getpid().as_raw();
        let tid = t.unix_tid.as_raw();

        if gettid().as_raw() != tid {
            let res = pthread_sigqueue(pthread_self(), transmute(Signal::SIGUSR1), sigval_);//rt_tgsigqueueinfo(pid, tid, transmute(Signal::SIGUSR1), Box::leak(box signal_info));//todo use after free?
            if res != 0 {
                dbg!(gettid());
                dbg!(errno());
                dbg!(res);
                panic!()
            }
        }else {
            let signal_info = siginfo_t {
                si_signo: transmute(Signal::SIGUSR1),
                si_errno: 0,
                si_code: SI_QUEUE,
                __pad0: 0,
                _sifields: siginfo_t__bindgen_ty_1 {
                    _rt: siginfo_t__bindgen_ty_1__bindgen_ty_3 {
                        si_pid: tid,
                        si_uid: getuid().as_raw(),
                        si_sigval: sigval_
                    }
                }
            };
            handler(transmute(Signal::SIGUSR1),Box::leak(box signal_info) as *mut signal::siginfo_t as *mut libc::c_void as *mut libc::siginfo_t,null_mut());
        }
    }
}
/*

const RT_TGSIGQUEUEINFO_SYSCALL_NUM: usize = 297;

unsafe extern "C" fn rt_tgsigqueueinfo( tgid: libc::pid_t, tid: libc::pid_t, sig: libc::c_int,  uinfo: *mut siginfo_t) -> libc::c_int {
    syscall::syscall4(RT_TGSIGQUEUEINFO_SYSCALL_NUM, tgid as usize, tid as usize, sig as usize, transmute(uinfo)) as i32
}
*/

extern fn handler(signal_number: libc::c_int, siginfo: *mut libc::siginfo_t, _data: *mut libc::c_void) {
    assert_eq!(Signal::try_from(signal_number).unwrap(), Signal::SIGUSR1);
    let reason = unsafe {
        let siginfo_signals_h = (siginfo as *mut siginfo_t).read();
        let signal_reason_ptr = siginfo_signals_h._sifields._rt.si_sigval.sival_ptr;
        assert_ne!(signal_reason_ptr, null_mut());
        // assert_eq!(siginfo_signals_h.si_code, SI_QUEUE);
        (signal_reason_ptr as *mut SignalReason).read()
    };
    match reason {
        SignalReason::JVMTIEvent(jvmti_data) => {
            let JVMTIEventData { event, jvm } = jvmti_data;
            match event {
                JVMTIEvent::VMInit(init) => {
                    unsafe { jvm.jvmti_state.as_ref().unwrap().built_in_jdwp.VMInit(jvm, init) }
                }
                JVMTIEvent::ThreadStart(thread_start) => {
                    unsafe { jvm.jvmti_state.as_ref().unwrap().built_in_jdwp.ThreadStart(jvm, thread_start) }
                }
                JVMTIEvent::Breakpoint(breakpoint) => {
                    unsafe { jvm.jvmti_state.as_ref().unwrap().built_in_jdwp.Breakpoint(jvm, breakpoint) }
                }
                JVMTIEvent::ClassPrepare(classprepare) => {
                    unsafe { jvm.jvmti_state.as_ref().unwrap().built_in_jdwp.ClassPrepare(jvm, classprepare) }
                }
            }
        }
    }
}
