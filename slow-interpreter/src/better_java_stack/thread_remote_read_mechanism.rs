use std::collections::HashMap;
use std::ffi::c_void;
use std::hint;
use std::ptr::null_mut;
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::Ordering;

use nix::sys::pthread::{Pthread, pthread_sigqueue, SigVal};
use nix::sys::signal::{SaFlags, sigaction, SigAction, SigHandler, SigSet};

use thread_signal_handler::{handler, RemoteQuery, RemoteQueryAnswer, RemoteQueryAnswerInternal, RemoteQueryInternal, SignalAccessibleJavaStackData, THREAD_PAUSE_SIGNAL};

pub struct ThreadSignalBasedInterrupter {
    per_thread_signal_lock: RwLock<HashMap<Pthread, Arc<Mutex<()>>>>,
}

impl ThreadSignalBasedInterrupter {
    fn thread_signal_lock(&self, tid: Pthread) -> Arc<Mutex<()>> {
        self.per_thread_signal_lock.write().unwrap().entry(tid).or_default().clone()
    }

    pub fn perform_remote_query(&self, tid: Pthread, remote_query: RemoteQuery, signal_data: &SignalAccessibleJavaStackData, with_answer: impl FnOnce(RemoteQueryAnswer)) {
        let thread_signal_lock = self.thread_signal_lock(tid);
        let signal_guard = thread_signal_lock.lock().unwrap();
        //todo just have  a threads signalling lock
        let mut remote_query = remote_query.to_remote_query_internal();
        let remote_query_mut = &mut remote_query;
        let raw_remote_qury = remote_query_mut as *mut RemoteQueryInternal;
        let mut answer = RemoteQueryAnswerInternal::Empty;
        let remote_query_mut = &mut answer;
        let raw_remote_answer = remote_query_mut as *mut RemoteQueryAnswerInternal;
        while let Err(old) = signal_data.remote_request.compare_exchange(null_mut(), raw_remote_qury, Ordering::SeqCst, Ordering::SeqCst) {
            hint::spin_loop();
        }
        assert!(!signal_data.answer_written.load(Ordering::SeqCst));
        signal_data.remote_request_answer.compare_exchange(null_mut(), raw_remote_answer, Ordering::SeqCst, Ordering::SeqCst).unwrap();
        let ptr = signal_data as *const SignalAccessibleJavaStackData as *mut c_void;
        pthread_sigqueue(tid, Some(THREAD_PAUSE_SIGNAL), SigVal::Ptr(ptr)).unwrap();
        while signal_data.answer_written.load(Ordering::SeqCst) != true {
            hint::spin_loop();
        }
        signal_data.remote_request_answer.compare_exchange(raw_remote_answer, null_mut(), Ordering::SeqCst, Ordering::SeqCst).unwrap();
        signal_data.remote_request.compare_exchange(raw_remote_qury, null_mut(), Ordering::SeqCst, Ordering::SeqCst).unwrap();
        signal_data.answer_written.store(false, Ordering::SeqCst);
        match answer {
            RemoteQueryAnswerInternal::GetGuestFrameStackInstructionPointer {
                answer
            } => {
                with_answer(RemoteQueryAnswer::GetGuestFrameStackInstructionPointer(answer));
            }
            RemoteQueryAnswerInternal::Panic(panic_data) => {
                std::panic::resume_unwind(panic_data)
            }
            RemoteQueryAnswerInternal::Empty => {
                todo!("handle unhandled signals")
            }
        }
        if remote_query.to_remote_query().wait_for_next_signal() {
            pthread_sigqueue(tid, Some(THREAD_PAUSE_SIGNAL), SigVal::Ptr(null_mut())).unwrap();
        }
        drop(signal_guard);
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