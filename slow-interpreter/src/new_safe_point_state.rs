use std::ops::Add;
use std::sync::{Condvar, Mutex};
use std::time::{Duration, Instant};

use jvmti_jni_bindings::{jint, JVMTI_THREAD_STATE_ALIVE, JVMTI_THREAD_STATE_BLOCKED_ON_MONITOR_ENTER, JVMTI_THREAD_STATE_IN_OBJECT_WAIT, JVMTI_THREAD_STATE_INTERRUPTED, JVMTI_THREAD_STATE_PARKED, JVMTI_THREAD_STATE_RUNNABLE, JVMTI_THREAD_STATE_SLEEPING, JVMTI_THREAD_STATE_SUSPENDED, JVMTI_THREAD_STATE_TERMINATED, JVMTI_THREAD_STATE_WAITING, JVMTI_THREAD_STATE_WAITING_INDEFINITELY, JVMTI_THREAD_STATE_WAITING_WITH_TIMEOUT};
use rust_jvm_common::JavaThreadId;

use crate::better_java_stack::frames::PushableFrame;
use crate::exceptions::WasException;
use crate::jvm_state::JVMState;
use crate::stdlib::java::lang::thread::JThread;
use crate::threading::java_thread::{ResumeError, SuspendError};
use crate::threading::safepoints::MonitorID;

pub enum ThreadOrBootstrap<'gc> {
    Bootstrap,
    Thread(JThread<'gc>),
}


#[derive(Debug)]
pub struct MonitorWait {
    wait_until: Option<Instant>,
    monitor: MonitorID,
    // prev_count: usize,
}

pub struct SafePointStateInner<'gc> {
    jvm: &'gc JVMState<'gc>,
    thread_to_update: ThreadOrBootstrap<'gc>,

    alive: bool,
    terminate: bool,

    gc_suspend: bool,
    interrupt: bool,
    suspended: bool,
    parks: i64,
    park_until: Option<Instant>,
    sleep_until: Option<Instant>,
    joining_thread: Option<JavaThreadId>,
    waiting_monitor_lock: Option<MonitorID>,
    waiting_monitor_notify: Option<MonitorWait>,
}

impl<'gc> SafePointStateInner<'gc> {
    pub fn new(jvm: &'gc JVMState<'gc>, thread_to_update: ThreadOrBootstrap<'gc>) -> Self {
        Self {
            jvm,
            thread_to_update,
            alive: false,
            terminate: false,
            gc_suspend: false,
            interrupt: false,
            suspended: false,
            parks: 0,
            park_until: None,
            sleep_until: None,
            joining_thread: None,
            waiting_monitor_lock: None,
            waiting_monitor_notify: None,
        }
    }

    fn get_thread_status_number(&self) -> jint {
        let mut res = 0;
        let guard = self;
        if guard.alive {
            res |= JVMTI_THREAD_STATE_ALIVE;
            //todo is in native code
            if guard.interrupt {
                res |= JVMTI_THREAD_STATE_INTERRUPTED;
            }
            if guard.suspended {
                res |= JVMTI_THREAD_STATE_SUSPENDED;
            }
            if guard.waiting_monitor_lock.is_some() {
                res |= JVMTI_THREAD_STATE_BLOCKED_ON_MONITOR_ENTER;
            } else if let Some(MonitorWait { wait_until, .. }) = &guard.waiting_monitor_notify {
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
            if guard.terminate {
                res |= JVMTI_THREAD_STATE_TERMINATED;
            }
        }
        res as jint
    }

    fn update_thread_object(&mut self) {
        let thread_status_number = self.get_thread_status_number();
        let jvm = self.jvm;
        if let ThreadOrBootstrap::Thread(jthread) = &self.thread_to_update {
            jthread.set_thread_status(jvm, thread_status_number)
        }
    }

    pub fn set_alive(&mut self) {
        self.alive = true;
        self.update_thread_object();
    }

    pub fn set_terminated(&mut self) {
        self.alive = false;
        self.terminate = true;
        self.update_thread_object();
    }
}

pub struct NewSafePointState<'gc> {
    inner: Mutex<SafePointStateInner<'gc>>,
    condvar: Condvar,
}

impl<'gc> NewSafePointState<'gc> {
    pub fn new(jvm: &'gc JVMState<'gc>, thread_to_update: ThreadOrBootstrap<'gc>) -> Self {
        Self {
            inner: Mutex::new(SafePointStateInner::new(jvm, thread_to_update)),
            condvar: Condvar::new(),
        }
    }

    pub fn wait_thread_exit(&self) {
        let mut guard = self.inner.lock().unwrap();
        loop {
            if guard.terminate && !guard.alive{
                break;
            }
            guard = self.condvar.wait(guard).unwrap();
        }
    }

