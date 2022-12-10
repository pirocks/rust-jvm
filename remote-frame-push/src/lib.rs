use std::collections::HashMap;
use std::ffi::{c_int, c_void};
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use nix::libc;
use nix::libc::{siginfo_t, ucontext_t};
use nix::sys::pthread::{Pthread, pthread_sigqueue, SigVal};
use nix::sys::signal::{SaFlags, sigaction, SigAction, SigHandler, Signal, SigSet};
use thread_signal_handler::SignalAccessibleJavaStackData;

#[cfg(debug_assertions)] // required when disable_release is set (default)
#[global_allocator]
static A: AllocDisabler = AllocDisabler;


pub struct SignalAccessibleJavaStackData {
    stack_top: *const c_void,
    stack_bottom: *const c_void,
    pub interpreter_should_safepoint_check: AtomicBool,
    pub in_signal: AtomicBool,
}

impl SignalAccessibleJavaStackData {
    pub fn new(stack_top: *const c_void, stack_bottom: *const c_void) -> Self {
        Self {
            stack_top,
            stack_bottom,
            interpreter_should_safepoint_check: AtomicBool::new(false),
            in_signal: AtomicBool::new(false),
        }
    }
}

pub enum ThingToPush<'signal_life>{
    PrevRBP,
    PrevSP,
    Data(&'signal_life [u8])
}

#[derive(Debug)]
pub struct RemoteQueryUnsafe<'signal_life> {
    signal_safe_data: NonNullConst<SignalAccessibleJavaStackData>,
    to_push: &'signal_life [ThingToPush<'signal_life>],
    new_ip: *mut c_void,
    okay_to_free_this: AtomicBool
}

impl RemoteQueryUnsafe{
    pub fn signal_safe_data(&self) -> &SignalAccessibleJavaStackData{
        self.signal_safe_data
    }
}

pub const THREAD_PAUSE_SIGNAL: Signal = Signal::SIGUSR1;
pub const THREAD_PAUSE_SIGNAL_RAW: c_int = THREAD_PAUSE_SIGNAL as i32;

pub extern "C" fn handler(sig: c_int, info: *mut siginfo_t, ucontext: *mut c_void) {
    let sig_expected = THREAD_PAUSE_SIGNAL_RAW;
    unsafe {
        if sig != sig_expected {
            eprintln!("unexpected signal");
            libc::abort()
        }
        assert_no_alloc(|| {
            handler_impl(info, Some(ucontext as *const ucontext_t))
        });
    };
}

unsafe fn handler_impl(info: *mut siginfo_t, mut ucontext: Option<*const ucontext_t>) {
    if let Err(_err) = std::panic::catch_unwind(|| {
        let si_value = (info.as_ref().unwrap().si_value().sival_ptr as *mut RemoteQueryUnsafe).as_mut().unwrap();
        let signal_safe_data = si_value.signal_safe_data();
        assert!(!signal_safe_data.in_signal.load(Ordering::SeqCst));
        signal_safe_data.in_signal.store(true, Ordering::SeqCst);
        (ucontext.unwrap() as *mut ucontext_t).as_mut().unwrap().uc_mcontext.gregs = [0;23];
        signal_safe_data.in_signal.store(false, Ordering::SeqCst);
    }) {
        eprintln!("panic in signal handler");
        libc::abort();
    }
}



pub struct RemoteFramePush {
    per_thread_signal_lock: RwLock<HashMap<Pthread, Arc<Mutex<()>>>>,
}

impl RemoteFramePush {
    fn thread_signal_lock(&self, tid: Pthread) -> Arc<Mutex<()>> {
        self.per_thread_signal_lock.write().unwrap().entry(tid).or_default().clone()
    }

    fn send_signal(&self, tid: Pthread, data: *mut RemoteQueryUnsafe) {
        pthread_sigqueue(tid, Some(THREAD_PAUSE_SIGNAL), SigVal::Ptr(data as *mut c_void)).unwrap();
    }

    pub fn sigaction_setup() -> Self {
        let mut signal_set = SigSet::empty();
        signal_set.add(THREAD_PAUSE_SIGNAL);
        let sig_handler = SigHandler::SigAction(handler);
        let _old_sigaction = unsafe { sigaction(THREAD_PAUSE_SIGNAL, &SigAction::new(sig_handler, SaFlags::SA_SIGINFO, signal_set)).unwrap() };
        Self { per_thread_signal_lock: RwLock::new(HashMap::new()) }
    }
}

#[cfg(test)]
pub mod test{
    use crate::RemoteFramePush;

    #[test]
    pub fn test() {
        RemoteFramePush::sigaction_setup();

        let other_thread = std::thread::spawn(||{
            loop {
            }
        }).thread();
        other_thread.id().
    }
}