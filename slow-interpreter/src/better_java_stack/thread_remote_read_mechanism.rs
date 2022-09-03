use std::any::Any;
use std::hint;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

use libc::{c_int, c_void};
use nix::sys::pthread::{Pthread, pthread_sigqueue, SigVal};
use nix::sys::signal::{SaFlags, SigAction, sigaction, SigHandler, Signal, SigSet};

use threads::signal::ucontext_t;

pub struct SignalAccessibleJavaStackData {
    pub(crate) interpreter_should_safepoint_check: AtomicBool,
    in_guest: AtomicBool,
    // both get reset to null once answer received. only should be set when null
    remote_request: AtomicPtr<RemoteQuery>,
    remote_request_answer: AtomicPtr<RemoteQueryAnswerInternal>,
    signal_handling_done: AtomicBool,
}

impl SignalAccessibleJavaStackData {
    pub fn new() -> Self {
        Self {
            interpreter_should_safepoint_check: AtomicBool::new(false),
            in_guest: AtomicBool::new(false),
            remote_request: AtomicPtr::new(null_mut()),
            remote_request_answer: AtomicPtr::new(null_mut()),
            signal_handling_done: AtomicBool::new(false),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum RemoteQuery {
    GetGuestFrameStackInstructionPointer,
    GC,
}

pub enum RemoteQueryAnswerInternal {
    GetGuestFrameStackInstructionPointer {
        rbp: u64,
        rsp: u64,
        rip: u64,
    },
    Panic(Box<dyn Any + Send>),
    Empty,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum RemoteQueryAnswer {
    GetGuestFrameStackInstructionPointer {
        rbp: u64,
        rsp: u64,
        rip: u64,
    }
}

pub const THREAD_PAUSE_SIGNAL: Signal = Signal::SIGUSR1;
pub const THREAD_PAUSE_SIGNAL_RAW: c_int = THREAD_PAUSE_SIGNAL as i32;


extern "C" fn handler(sig: c_int, info: *mut nix::libc::siginfo_t, ucontext: *mut c_void) {
    let sig_expected = THREAD_PAUSE_SIGNAL_RAW;
    unsafe {
        if sig != sig_expected {
            eprintln!("unexpected signal");
            libc::abort()
        }
        handler_impl(info, ucontext)
    };
}

unsafe fn handler_impl(info: *mut nix::libc::siginfo_t, ucontext: *mut c_void) {
    let si_value = (info.as_ref().unwrap().si_value().sival_ptr as *const SignalAccessibleJavaStackData).as_ref().unwrap();
    if let Err(err) = std::panic::catch_unwind(|| {
        assert!(!si_value.interpreter_should_safepoint_check.load(Ordering::SeqCst));
        let answer = si_value.remote_request_answer.load(Ordering::SeqCst) as *mut RemoteQueryAnswerInternal;
        let answer = answer.as_mut().unwrap();
        let remote_request = match si_value.remote_request.load(Ordering::SeqCst).as_ref() {
            Some(x) => x,
            None => {
                eprintln!("No remote request was received");
                todo!()
            }
        };
        match remote_request {
            RemoteQuery::GetGuestFrameStackInstructionPointer => {
                let ucontext = (ucontext as *const ucontext_t).as_ref().unwrap();
                let general_purpose_regs = ucontext.uc_mcontext.gregs;
                let stack_pointer = general_purpose_regs[threads::signal::REG_RSP as usize];
                let frame_pointer = general_purpose_regs[threads::signal::REG_RBP as usize];
                let instruction_pointer = general_purpose_regs[threads::signal::REG_RIP as usize];
                *answer = RemoteQueryAnswerInternal::GetGuestFrameStackInstructionPointer {
                    rbp: frame_pointer as u64,
                    rsp: stack_pointer as u64,
                    rip: instruction_pointer as u64,
                };
            }
            RemoteQuery::GC => {
                todo!()
            }
        }
    }) {
        let answer = si_value.remote_request_answer.load(Ordering::SeqCst) as *mut RemoteQueryAnswerInternal;
        let answer = match answer.as_mut() {
            Some(x) => x,
            None => {
                eprintln!("unable to forward panic in signal");
                libc::abort();
            }
        };
        *answer = RemoteQueryAnswerInternal::Panic(err);
    }
    si_value.signal_handling_done.store(true, Ordering::SeqCst);
}

pub fn perform_remote_query(tid: Pthread, mut remote_query: RemoteQuery, signal_data: &SignalAccessibleJavaStackData) -> RemoteQueryAnswer {
    let remote_query_mut = &mut remote_query;
    let alloc_remote_query_raw = remote_query_mut as *mut RemoteQuery;
    while let Err(old) = signal_data.remote_request.compare_exchange(null_mut(), alloc_remote_query_raw, Ordering::SeqCst, Ordering::SeqCst) {
        dbg!("pending query:");
        dbg!(old);
        unsafe { dbg!(old.as_ref().unwrap()); }
        hint::spin_loop();
    }
    assert_eq!(signal_data.remote_request_answer.load(Ordering::SeqCst), null_mut());
    let mut answer = RemoteQueryAnswerInternal::Empty;
    let remote_query_mut = &mut answer;
    let raw_remote_query = remote_query_mut as *mut RemoteQueryAnswerInternal;
    signal_data.remote_request_answer.compare_exchange(null_mut(), raw_remote_query, Ordering::SeqCst, Ordering::SeqCst).unwrap();
    assert!(!signal_data.signal_handling_done.load(Ordering::SeqCst));
    pthread_sigqueue(tid, Some(THREAD_PAUSE_SIGNAL), SigVal::Ptr(signal_data as *const SignalAccessibleJavaStackData as *mut c_void)).unwrap();
    while signal_data.signal_handling_done.load(Ordering::SeqCst) != true {
        hint::spin_loop();
    }
    signal_data.signal_handling_done.store(false, Ordering::SeqCst);
    signal_data.remote_request_answer.compare_exchange(raw_remote_query, null_mut(), Ordering::SeqCst, Ordering::SeqCst).unwrap();
    signal_data.remote_request.compare_exchange(alloc_remote_query_raw, null_mut(), Ordering::SeqCst, Ordering::SeqCst).unwrap();
    match answer {
        RemoteQueryAnswerInternal::GetGuestFrameStackInstructionPointer { rbp, rsp, rip } => {
            RemoteQueryAnswer::GetGuestFrameStackInstructionPointer {
                rbp,
                rsp,
                rip,
            }
        }
        RemoteQueryAnswerInternal::Panic(panic_data) => {
            std::panic::resume_unwind(panic_data)
        }
        RemoteQueryAnswerInternal::Empty => {
            todo!("handle unhandled signals")
        }
    }
}

pub struct ThreadSignalBasedInterrupter {}

pub fn sigaction_setup() -> ThreadSignalBasedInterrupter {
    let mut signal_set = SigSet::empty();
    signal_set.add(THREAD_PAUSE_SIGNAL);
    let sig_handler = SigHandler::SigAction(handler);
    let _old_sigaction = unsafe { sigaction(THREAD_PAUSE_SIGNAL, &SigAction::new(sig_handler, SaFlags::SA_SIGINFO, signal_set)).unwrap() };
    ThreadSignalBasedInterrupter {}
}

#[cfg(test)]
pub mod test {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::thread;

    use nix::sys::pthread::pthread_self;
    use once_cell::sync::OnceCell;

    use crate::better_java_stack::thread_remote_read_mechanism::{perform_remote_query, RemoteQuery, sigaction_setup, SignalAccessibleJavaStackData};

    #[test]
    pub fn test() {
        sigaction_setup();
        let tid = OnceCell::new();
        let answered = Arc::new(AtomicBool::new(false));
        let signal_accessible = SignalAccessibleJavaStackData::new();
        thread::scope(|scope| {
            scope.spawn(|| {
                tid.set(pthread_self()).unwrap();
                let arc = answered.clone();
                while arc.load(Ordering::SeqCst) != true {
                    std::hint::spin_loop();
                }
            });
            scope.spawn(|| {
                let tid = *tid.wait();
                let _remote_query_answer = perform_remote_query(tid, RemoteQuery::GetGuestFrameStackInstructionPointer, &signal_accessible);
            }).join().unwrap();
            answered.store(true, Ordering::SeqCst);
        })
    }
}