use std::arch::asm;
use std::collections::HashMap;
use std::ffi::{c_int, c_void};
use std::mem::{MaybeUninit, size_of};
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(debug_assertions)]
use assert_no_alloc::AllocDisabler;
use assert_no_alloc::assert_no_alloc;
use libc::REG_R10;
use nix::libc;
use nix::libc::{greg_t, REG_RBP, REG_RIP, REG_RSP, setcontext, siginfo_t, ucontext_t};
use nix::sys::pthread::{Pthread, pthread_sigqueue, SigVal};
use nix::sys::signal::{SaFlags, sigaction, SigAction, SigHandler, Signal, SigSet};
use nonnull_const::NonNullConst;

use another_jit_vm::{R10_SIGNAL_GUEST_OFFSET_CONST, R11_SIGNAL_GUEST_OFFSET_CONST, R12_SIGNAL_GUEST_OFFSET_CONST, R13_SIGNAL_GUEST_OFFSET_CONST, R14_SIGNAL_GUEST_OFFSET_CONST, R8_SIGNAL_GUEST_OFFSET_CONST, R9_SIGNAL_GUEST_OFFSET_CONST, RAX_SIGNAL_GUEST_OFFSET_CONST, RBP_SIGNAL_GUEST_OFFSET_CONST, RBX_SIGNAL_GUEST_OFFSET_CONST, RCX_SIGNAL_GUEST_OFFSET_CONST, RDI_SIGNAL_GUEST_OFFSET_CONST, RDX_SIGNAL_GUEST_OFFSET_CONST, RIP_NATIVE_OFFSET_CONST, RSI_SIGNAL_GUEST_OFFSET_CONST, RSP_SIGNAL_GUEST_OFFSET_CONST};
use another_jit_vm_ir::vm_exit_abi::runtime_input::RawVMExitType;

#[cfg(debug_assertions)] // required when disable_release is set (default)
#[global_allocator]
static A: AllocDisabler = AllocDisabler;

// need try push java frame

pub const EXIT_NUMBER: u64 = RawVMExitType::SavepointRemoteExit as u64;

//        assembler.set_label(before_exit_label).unwrap();
//         for register in registers_to_save {
//             assembler.mov(r15 + register.guest_offset_const(), register.to_native_64()).unwrap();
//         }
//         assembler.mov(r15 + RBP_GUEST_OFFSET_CONST, rbp).unwrap();
//         assembler.mov(r15 + RSP_GUEST_OFFSET_CONST, rsp).unwrap();
//         assembler.lea(r10, qword_ptr(*before_exit_label)).unwrap();//safe to clober r10 b/c it was saved
//         assembler.mov(r15 + RIP_GUEST_OFFSET_CONST, r10).unwrap();
//         assembler.jmp(qword_ptr(r15 + RIP_NATIVE_OFFSET_CONST)).unwrap();
//         assembler.set_label(after_exit_label).unwrap();


