use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};

use crate::interpreter::WasException;
use crate::interpreter_state::InterpreterStateGuard;
use crate::java_values::Object;
use crate::jvm_state::JVMState;
use crate::jvmti::threads::suspend_resume::suspend_thread;
use crate::threading::monitors::Monitor;

pub type MonitorID = usize;

struct SafePointStopReasonState {
    waiting_monitor_lock: Option<MonitorID>,
    waiting_monitor_notify: Option<MonitorID>,
    suspended: bool,
    parks: usize,
    throw_exception: Option<Arc<Object>>,
}

struct SafePoint {
    state: Mutex<SafePointStopReasonState>,
    waiton: Condvar,
}

impl SafePoint {
    fn check(&self, jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> Result<(), WasException> {
        let mut guard = self.state.lock().unwrap();

        if let Some(exception) = &guard.throw_exception {
            int_state.set_throw(exception.clone().into());
            return Err(WasException);
        }
        if guard.suspended {
            self.waiton.wait(guard);
            return self.check(jvm, int_state);
        }
        if guard.parks != 0 {
            self.waiton.wait(guard);
        }
        if let Some(_monitor) = &guard.waiting_monitor_lock {
            todo!()
        }
        if let Some(_monitor) = &guard.waiting_monitor_notify {
            todo!()
        }
        Ok(())
    }
}