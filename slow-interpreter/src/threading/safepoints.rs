use std::ops::Add;
use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::time::{Duration, Instant};

use jvmti_jni_bindings::{jint, JVMTI_THREAD_STATE_ALIVE, JVMTI_THREAD_STATE_BLOCKED_ON_MONITOR_ENTER, JVMTI_THREAD_STATE_IN_OBJECT_WAIT, JVMTI_THREAD_STATE_INTERRUPTED, JVMTI_THREAD_STATE_PARKED, JVMTI_THREAD_STATE_RUNNABLE, JVMTI_THREAD_STATE_SLEEPING, JVMTI_THREAD_STATE_SUSPENDED, JVMTI_THREAD_STATE_TERMINATED, JVMTI_THREAD_STATE_WAITING, JVMTI_THREAD_STATE_WAITING_INDEFINITELY, JVMTI_THREAD_STATE_WAITING_WITH_TIMEOUT};

use crate::interpreter::WasException;
use crate::interpreter_state::InterpreterStateGuard;
use crate::java_values::Object;
use crate::jvm_state::JVMState;
use crate::threading::{JavaThreadId, ResumeError, SuspendError, ThreadStatus};

pub type MonitorID = usize;

#[derive(Debug)]
pub struct MonitorWait {
    wait_until: Option<Instant>,
    monitor: MonitorID,
    prev_count: usize,
}

#[derive(Debug)]
struct SafePointStopReasonState {
    waiting_monitor_lock: Option<MonitorID>,
    waiting_monitor_notify: Option<MonitorWait>,
    suspended: bool,
    parks: isize,
    park_until: Option<Instant>,
    throw_exception: Option<Arc<Object>>,
    sleep_until: Option<Instant>,
}

impl Default for SafePointStopReasonState {
    fn default() -> Self {
        Self {
            waiting_monitor_lock: None,
            waiting_monitor_notify: None,
            suspended: false,
            parks: 0,
            park_until: None,
            throw_exception: None,
            sleep_until: None,
        }
    }
}

#[derive(Debug)]
pub struct SafePoint {
    state: Mutex<SafePointStopReasonState>,
    waiton: Condvar,
}

