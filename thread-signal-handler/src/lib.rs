use std::any::Any;
use std::ptr::{NonNull, null_mut};
use std::sync::{Barrier};
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

use assert_no_alloc::*;
use libc::{c_int, c_void, siginfo_t};
use nix::sys::signal::Signal;
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
    pub remote_request: AtomicPtr<RemoteQuery>,
    pub remote_request_answer: AtomicPtr<RemoteQueryAnswerInternal>,
    pub answer_written: AtomicBool,
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
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum RemoteQuery {
    GetGuestFrameStackInstructionPointer,
    GC,
}

pub enum RemoteQueryInternal {
    GetGuestFrameStackInstructionPointer {
        restart: Barrier
    },
    GC,
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
            handler_impl(info, ucontext)
        });
    };
}

unsafe fn handler_impl(info: *mut siginfo_t, ucontext: *mut c_void) {
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
        //todo no alloc
        match remote_request {
            RemoteQuery::GetGuestFrameStackInstructionPointer => {
                let ucontext = (ucontext as *const ucontext_t).as_ref().unwrap();
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

                *answer = RemoteQueryAnswerInternal::GetGuestFrameStackInstructionPointer {
                    answer: if transitioning {
                        assert!(!in_guest);
                        GetGuestFrameStackInstructionPointer::Transitioning {}
                    } else if in_guest {
                        //check for magics so that we know this frame isn't being created
                        let frame_pointer_raw = frame_pointer;
                        let frame_pointer = NonNull::new(frame_pointer as *mut c_void).unwrap();
                        let magic1_ptr = frame_pointer.as_ptr().sub(FRAME_HEADER_PREV_MAGIC_1_OFFSET) as *const u64;
                        let magic2_ptr = frame_pointer.as_ptr().sub(FRAME_HEADER_PREV_MAGIC_2_OFFSET) as *const u64;
                        if magic1_ptr.read() != MAGIC_1_EXPECTED || magic2_ptr.read() != MAGIC_2_EXPECTED{
                            GetGuestFrameStackInstructionPointer::FrameBeingCreated {
                                rbp: frame_pointer_raw as u64,
                                rsp: stack_pointer as u64,
                                rip: instruction_pointer as u64
                            }
                        }else {
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
                };
            }
            RemoteQuery::GC => {
                todo!()
            }
        }
        si_value.answer_written.store(true, Ordering::SeqCst);
        // restart_barrier.wait()
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
}