    pub fn check(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<(), WasException<'gc>> {
        let mut guard = self.inner.lock().unwrap();
        loop {
            if guard.terminate {
                todo!("exit the whole thread")
            }

            if guard.gc_suspend {
                todo!("blocking on gc impl")
            }

            if guard.interrupt {
                todo!("throw interrupted exception")
            }

            if guard.suspended {
                guard = self.condvar.wait(guard).unwrap();
                continue;
            }

            if guard.parks > 0 {
                match guard.park_until {
                    None => {
                        guard = self.condvar.wait(guard).unwrap();
                    }
                    Some(park_until) => {
                        let duration = match park_until.checked_duration_since(Instant::now()) {
                            Some(x) => x,
                            None => {
                                guard.park_until = None;
                                guard.parks += 1;
                                //todo need to update thread obj
                                continue;
                            }
                        };
                        let (guard_tmp, _) = self.condvar.wait_timeout(guard, duration).unwrap();
                        guard = guard_tmp;
                    }
                }
                continue;
            }

            if let Some(sleep_until) = guard.sleep_until {
                let duration = match sleep_until.checked_duration_since(Instant::now()) {
                    Some(x) => x,
                    None => {
                        guard.sleep_until = None;
                        //todo need to update thread object
                        continue;
                    }
                };
                let (guard_tmp, _) = self.condvar.wait_timeout(guard, duration).unwrap();
                guard = guard_tmp;
                continue;
            }

            if let Some(joining_thread) = guard.joining_thread.as_ref() {
                todo!()
            }

            if let Some(waiting_monitor_lock) = guard.waiting_monitor_lock.as_ref() {
                let guard_tmp = self.condvar.wait(guard).unwrap();
                guard = guard_tmp;
                continue;
            }

            if let Some(MonitorWait{ wait_until, .. }) = guard.waiting_monitor_notify.as_ref() {
                let duration = match wait_until {
                    None => None,
                    Some(wait_until) => {
                        match wait_until.checked_duration_since(Instant::now()) {
                            Some(duration) => Some(duration),
                            None => {
                                return Ok(());//time has expired need to return from safepoint, given all other statuses are not taken
                            }
                        }
                    }
                };

                let guard_tmp = match duration {
                    None => {
                        self.condvar.wait(guard).unwrap()
                    }
                    Some(duration) => {
                        self.condvar.wait_timeout(guard, duration).unwrap().0
                    }
                };
                guard = guard_tmp;
                continue;
            }

            return Ok(());
        }
    }

    pub fn set_monitor_unlocked(&self) {
        let mut guard = self.inner.lock().unwrap();
        guard.waiting_monitor_lock = None;
        self.condvar.notify_all();
    }

    pub fn set_monitor_lock(&self, to: MonitorID) {
        let mut guard = self.inner.lock().unwrap();
        assert!(guard.waiting_monitor_lock.is_none());
        guard.waiting_monitor_lock = Some(to);
        self.condvar.notify_all();
    }

    pub fn set_waiting_notify(&self, monitor: MonitorID, wait_until: Option<Instant>) {
        let mut guard = self.inner.lock().unwrap();
        assert!(guard.waiting_monitor_notify.is_none());
        guard.waiting_monitor_notify = Some(MonitorWait { wait_until, monitor/*, prev_count*/ });
        self.condvar.notify_all();
    }

    pub fn set_notified(&self) {
        let mut guard = self.inner.lock().unwrap();
        assert!(guard.waiting_monitor_notify.is_some());
        guard.waiting_monitor_notify = None;
        self.condvar.notify_all();
    }

    pub fn set_suspended(&self) -> Result<(), SuspendError> {
        let mut guard = self.inner.lock().unwrap();
        if guard.suspended {
            return Err(SuspendError::AlreadySuspended);
        }
        guard.suspended = true;
        self.condvar.notify_all();
        Ok(())
    }

    pub fn set_gc_suspended(&self) -> Result<(), SuspendError> {
        let mut guard = self.inner.lock().unwrap();
        if guard.gc_suspend {
            return Err(SuspendError::AlreadySuspended);
        }
        guard.gc_suspend = true;
        self.condvar.notify_all();
        Ok(())
    }

    pub fn set_gc_unsuspended(&self) -> Result<(), ResumeError> {
        let mut guard = self.inner.lock().unwrap();
        if !guard.gc_suspend {
            return Err(ResumeError::NotSuspended);
        }
        guard.gc_suspend = false;
        self.condvar.notify_all();
        Ok(())
    }

    pub fn set_unsuspended(&self) -> Result<(), ResumeError> {
        let mut guard = self.inner.lock().unwrap();
        if !guard.suspended {
            return Err(ResumeError::NotSuspended);
        }
        guard.suspended = false;
        self.condvar.notify_all();
        Ok(())
    }

    pub fn set_sleeping(&self, to_sleep: Duration) {
        let mut guard = self.inner.lock().unwrap();
        guard.sleep_until = Some(Instant::now().add(to_sleep));
        self.condvar.notify_all();
    }

    pub fn set_park(&self, time: Option<Duration>) {
        let park_until = time.map(|time| Instant::now().add(time));
        let mut guard = self.inner.lock().unwrap();
        assert!(guard.waiting_monitor_notify.is_none());
        assert!(guard.waiting_monitor_lock.is_none());
        guard.park_until = park_until;
        guard.parks += 1;
        self.condvar.notify_all()
    }

    pub fn set_unpark(&self) {
        let mut guard = self.inner.lock().unwrap();
        assert!(guard.parks <= 1);
        guard.park_until = None;
        guard.parks -= 1;
        self.condvar.notify_all()
    }


    pub fn set_alive(&self) {
        let mut guard = self.inner.lock().unwrap();
        guard.set_alive();
    }

    pub fn set_terminated(&self) {
        let mut guard = self.inner.lock().unwrap();
        guard.set_terminated();
    }

    pub fn is_interrupted(&self) -> bool{
        self.inner.lock().unwrap().interrupt
    }

    pub fn is_alive(&self) -> bool{
        self.inner.lock().unwrap().alive
    }
}