impl SafePoint {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(Default::default()),
            waiton: Default::default(),
        }
    }

    pub fn set_monitor_unlocked(&self) {
        let mut guard = self.state.lock().unwrap();
        assert!(guard.waiting_monitor_lock.is_some());
        guard.waiting_monitor_lock = None;
        self.waiton.notify_one();
    }

    pub fn set_monitor_lock(&self, to: MonitorID) {
        let mut guard = self.state.lock().unwrap();
        assert!(guard.waiting_monitor_lock.is_none());
        guard.waiting_monitor_lock = Some(to);
    }

    pub fn set_waiting_notify(&self, monitor: MonitorID, wait_until: Option<Instant>, prev_count: usize) {
        let mut guard = self.state.lock().unwrap();
        assert!(guard.waiting_monitor_notify.is_none());
        guard.waiting_monitor_notify = Some(MonitorWait { wait_until, monitor, prev_count });
    }

    pub fn set_notified(&self) {
        let mut guard = self.state.lock().unwrap();
        assert!(guard.waiting_monitor_notify.is_some());
        guard.waiting_monitor_notify = None;
    }

    pub fn set_suspended(&self) -> Result<(), SuspendError> {
        let mut guard = self.state.lock().unwrap();
        if guard.suspended {
            return Result::Err(SuspendError::AlreadySuspended);
        }
        guard.suspended = true;

        Ok(())
    }

    pub fn set_unsuspended(&self) -> Result<(), ResumeError> {
        let mut guard = self.state.lock().unwrap();
        if !guard.suspended {
            return Err(ResumeError::NotSuspended);
        }
        guard.suspended = false;
        self.waiton.notify_one();// technically I don't need this to be a condvar since I only use notify_one I could have this be a lock
        Ok(())
    }

    pub fn set_sleeping(&self, to_sleep: Duration) {
        let mut guard = self.state.lock().unwrap();
        guard.sleep_until = Some(Instant::now().add(to_sleep))
    }

    pub fn set_park(&self, time: Option<Duration>) {
        let park_until = time.map(|time| Instant::now().add(time));
        let mut guard = self.state.lock().unwrap();
        guard.park_until = park_until;
        guard.parks += 1;
        self.waiton.notify_one()
    }

    pub fn set_unpark(&self) {
        let mut guard = self.state.lock().unwrap();
        assert!(guard.parks <= 1);
        guard.park_until = None;
        guard.parks -= 1;
        self.waiton.notify_one()
    }

    pub(crate) fn get_thread_status_number(&self, thread_status: &ThreadStatus) -> jint {
        let mut res = 0;
        let guard = self.state.lock().unwrap();
        if thread_status.alive {
            res |= JVMTI_THREAD_STATE_ALIVE;
            //todo is in native code
            if thread_status.interrupted {
                res |= JVMTI_THREAD_STATE_INTERRUPTED;
            }
            if guard.suspended {
                res |= JVMTI_THREAD_STATE_SUSPENDED;
            }
            if guard.waiting_monitor_lock.is_some() {
                res |= JVMTI_THREAD_STATE_BLOCKED_ON_MONITOR_ENTER;
            } else if let Some(MonitorWait { wait_until, monitor, prev_count }) = &guard.waiting_monitor_notify {
                res |= JVMTI_THREAD_STATE_WAITING;
                res |= JVMTI_THREAD_STATE_IN_OBJECT_WAIT;
                if wait_until.is_none() {
                    res |= JVMTI_THREAD_STATE_WAITING_INDEFINITELY;
                } else {
                    res |= JVMTI_THREAD_STATE_WAITING_WITH_TIMEOUT;
                }
            } else if guard.sleep_until.is_some() {
                res |= JVMTI_THREAD_STATE_WAITING;
                res |= JVMTI_THREAD_STATE_SLEEPING;
                res |= JVMTI_THREAD_STATE_WAITING_WITH_TIMEOUT;
            } else if guard.parks > 0 {
                res |= JVMTI_THREAD_STATE_WAITING;
                res |= JVMTI_THREAD_STATE_PARKED;
                if guard.park_until.is_none() {
                    res |= JVMTI_THREAD_STATE_WAITING_INDEFINITELY;
                } else {
                    res |= JVMTI_THREAD_STATE_WAITING_WITH_TIMEOUT;
                }
            } else {
                res |= JVMTI_THREAD_STATE_RUNNABLE;
            }
        } else {
            if thread_status.terminated {
                res |= JVMTI_THREAD_STATE_TERMINATED;
            }
        }
        res as jint
    }
}

impl SafePoint {
    pub fn check(&self, jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> Result<(), WasException> {
        let guard = self.state.lock().unwrap();

        if let Some(exception) = &guard.throw_exception {
            int_state.set_throw(exception.clone().into());
            return Err(WasException);
        }
        if guard.suspended {
            let _ = self.waiton.wait(guard).unwrap();
            return self.check(jvm, int_state);
        }
        if guard.parks > 0 {
            let _ = self.waiton.wait(guard).unwrap();
            return self.check(jvm, int_state);
        }
        if let Some(_) = &guard.waiting_monitor_lock {
            let _ = self.waiton.wait(guard).unwrap();
            return self.check(jvm, int_state);
        }
        if let Some(MonitorWait { wait_until, monitor, prev_count }) = &guard.waiting_monitor_notify {
            let wait_until = *wait_until;
            let monitor = *monitor;
            let prev_count = *prev_count;
            let time_to_wait = wait_until.map(|wait_until| match wait_until.checked_duration_since(Instant::now()) {
                None => Duration::new(0, 0),
                Some(time_to_wait) => time_to_wait
            });
            let (guard, should_reacquire) = match time_to_wait {
                None => {
                    let guard = self.waiton.wait(guard).unwrap();
                    let should_reacquire = guard.waiting_monitor_notify.is_none();
                    (guard, should_reacquire)
                }
                Some(time_to_wait) => {
                    let (guard, timeout) = self.waiton.wait_timeout(guard, time_to_wait).unwrap();
                    let should_reacquire = if timeout.timed_out() {
                        wait_until.unwrap().checked_duration_since(Instant::now()).is_none()
                    } else {
                        guard.waiting_monitor_notify.is_none()
                    };
                    (guard, should_reacquire)
                }
            };


            if should_reacquire {
                let monitors_gaurd = jvm.monitors2.read().unwrap();
                let monitor = &monitors_gaurd[monitor];
                drop(guard);
                monitor.notify_reacquire(jvm, int_state, prev_count)?;
            } else {
                drop(guard);//shouldn't need these but they are here for now b/c I'm paranoid
            }
            return self.check(jvm, int_state);
        }
        Ok(())
    }
}

pub struct Monitor2(RwLock<Monitor2Priv>);

struct Monitor2Priv {
    pub id: MonitorID,
    pub owner: Option<JavaThreadId>,
    pub count: usize,
    pub waiting_notify: Vec<JavaThreadId>,
    pub waiting_lock: Vec<JavaThreadId>,
}

impl Monitor2 {
    pub fn new() -> Self {
        Self(RwLock::new(Monitor2Priv {
            id: 0,
            owner: None,
            count: 0,
            waiting_notify: vec![],
            waiting_lock: vec![],
        }))
    }

