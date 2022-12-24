use std::mem::MaybeUninit;
use std::ptr::{NonNull, null_mut};
use std::sync::atomic::{AtomicBool, Ordering};

// use assert_no_alloc::*;
use libc::{c_int, c_void, siginfo_t, sigset_t, sigwaitinfo};
use nix::sys::signal::{SigmaskHow, Signal, sigprocmask, SigSet};

use another_jit_vm::{MAGIC_1_EXPECTED, MAGIC_2_EXPECTED};
use gc_memory_layout_common::frame_layout::{FRAME_HEADER_PREV_MAGIC_1_OFFSET, FRAME_HEADER_PREV_MAGIC_2_OFFSET};
use threads::signal::ucontext_t;

use crate::remote_queries::{GetGuestFrameStackInstructionPointer, RemoteQuerySafe, RemoteQueryUnsafe};

pub mod signal_safety;
pub mod remote_queries;

// #[cfg(debug_assertions)] // required when disable_release is set (default)
// #[global_allocator]
// static A: AllocDisabler = AllocDisabler;

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

pub const THREAD_PAUSE_SIGNAL: Signal = Signal::SIGUSR1;
pub const THREAD_RESTART_SIGNAL: Signal = Signal::SIGUSR2;
pub const THREAD_PAUSE_SIGNAL_RAW: c_int = THREAD_PAUSE_SIGNAL as i32;
pub const THREAD_RESTART_SIGNAL_RAW: c_int = THREAD_RESTART_SIGNAL as i32;


pub extern "C" fn handler(sig: c_int, info: *mut siginfo_t, ucontext: *mut c_void) {
    let sig_expected = THREAD_PAUSE_SIGNAL_RAW;
    unsafe {
        if sig != sig_expected {
            eprintln!("unexpected signal");
            libc::abort()
        }
        // assert_no_alloc(|| {
            handler_impl(info, Some(ucontext as *const ucontext_t))
        // });
    };
}

unsafe fn handler_impl(info: *mut siginfo_t, ucontext: Option<*const ucontext_t>) {
    if let Err(_err) = std::panic::catch_unwind(|| {
        let si_value = (info.as_ref().unwrap().si_value().sival_ptr as *mut RemoteQueryUnsafe).as_mut().unwrap();
        let signal_safe_data = si_value.signal_safe_data();
        assert!(!signal_safe_data.in_signal.load(Ordering::SeqCst));
        signal_safe_data.in_signal.store(true, Ordering::SeqCst);
        let remote_query = si_value.to_remote_query();
        assert!(!signal_safe_data.interpreter_should_safepoint_check.load(Ordering::SeqCst));
        handle_query(ucontext, signal_safe_data, remote_query);
        signal_safe_data.in_signal.store(false, Ordering::SeqCst);
    }) {
        eprintln!("panic in signal handler");
        libc::abort();
    }
}

unsafe fn wait_for_restart_signal() {
    let mut signal_set = SigSet::empty();
    signal_set.add(THREAD_RESTART_SIGNAL);
    let mut siginfo: MaybeUninit<siginfo_t> = MaybeUninit::uninit();
    let err = sigwaitinfo(signal_set.as_ref() as *const sigset_t, siginfo.as_mut_ptr());
    assert_eq!(err, THREAD_RESTART_SIGNAL_RAW);
    if err == -1 {
        todo!()
    }
    assert_eq!(siginfo.assume_init_read().si_value().sival_ptr, null_mut());
}

unsafe fn handle_query(ucontext: Option<*const ucontext_t>, si_value: &SignalAccessibleJavaStackData, remote_query: RemoteQuerySafe) {
    match remote_query {
        RemoteQuerySafe::GetGuestFrameStackInstructionPointer { answer: answer_mut, answer_written } => {
            let ucontext = (ucontext.unwrap() as *const ucontext_t).as_ref().unwrap();
            let general_purpose_regs = ucontext.uc_mcontext.gregs;
            let stack_pointer = general_purpose_regs[threads::signal::REG_RSP as usize];
            let frame_pointer = general_purpose_regs[threads::signal::REG_RBP as usize];
            let instruction_pointer = general_purpose_regs[threads::signal::REG_RIP as usize];
            let stack_top = si_value.stack_top;
            let stack_bottom = si_value.stack_bottom;
            assert!(stack_top > stack_bottom);
            let stack_pointer_in_stack = (stack_pointer as *const c_void) < stack_top && (stack_pointer as *const c_void) > stack_bottom;
            let frame_pointer_in_stack = (frame_pointer as *const c_void) < stack_top && (frame_pointer as *const c_void) > stack_bottom;
            let in_guest = stack_pointer_in_stack && frame_pointer_in_stack;
            let transitioning = stack_pointer_in_stack ^ frame_pointer_in_stack;
            let answer = if transitioning {
                assert!(!in_guest);
                GetGuestFrameStackInstructionPointer::Transitioning {}
            } else if in_guest {
                //check for magics so that we know this frame isn't being created
                let frame_pointer_raw = frame_pointer;
                let frame_pointer = NonNull::new(frame_pointer as *mut c_void).unwrap();
                let magic1_ptr = frame_pointer.as_ptr().sub(FRAME_HEADER_PREV_MAGIC_1_OFFSET) as *const u64;
                let magic2_ptr = frame_pointer.as_ptr().sub(FRAME_HEADER_PREV_MAGIC_2_OFFSET) as *const u64;
                let magic_1 = magic1_ptr.read();
                let magic_2 = magic2_ptr.read();
                if magic_1 != MAGIC_1_EXPECTED || magic_2 != MAGIC_2_EXPECTED {
                    GetGuestFrameStackInstructionPointer::FrameBeingCreated {
                        rbp: frame_pointer_raw as u64,
                        rsp: stack_pointer as u64,
                        rip: instruction_pointer as u64,
                    }
                } else {
                    GetGuestFrameStackInstructionPointer::InGuest {
                        rbp: frame_pointer_raw as u64,
                        rsp: stack_pointer as u64,
                        rip: instruction_pointer as u64,
                    }
                }
            } else {
                GetGuestFrameStackInstructionPointer::InVM {
                    rbp: frame_pointer as u64,
                    rsp: stack_pointer as u64,
                    rip: instruction_pointer as u64,
                }
            };
            let mut sig_set = SigSet::empty();
            sig_set.add(THREAD_RESTART_SIGNAL);
            sigprocmask(SigmaskHow::SIG_BLOCK, Some(&sig_set),None).unwrap();
            *answer_mut = Some(answer);
            answer_written.store(true, Ordering::SeqCst);
            wait_for_restart_signal()
        }
        RemoteQuerySafe::GC => {
            todo!()
        }
        RemoteQuerySafe::RestartFromGetGuestFrameStackInstructionPointer => {}
    }
}