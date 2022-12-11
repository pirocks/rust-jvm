#![feature(box_syntax)]

use std::collections::HashMap;
use std::ffi::{c_int, c_void};
use std::mem::size_of;
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};

use assert_no_alloc::{AllocDisabler, assert_no_alloc};
use nix::libc;
use nix::libc::{REG_RAX, REG_RDI, REG_RIP, REG_RSP, RIP, RSP, siginfo_t, ucontext_t};
use nix::sys::pthread::{Pthread, pthread_sigqueue, SigVal};
use nix::sys::signal::{SaFlags, sigaction, SigAction, SigHandler, Signal, SigSet};
use nonnull_const::NonNullConst;

use another_jit_vm::saved_registers_utils::{SavedRegistersWithIP, SavedRegistersWithoutIP};

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

#[derive(Debug)]
pub enum ThingToPush<'signal_life> {
    PrevRBP,
    PrevSP,
    Data(&'signal_life [u8]),
}

#[derive(Debug)]
pub struct RemoteQueryUnsafe<'signal_life> {
    signal_safe_data: NonNullConst<SignalAccessibleJavaStackData>,
    to_push: &'signal_life [ThingToPush<'signal_life>],
    registers_to_set_to: SavedRegistersWithIP,
    okay_to_free_this: AtomicBool,
}

impl RemoteQueryUnsafe<'_> {
    pub fn signal_safe_data(&self) -> &SignalAccessibleJavaStackData {
        unsafe { self.signal_safe_data.as_ref() }
    }
}

pub const THREAD_PAUSE_SIGNAL: Signal = Signal::SIGUSR1;
pub const THREAD_PAUSE_SIGNAL_RAW: c_int = THREAD_PAUSE_SIGNAL as i32;

pub extern "C" fn handler(sig: c_int, info: *mut siginfo_t, ucontext: *mut c_void) {
    unsafe {
        let saved = libc::__errno_location().read();
        let sig_expected = THREAD_PAUSE_SIGNAL_RAW;
        if sig != sig_expected {
            eprintln!("unexpected signal");
            libc::abort()
        }
        assert_no_alloc(|| {
            handler_impl(info, Some(ucontext as *const ucontext_t))
        });
        libc::__errno_location().write(saved);
    }
}

unsafe fn handler_impl(info: *mut siginfo_t, mut ucontext: Option<*const ucontext_t>) {
    if let Err(_err) = std::panic::catch_unwind(|| {
        let si_value = (info.as_ref().unwrap().si_value().sival_ptr as *mut RemoteQueryUnsafe).as_mut().unwrap();
        let signal_safe_data = si_value.signal_safe_data();
        assert!(!signal_safe_data.in_signal.load(Ordering::SeqCst));
        signal_safe_data.in_signal.store(true, Ordering::SeqCst);
        let gpregs = &mut (ucontext.unwrap() as *mut ucontext_t).as_mut().unwrap().uc_mcontext.gregs;
        let prev_rip = gpregs[REG_RIP as usize];
        let stack = gpregs[REG_RSP as usize] as *mut c_void as *mut u64;
        stack.write(prev_rip as u64);
        gpregs[REG_RIP as usize] = si_value.registers_to_set_to.rip as u64 as i64;
        gpregs[REG_RDI as usize] = gpregs[REG_RAX as usize];
        gpregs[REG_RSP as usize] -= size_of::<u64>() as i64;
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
pub mod test {
    use std::ffi::c_void;
    use std::ptr::null_mut;
    use std::sync::atomic::AtomicBool;
    use std::time::Duration;

    use nix::libc;
    use nix::sys::pthread::pthread_self;
    use nonnull_const::NonNullConst;

    use another_jit_vm::saved_registers_utils::{SavedRegistersWithIP, SavedRegistersWithoutIP};

    use crate::{RemoteFramePush, RemoteQueryUnsafe, SignalAccessibleJavaStackData};

    extern "C" fn no_longer_in_handler(rax_in: u64) -> u64 {
        let saved = unsafe { libc::__errno_location().read() };
        std::thread::sleep(Duration::new(1,0));
        // loop {
            println!("not in handler");
        // }
        unsafe { libc::__errno_location().write(saved) }
        rax_in
    }

    #[test]
    pub fn test() {
        let remote_frame_push = RemoteFramePush::sigaction_setup();

        let (sender, receiver) = std::sync::mpsc::channel();
        let other_thread = std::thread::spawn(move || {
            let self_id = pthread_self();
            sender.send(self_id).unwrap();
            std::thread::sleep(Duration::new(10, 0));
        }).thread();
        let other_thread_id = receiver.recv().unwrap();
        let the_box = box RemoteQueryUnsafe {
            signal_safe_data: NonNullConst::new(Box::into_raw(box SignalAccessibleJavaStackData::new(null_mut(), null_mut()))).unwrap(),
            to_push: &[],
            okay_to_free_this: AtomicBool::new(false),
            registers_to_set_to: SavedRegistersWithIP {
                rip: no_longer_in_handler as *const c_void,
                saved_registers_without_ip: SavedRegistersWithoutIP {
                    rax: 0,
                    rbx: 0,
                    rcx: 0,
                    rdx: 0,
                    rsi: 0,
                    rdi: 0,
                    rbp: 0,
                    rsp: 0,
                    r8: 0,
                    r9: 0,
                    r10: 0,
                    r11: 0,
                    r12: 0,
                    r13: 0,
                    r14: 0,
                    xsave_area: [0; 64],
                },
            },
        };
        remote_frame_push.send_signal(other_thread_id, Box::into_raw(the_box));
        std::thread::sleep(Duration::new(10, 0));
    }
}