    pub fn lock(&self, jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> Result<(), WasException> {
        let mut guard = self.0.write().unwrap();
        let current_thread = jvm.thread_state.get_current_thread();
        if let Some(owner) = guard.owner.as_ref() {
            if *owner == current_thread.java_tid {
                guard.count += 1;
            } else {
                guard.waiting_lock.push(current_thread.java_tid);
                current_thread.safepoint_state.set_monitor_lock(guard.id);
                drop(guard);
                current_thread.safepoint_state.check(jvm, int_state)?;
            }
        } else {
            guard.owner = Some(current_thread.java_tid);
            guard.count = 1;
        }
        Ok(())
    }

    pub fn unlock(&self, jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> Result<(), WasException> {
        let mut guard = self.0.write().unwrap();
        let current_thread = jvm.thread_state.get_current_thread();
        if guard.owner == current_thread.java_tid.into() {
            guard.count -= 1;
            if guard.count == 0 {
                guard.owner = None;
                if let Some(tid) = guard.waiting_lock.pop() {
                    let to_unlock_thread = jvm.thread_state.get_thread_by_tid(tid);
                    to_unlock_thread.safepoint_state.set_monitor_unlocked();
                }
            }
        } else {
            todo!("illegal monitor state")
        }
        Ok(())
    }


    pub fn notify(&self, jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> Result<(), WasException> {
        let mut guard = self.0.write().unwrap();
        if let Some(to_notify) = guard.waiting_notify.pop() {
            let to_notify_thread = jvm.thread_state.get_thread_by_tid(to_notify);
            to_notify_thread.safepoint_state.set_notified();
        }
        Ok(())
    }


    pub fn notify_all(&self, jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> Result<(), WasException> {
        let mut guard = self.0.write().unwrap();
        for to_notify in guard.waiting_notify.drain(..) {
            let to_notify_thread = jvm.thread_state.get_thread_by_tid(to_notify);
            to_notify_thread.safepoint_state.set_notified();
        }
        Ok(())
    }

    pub fn wait(&self, jvm: &JVMState, int_state: &mut InterpreterStateGuard, wait_duration: Option<Duration>) -> Result<(), WasException> {
        let mut guard = self.0.write().unwrap();
        let now = Instant::now();
        let wait_until = wait_duration.map(|wait_duration| match now.checked_add(wait_duration) {
            None => panic!("If you are reading this there something wrong with the amount of time you are calling a wait for"),
            Some(wait_until) => wait_until,
        });
        let current_thread = jvm.thread_state.get_current_thread();
        if guard.owner != current_thread.java_tid.into() {
            let prev_count = guard.count;
            guard.owner = None;
            guard.waiting_notify.push(current_thread.java_tid);
            current_thread.safepoint_state.set_waiting_notify(guard.id, wait_until, prev_count);
            drop(guard);
            current_thread.safepoint_state.check(jvm, int_state)?;
        } else {
            todo!("throw illegal monitor state")
        }
        Ok(())
    }

    pub fn notify_reacquire(&self, jvm: &JVMState, int_state: &mut InterpreterStateGuard, prev_count: usize) -> Result<(), WasException> {
        let mut guard = self.0.write().unwrap();
        self.lock(jvm, int_state)?;
        let current_thread = jvm.thread_state.get_current_thread();
        guard.count = prev_count;
        guard.owner = Some(current_thread.java_tid);
        Ok(())
    }
}