#[no_mangle]
#[allow(named_asm_labels)]
#[naked]
pub unsafe extern "system" fn exit_to_safepoint_check() {
    //save all registers
    // remote query in r10
    //doesn't need reentrance b/c signaling is guarded by a lock
    //but does need to not take any space on stack b/c there might be frames below this one, like exception init frames
    // only needs to save r10 b/c the others will be restored by setcontext?
    //a different save area is needed for handling signals b/c otherwise r10 could overwrite a existing guest r10
    asm!(
    "mov rax, {__rust_jvm_exit_number}",//todo need some sort of assert that this return to value stays in sync with the one in the exit struct
    "mov [r15 + {__rust_jvm_rax_signal_guest_offset_const}], rax",
    "mov [r15 + {__rust_jvm_rbx_signal_guest_offset_const}], rbx",
    "mov [r15 + {__rust_jvm_rcx_signal_guest_offset_const}], rcx",
    "mov [r15 + {__rust_jvm_rdx_signal_guest_offset_const}], rdx",
    "mov [r15 + {__rust_jvm_rsi_signal_guest_offset_const}], rsi",
    "mov [r15 + {__rust_jvm_rdi_signal_guest_offset_const}], rdi",
    "mov [r15 + {__rust_jvm_rbp_signal_guest_offset_const}], rbp",
    "mov [r15 + {__rust_jvm_rsp_signal_guest_offset_const}], rsp",
    "mov [r15 + {__rust_jvm_r8_signal_guest_offset_const}], r8",
    "mov [r15 + {__rust_jvm_r9_signal_guest_offset_const}], r9",
    "mov [r15 + {__rust_jvm_r10_signal_guest_offset_const}], r10",
    "mov [r15 + {__rust_jvm_r11_signal_guest_offset_const}], r11",
    "mov [r15 + {__rust_jvm_r12_signal_guest_offset_const}], r12",
    "mov [r15 + {__rust_jvm_r13_signal_guest_offset_const}], r13",
    "mov [r15 + {__rust_jvm_r14_signal_guest_offset_const}], r14",
    "jmp qword ptr [r15 + {__rust_jvm_rip_native_offset_const}]",
    __rust_jvm_exit_number = const EXIT_NUMBER,
    // __rust_jvm_rip_signal_guest_offset_const = const RIP_SIGNAL_GUEST_OFFSET_CONST,
    __rust_jvm_rax_signal_guest_offset_const = const RAX_SIGNAL_GUEST_OFFSET_CONST,
    __rust_jvm_rbx_signal_guest_offset_const = const RBX_SIGNAL_GUEST_OFFSET_CONST,
    __rust_jvm_rcx_signal_guest_offset_const = const RCX_SIGNAL_GUEST_OFFSET_CONST,
    __rust_jvm_rdx_signal_guest_offset_const = const RDX_SIGNAL_GUEST_OFFSET_CONST,
    __rust_jvm_rsi_signal_guest_offset_const = const RSI_SIGNAL_GUEST_OFFSET_CONST,
    __rust_jvm_rdi_signal_guest_offset_const = const RDI_SIGNAL_GUEST_OFFSET_CONST,
    __rust_jvm_rbp_signal_guest_offset_const = const RBP_SIGNAL_GUEST_OFFSET_CONST,
    __rust_jvm_rsp_signal_guest_offset_const = const RSP_SIGNAL_GUEST_OFFSET_CONST,
    __rust_jvm_r8_signal_guest_offset_const = const R8_SIGNAL_GUEST_OFFSET_CONST,
    __rust_jvm_r9_signal_guest_offset_const = const R9_SIGNAL_GUEST_OFFSET_CONST,
    __rust_jvm_r10_signal_guest_offset_const = const R10_SIGNAL_GUEST_OFFSET_CONST,
    __rust_jvm_r11_signal_guest_offset_const = const R11_SIGNAL_GUEST_OFFSET_CONST,
    __rust_jvm_r12_signal_guest_offset_const = const R12_SIGNAL_GUEST_OFFSET_CONST,
    __rust_jvm_r13_signal_guest_offset_const = const R13_SIGNAL_GUEST_OFFSET_CONST,
    __rust_jvm_r14_signal_guest_offset_const = const R14_SIGNAL_GUEST_OFFSET_CONST,
    __rust_jvm_rip_native_offset_const = const RIP_NATIVE_OFFSET_CONST,
    options(noreturn)
    );
}

pub struct SignalAccessibleJavaStackData {
    stack_top: *const c_void,
    stack_bottom: *const c_void,
    pub interpreter_should_safepoint_check: AtomicBool,
}

impl SignalAccessibleJavaStackData {
    pub fn new(stack_top: *const c_void, stack_bottom: *const c_void) -> Self {
        Self {
            stack_top,
            stack_bottom,
            interpreter_should_safepoint_check: AtomicBool::new(false),
        }
    }
}

//TODO STILL NEED TO PUSH AN OPAQUE FRAME.
// do it outside of signal handler in the exit handler, b/c its easier that way.
// will need to save rbp/rsp/rip to enable that.
// #[derive(Debug)]
// pub enum ThingToPush<'signal_life> {
//     PrevRBP,
//     PrevSP,
//     Data(&'signal_life [u8]),
// }


pub struct RemoteQuerySafeEnterSafePointCheck{
}

