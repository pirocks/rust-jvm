use std::any::Any;
use std::mem::MaybeUninit;
use std::ptr::{NonNull, null_mut};
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use std::sync::Barrier;

use assert_no_alloc::*;
use libc::{c_int, c_void, siginfo_t, sigset_t, sigwaitinfo};
use nix::sys::signal::{Signal, SigSet};
use nonnull_const::NonNullConst;

use another_jit_vm::{MAGIC_1_EXPECTED, MAGIC_2_EXPECTED};
use gc_memory_layout_common::layout::{FRAME_HEADER_PREV_MAGIC_1_OFFSET, FRAME_HEADER_PREV_MAGIC_2_OFFSET};
use threads::signal::ucontext_t;

pub mod signal_safety;

#[cfg(debug_assertions)] // required when disable_release is set (default)
#[global_allocator]
static A: AllocDisabler = AllocDisabler;

pub struct SignalAccessibleJavaStackData {
    stack_top: *const c_void,
    stack_bottom: *const c_void,
    pub interpreter_should_safepoint_check: AtomicBool,
    // both get reset to null once answer received. only should be set when null
    pub remote_request: AtomicPtr<RemoteQueryInternal>,
    pub remote_request_answer: AtomicPtr<RemoteQueryAnswerInternal>,
    pub answer_written: AtomicBool,
    pub in_signal: AtomicBool,
}

impl SignalAccessibleJavaStackData {
    pub fn new(stack_top: *const c_void, stack_bottom: *const c_void) -> Self {
        Self {
            stack_top,
            stack_bottom,
            interpreter_should_safepoint_check: AtomicBool::new(false),
            remote_request: AtomicPtr::new(null_mut()),
            remote_request_answer: AtomicPtr::new(null_mut()),
            answer_written: AtomicBool::new(false),
            in_signal: AtomicBool::new(false),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum RemoteQuery<'l> {
    GetGuestFrameStackInstructionPointer {
        restart: &'l Barrier
    },
    GC,
}

impl RemoteQuery<'_> {
    pub fn to_remote_query_internal(&self) -> RemoteQueryInternal {
        match self {
            RemoteQuery::GetGuestFrameStackInstructionPointer { restart } => {
                let restart = *restart;
                RemoteQueryInternal::GetGuestFrameStackInstructionPointer { restart: NonNullConst::new(restart as *const Barrier).unwrap() }
            }
            RemoteQuery::GC => {
                RemoteQueryInternal::GC
            }
        }
    }

    pub fn restart_barrier(&self) -> Option<&Barrier> {
        match self {
            RemoteQuery::GetGuestFrameStackInstructionPointer { restart } => {
                Some(restart)
            }
            RemoteQuery::GC => {
                None
            }
        }
    }

    pub fn wait_for_next_signal(&self) -> bool {
        match self {
            RemoteQuery::GetGuestFrameStackInstructionPointer { .. } => {
                true
            }
            RemoteQuery::GC => {
                todo!()
            }
        }
    }
}

#[derive(Debug)]
pub enum RemoteQueryInternal {
    GetGuestFrameStackInstructionPointer {
        restart: NonNullConst<Barrier>
    },
    GC,
}

impl RemoteQueryInternal {
    pub fn to_remote_query<'l>(&'_ self) -> RemoteQuery<'l> {
        match self {
            RemoteQueryInternal::GetGuestFrameStackInstructionPointer { restart } => unsafe {
                RemoteQuery::GetGuestFrameStackInstructionPointer { restart: restart.as_ref() }
            }
            RemoteQueryInternal::GC => {
                RemoteQuery::GC
            }
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum GetGuestFrameStackInstructionPointer {
    InGuest {
        rbp: u64,
        rsp: u64,
        rip: u64,
    },
    InVM {
        rbp: u64,
        rsp: u64,
        rip: u64,
    },
    Transitioning {},
    FrameBeingCreated {
        rbp: u64,
        rsp: u64,
        rip: u64,
    },
}


pub enum RemoteQueryAnswerInternal {
    GetGuestFrameStackInstructionPointer {
        answer: GetGuestFrameStackInstructionPointer,
    },
    Panic(Box<dyn Any + Send>),
    Empty,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum RemoteQueryAnswer {
    GetGuestFrameStackInstructionPointer(GetGuestFrameStackInstructionPointer),
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

unsafe fn handler_impl(info: *mut siginfo_t, ucontext: Option<*const ucontext_t>) {
    let si_value = (info.as_ref().unwrap().si_value().sival_ptr as *const SignalAccessibleJavaStackData).as_ref().unwrap();
    assert!(!si_value.in_signal.load(Ordering::SeqCst));
    si_value.in_signal.store(true, Ordering::SeqCst);
    if let Err(err) = std::panic::catch_unwind(|| {
        assert!(!si_value.interpreter_should_safepoint_check.load(Ordering::SeqCst));
        assert!(!si_value.answer_written.load(Ordering::SeqCst));
        let answer = si_value.remote_request_answer.load(Ordering::SeqCst) as *mut RemoteQueryAnswerInternal;
        let answer = answer.as_mut().unwrap();
        let remote_request = match si_value.remote_request.load(Ordering::SeqCst).as_ref() {
            Some(x) => x,
            None => {
                eprintln!("No remote request was received");
                todo!()
            }
        };
        //todo no alloc
        let remote_query = remote_request.to_remote_query();
        *answer = get_answer_from_query(ucontext, si_value, remote_query);
        assert!(!si_value.answer_written.load(Ordering::SeqCst));
        si_value.answer_written.store(true, Ordering::SeqCst);
        if remote_query.wait_for_next_signal() {
            wait_for_next_signal()
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
        si_value.answer_written.store(true, Ordering::SeqCst);
    }
    si_value.in_signal.store(false, Ordering::SeqCst);
}

unsafe fn wait_for_next_signal() {
    let mut signal_set = SigSet::empty();
    signal_set.add(Signal::SIGUSR1);
    let mut siginfo: MaybeUninit<siginfo_t> = MaybeUninit::uninit();
    let err = sigwaitinfo(signal_set.as_ref() as *const sigset_t, siginfo.as_mut_ptr());
    if err == -1 {
        todo!()
    }
    // let mut siginfo = siginfo.assume_init();
    // handler_impl(&mut siginfo as *mut siginfo_t, None)
}

unsafe fn get_answer_from_query(ucontext: Option<*const ucontext_t>, si_value: &SignalAccessibleJavaStackData, remote_query: RemoteQuery) -> RemoteQueryAnswerInternal {
    match remote_query {
        RemoteQuery::GetGuestFrameStackInstructionPointer { .. } => {
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

            RemoteQueryAnswerInternal::GetGuestFrameStackInstructionPointer {
                answer: if transitioning {
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
                },
            }
        }
        RemoteQuery::GC => {
            todo!()
        }
    }
}