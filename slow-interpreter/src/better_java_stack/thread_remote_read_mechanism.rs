use std::ffi::c_void;
use std::hint;
use std::ptr::null_mut;
use std::sync::atomic::Ordering;
use nix::sys::pthread::{Pthread, pthread_sigqueue, SigVal};
use nix::sys::signal::{SaFlags, sigaction, SigAction, SigHandler, SigSet};
use thread_signal_handler::{handler, RemoteQuery, RemoteQueryAnswer, RemoteQueryAnswerInternal, SignalAccessibleJavaStackData, THREAD_PAUSE_SIGNAL};

pub fn perform_remote_query(tid: Pthread, mut remote_query: RemoteQuery, signal_data: &SignalAccessibleJavaStackData, with_answer: impl FnOnce(RemoteQueryAnswer)) {
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
    assert!(!signal_data.answer_written.load(Ordering::SeqCst));
    pthread_sigqueue(tid, Some(THREAD_PAUSE_SIGNAL), SigVal::Ptr(signal_data as *const SignalAccessibleJavaStackData as *mut c_void)).unwrap();
    while signal_data.answer_written.load(Ordering::SeqCst) != true {
        hint::spin_loop();
    }
    signal_data.answer_written.store(false, Ordering::SeqCst);
    signal_data.remote_request_answer.compare_exchange(raw_remote_query, null_mut(), Ordering::SeqCst, Ordering::SeqCst).unwrap();
    signal_data.remote_request.compare_exchange(alloc_remote_query_raw, null_mut(), Ordering::SeqCst, Ordering::SeqCst).unwrap();
    match answer {
        RemoteQueryAnswerInternal::GetGuestFrameStackInstructionPointer {
            answer
        } => {
            with_answer(RemoteQueryAnswer::GetGuestFrameStackInstructionPointer(answer));
            // restart.wait();
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
    use thread_signal_handler::{RemoteQuery, SignalAccessibleJavaStackData};

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