pub enum RemoteQuerySafeEnterSafePointCheckResult{
    InGuest,
    NotInGuest
}

#[derive(Debug)]
#[repr(C)]
pub struct RemoteQueryUnsafe {
    signal_safe_data: NonNullConst<SignalAccessibleJavaStackData>,
    // to_push_opaque_id: OpaqueID,
    register_save_area: MaybeUninit<ucontext_t>,
    new_frame_rip: *const c_void,
    okay_to_free_this: AtomicBool,
    was_not_in_guest: AtomicBool,
    was_in_guest: AtomicBool,
}

impl RemoteQueryUnsafe {
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


unsafe fn handler_impl(info: *mut siginfo_t, ucontext: Option<*const ucontext_t>) {
    if let Err(_err) = std::panic::catch_unwind(|| {
        let remote_query_pointer = info.as_ref().unwrap().si_value().sival_ptr as *mut RemoteQueryUnsafe;
        let remote_query = remote_query_pointer.as_mut().unwrap();
        let signal_safe_data = remote_query.signal_safe_data();
        let stack_top = signal_safe_data.stack_top;
        let stack_bottom = signal_safe_data.stack_bottom;
        assert!(stack_top > stack_bottom);

        remote_query.register_save_area = MaybeUninit::new(ucontext.unwrap().read());
        let new_rip = remote_query.new_frame_rip as u64 as i64;
        let ucontext = (ucontext.unwrap() as *mut ucontext_t).as_mut().unwrap();
        let uc_mcontext = &mut ucontext.uc_mcontext;
        let gpregs = &mut uc_mcontext.gregs;
        let is_in_guest = in_guest(stack_top, stack_bottom, gpregs);
        if is_in_guest {
            let prev_rip = gpregs[REG_RIP as usize];
            gpregs[REG_RSP as usize] &= -16;
            gpregs[REG_RSP as usize] -= size_of::<u64>() as i64;
            //more for debugger than actually being able to ret. return to non-handler is done with setcontext
            let stack = gpregs[REG_RSP as usize] as *mut c_void as *mut u64;
            stack.write(prev_rip as u64);
            gpregs[REG_RIP as usize] = new_rip;
            gpregs[REG_R10 as usize] = remote_query_pointer as *mut c_void as i64;
            ucontext.uc_stack.ss_sp = stack as *mut c_void;//todo is this needed?
        } else {
            remote_query.was_not_in_guest.store(true,Ordering::SeqCst);
            remote_query.was_in_guest.store(false,Ordering::SeqCst);
            remote_query.okay_to_free_this.store(true,Ordering::SeqCst);
            return;
        }
    }) {
        eprintln!("panic in signal handler");//todo this probably allocates so maybe it should not
        libc::abort();
    }
}

fn in_guest(stack_top: *const c_void, stack_bottom: *const c_void, gpregs: &[greg_t; 23]) -> bool {
    let stack_pointer = gpregs[REG_RSP as usize];
    let frame_pointer = gpregs[REG_RBP as usize];
    let _instruction_pointer = gpregs[REG_RIP as usize];
    let stack_pointer_in_stack = (stack_pointer as *const c_void) < stack_top && (stack_pointer as *const c_void) > stack_bottom;
    let frame_pointer_in_stack = (frame_pointer as *const c_void) < stack_top && (frame_pointer as *const c_void) > stack_bottom;
    let in_guest = stack_pointer_in_stack && frame_pointer_in_stack;
    let _transitioning = stack_pointer_in_stack ^ frame_pointer_in_stack;
    in_guest
}


// cannot have a full c function on the stack b/c that can't generate a vm exit and the frame
// can't be pushed on top of it.
// need some asm to generate a standard exit.

//need to make extra room on the stack for locking stack frame, below this method.
#[no_mangle]
unsafe extern "C" fn no_longer_in_handler(query: *mut RemoteQueryUnsafe) {
    let saved = libc::__errno_location().read();
    println!("not in handler");
    let all_registers_restore = query.as_ref().unwrap().register_save_area.clone();
    let all_registers_restore_ptr = all_registers_restore.assume_init_ref().clone();
    query.as_ref().unwrap().okay_to_free_this.store(true, Ordering::SeqCst);
    libc::__errno_location().write(saved);
    setcontext(&all_registers_restore_ptr);
    unreachable!();
}


pub struct RemoteFramePush {
    per_thread_signal_lock: RwLock<HashMap<Pthread, Arc<Mutex<()>>>>,
}

impl RemoteFramePush {
    fn thread_signal_lock(&self, tid: Pthread) -> Arc<Mutex<()>> {
        self.per_thread_signal_lock.write().unwrap().entry(tid).or_default().clone()
    }

