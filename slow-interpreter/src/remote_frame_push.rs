use std::collections::HashMap;
use std::ffi::{c_int, c_void};
use std::mem::{MaybeUninit, size_of};
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(debug_assertions)]
use assert_no_alloc::{AllocDisabler};
use assert_no_alloc::assert_no_alloc;
use nix::libc;
use nix::libc::{greg_t, REG_RBP, REG_RDI, REG_RIP, REG_RSP, setcontext, siginfo_t, ucontext_t};
use nix::sys::pthread::{Pthread, pthread_sigqueue, SigVal};
use nix::sys::signal::{SaFlags, sigaction, SigAction, SigHandler, Signal, SigSet};
use nonnull_const::NonNullConst;


#[cfg(debug_assertions)] // required when disable_release is set (default)
#[global_allocator]
static A: AllocDisabler = AllocDisabler;

// need try push java frame


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

#[derive(Debug)]
pub enum ThingToPush<'signal_life> {
    PrevRBP,
    PrevSP,
    Data(&'signal_life [u8]),
}

pub trait RunInRemoteFramePush: for<'gc, 'k, 'l> FnOnce(&'l mut OpaqueFrame<'gc, 'k>) + Send {}

#[derive(Debug)]
pub struct RemoteQueryUnsafe<'signal_life> {
    signal_safe_data: NonNullConst<SignalAccessibleJavaStackData>,
    to_push: &'signal_life [ThingToPush<'signal_life>],
    register_save_area: MaybeUninit<ucontext_t>,
    new_frame_rip: *const c_void,
    okay_to_free_this: AtomicBool,
    to_run_in_guest_frame: Box<dyn RunInRemoteFramePush>
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
        if is_in_guest{
            let prev_rip = gpregs[REG_RIP as usize];
            gpregs[REG_RSP as usize] &= -16;
            gpregs[REG_RSP as usize] -= size_of::<u64>() as i64;
            //more for debugger than actually being able to ret. return to non-handler is done with setcontext
            let stack = gpregs[REG_RSP as usize] as *mut c_void as *mut u64;
            stack.write(prev_rip as u64);
            gpregs[REG_RIP as usize] = new_rip;
            gpregs[REG_RDI as usize] = remote_query_pointer as *mut c_void as i64;
            ucontext.uc_stack.ss_sp = stack as *mut c_void;
        }else {
            todo!()
        }
    }) {
        eprintln!("panic in signal handler");
        libc::abort();
    }
}

fn in_guest(stack_top: *const c_void, stack_bottom: *const c_void, gpregs: &[greg_t; 23]) -> bool {
    let stack_pointer = gpregs[REG_RSP as usize];
    let frame_pointer = gpregs[REG_RBP as usize];
    let instruction_pointer = gpregs[REG_RIP as usize];
    let stack_pointer_in_stack = (stack_pointer as *const c_void) < stack_top && (stack_pointer as *const c_void) > stack_bottom;
    let frame_pointer_in_stack = (frame_pointer as *const c_void) < stack_top && (frame_pointer as *const c_void) > stack_bottom;
    let in_guest = stack_pointer_in_stack && frame_pointer_in_stack;
    let transitioning = stack_pointer_in_stack ^ frame_pointer_in_stack;
    in_guest
}

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

    fn send_signal(&self, tid: Pthread, data: *mut RemoteQueryUnsafe<>) {
        let signal_lock = self.thread_signal_lock(tid);
        let guard = signal_lock.lock().unwrap();
        pthread_sigqueue(tid, Some(THREAD_PAUSE_SIGNAL), SigVal::Ptr(data as *mut c_void)).unwrap();
        drop(guard);
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
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Barrier};

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
