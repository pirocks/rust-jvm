use std::collections::HashMap;
use std::ffi::c_void;
use std::hint;
use std::ptr::null_mut;
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};

use nix::sys::pthread::{Pthread, pthread_sigqueue, SigVal};
use nix::sys::signal::{SaFlags, sigaction, SigAction, SigHandler, SigSet};

use thread_signal_handler::{handler, SignalAccessibleJavaStackData, THREAD_PAUSE_SIGNAL, THREAD_RESTART_SIGNAL};
use thread_signal_handler::remote_queries::{RemoteQuery, RemoteQueryAnswer, RemoteQuerySafe, RemoteQueryUnsafe};

pub struct ThreadSignalBasedInterrupter {
    per_thread_signal_lock: RwLock<HashMap<Pthread, Arc<Mutex<()>>>>,
}

impl ThreadSignalBasedInterrupter {
    fn thread_signal_lock(&self, tid: Pthread) -> Arc<Mutex<()>> {
        self.per_thread_signal_lock.write().unwrap().entry(tid).or_default().clone()
    }

    pub fn perform_remote_query(&self, tid: Pthread, remote_query_public: RemoteQuery, signal_safe_data: &SignalAccessibleJavaStackData, with_answer: impl FnOnce(RemoteQueryAnswer)) {
        let thread_signal_lock = self.thread_signal_lock(tid);
        let signal_guard = thread_signal_lock.lock().unwrap();
        //todo just have  a threads signalling lock
        let mut answer = None;
        let answer_written = AtomicBool::new(false);
        match remote_query_public {
            RemoteQuery::GetGuestFrameStackInstructionPointer => {
                let remote_query_safe = RemoteQuerySafe::GetGuestFrameStackInstructionPointer { answer: &mut answer, answer_written: &answer_written };
                let mut remote_query = remote_query_safe.to_remote_query_unsafe(signal_safe_data);
                self.send_signal(tid, &mut remote_query as *mut RemoteQueryUnsafe);
                while answer_written.load(Ordering::SeqCst) != true {
                    hint::spin_loop();
                }
                let answer = answer.unwrap();
                with_answer(RemoteQueryAnswer::GetGuestFrameStackInstructionPointer(answer));
                self.send_restart_signal(tid);
            }
            RemoteQuery::GC => todo!()
        };
        drop(signal_guard);
    }

    fn send_signal(&self, tid: Pthread, data: *mut RemoteQueryUnsafe) {
        pthread_sigqueue(tid, Some(THREAD_PAUSE_SIGNAL), SigVal::Ptr(data as *mut c_void)).unwrap();
    }

    fn send_restart_signal(&self, tid: Pthread) {
        pthread_sigqueue(tid, Some(THREAD_RESTART_SIGNAL), SigVal::Ptr(null_mut())).unwrap();
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