    //signal lock very much needed b/c otherwise multiple signals could be recieved at once with consequences for signal register saving.
    fn send_safepoint_check_signal(&self, tid: Pthread, signal_safe_data: Arc<SignalAccessibleJavaStackData>, _query: RemoteQuerySafeEnterSafePointCheck)-> RemoteQuerySafeEnterSafePointCheckResult {
        let mut query_unsafe = RemoteQueryUnsafe{
            signal_safe_data: NonNullConst::new(signal_safe_data.as_ref() as *const SignalAccessibleJavaStackData).unwrap(),
            // to_push_opaque_id: OpaqueID(0),
            register_save_area: MaybeUninit::zeroed(),
            new_frame_rip: exit_to_safepoint_check as *const c_void,
            okay_to_free_this: AtomicBool::new(false),
            was_not_in_guest: AtomicBool::new(false),
            was_in_guest: AtomicBool::new(false),
        };
        let signal_lock = self.thread_signal_lock(tid);
        let guard = signal_lock.lock().unwrap();
        let data = (&mut query_unsafe) as *mut RemoteQueryUnsafe;
        pthread_sigqueue(tid, Some(THREAD_PAUSE_SIGNAL), SigVal::Ptr(data as *mut c_void)).unwrap();
        loop {
            if query_unsafe.okay_to_free_this.load(Ordering::SeqCst){
                break;
            }
            std::hint::spin_loop();
        }
        let was_in_guest = query_unsafe.was_in_guest.load(Ordering::SeqCst);
        let was_not_in_guest = query_unsafe.was_not_in_guest.load(Ordering::SeqCst);
        if was_in_guest{
            assert!(!was_not_in_guest);
            drop(guard);
            return RemoteQuerySafeEnterSafePointCheckResult::InGuest;
        }else {
            assert!(was_not_in_guest);
            return RemoteQuerySafeEnterSafePointCheckResult::NotInGuest
        }
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
    use std::hint::spin_loop;
    use std::mem::MaybeUninit;
    use std::ptr::null_mut;
    use std::sync::{Arc, Barrier};
    use std::sync::atomic::{AtomicBool, Ordering};

    use nix::sys::pthread::pthread_self;
    use nonnull_const::NonNullConst;

    use crate::{no_longer_in_handler, RemoteFramePush, RemoteQueryUnsafe, SignalAccessibleJavaStackData};

    #[test]
    pub fn test() {
        let remote_frame_push = RemoteFramePush::sigaction_setup();
        let mut remote_query = RemoteQueryUnsafe {
            signal_safe_data: NonNullConst::new(Box::into_raw(box SignalAccessibleJavaStackData::new(null_mut(), null_mut()))).unwrap(),
            to_push: &[],
            okay_to_free_this: AtomicBool::new(false),
            new_frame_rip: no_longer_in_handler as *const c_void,
            register_save_area: MaybeUninit::uninit(),
            to_run_in_guest_frame: Box::new(todo!()),
        };
        let barrier = Arc::new(Barrier::new(2));
        let barrier_clone = barrier.clone();
        let (sender, receiver) = std::sync::mpsc::channel();
        let join_handle = std::thread::spawn(move || {
            let self_id = pthread_self();
            sender.send(self_id).unwrap();
            barrier_clone.wait();
        });
        let other_thread_id = receiver.recv().unwrap();
        remote_frame_push.send_signal(other_thread_id, (&mut remote_query) as *mut RemoteQueryUnsafe);
        while !remote_query.okay_to_free_this.load(Ordering::SeqCst) {
            spin_loop();
        }
        barrier.wait();
        join_handle.join().unwrap();
    }
}
