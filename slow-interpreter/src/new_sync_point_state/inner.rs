use std::time::{Duration, Instant};

use jvmti_jni_bindings::{jint, JVMTI_THREAD_STATE_ALIVE, JVMTI_THREAD_STATE_BLOCKED_ON_MONITOR_ENTER, JVMTI_THREAD_STATE_IN_OBJECT_WAIT, JVMTI_THREAD_STATE_INTERRUPTED, JVMTI_THREAD_STATE_PARKED, JVMTI_THREAD_STATE_RUNNABLE, JVMTI_THREAD_STATE_SLEEPING, JVMTI_THREAD_STATE_SUSPENDED, JVMTI_THREAD_STATE_TERMINATED, JVMTI_THREAD_STATE_WAITING, JVMTI_THREAD_STATE_WAITING_INDEFINITELY, JVMTI_THREAD_STATE_WAITING_WITH_TIMEOUT};
use rust_jvm_common::JavaThreadId;

use crate::jvm_state::JVMState;
use crate::new_sync_point_state::ThreadOrBootstrap;
use crate::threading::java_thread::{ResumeError, SuspendError};
use crate::threading::safepoints::MonitorID;

#[derive(Copy, Clone, Debug)]
pub struct MonitorWait {
    pub(crate) wait_until: Option<Instant>,
    pub(crate) monitor: MonitorID,
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


    //todo should have a notion of terminating
    pub fn set_terminated(&mut self) {
        self.alive = false;
        self.terminate = true;
        self.update_thread_object();
    }

    pub fn park_timed_out_set(&mut self) {
        self.parks += 1;
        self.park_until = None;
        self.update_thread_object();
    }

    pub fn sleep_timed_out_set(&mut self) {
        self.sleep_until = None;
        self.update_thread_object();
    }

    pub fn monitor_wait_timed_out_set(&mut self) {
        self.waiting_monitor_notify = None;
        self.update_thread_object();
    }

    pub fn set_monitor_unlocked(&mut self) {
        assert!(self.waiting_monitor_lock.is_some());
        self.waiting_monitor_lock = None;
        self.update_thread_object();
    }

    pub fn set_monitor_lock(&mut self, to: MonitorID) {
        assert!(self.waiting_monitor_lock.is_none());
        self.waiting_monitor_lock = Some(to);
        self.update_thread_object();
    }

    pub fn set_waiting_notify(&mut self, monitor: MonitorID, wait_until: Option<Instant>) {
        assert!(self.waiting_monitor_notify.is_none());
        self.waiting_monitor_notify = Some(MonitorWait { wait_until, monitor });
        self.update_thread_object();
    }

    pub fn set_notified(&mut self) {
        assert!(self.waiting_monitor_notify.is_some());
        self.waiting_monitor_notify = None;
        self.update_thread_object();
    }

    pub fn set_suspended(&mut self) -> Result<(), SuspendError> {
        if self.suspended{
            todo!()
        }
        self.suspended = true;
        self.update_thread_object();
        Ok(())
    }
    pub fn set_unsuspended(&mut self) -> Result<(), ResumeError> {
        if !self.suspended{
            todo!()
        }
        self.suspended = false;
        self.update_thread_object();
        Ok(())
    }
    pub fn set_gc_suspended(&mut self) -> Result<(), SuspendError> {
        self.gc_suspend = true;
        self.update_thread_object();
        Ok(())
    }
    pub fn set_gc_unsuspended(&mut self) -> Result<(), ResumeError> {
        assert!(self.gc_suspend);
        self.gc_suspend = false;
        self.update_thread_object();
        Ok(())
    }

    pub fn set_sleeping(&mut self, to_sleep: Duration) {
        let target_instant = Instant::now().checked_add(to_sleep).expect("If you're reading this you tried to sleep for longer than should be possible");
        self.sleep_until = Some(target_instant);
        self.update_thread_object();
    }

    pub fn set_park(&mut self, time: Option<Duration>) {
        //todo is it possible that blocking on acquiring lock for inner causes weird unexpected latency here.
        self.park_until = time.map(|time|Instant::now().checked_add(time).expect("If you're reading this you tried to park for longer than should be possible"));
        self.update_thread_object()
    }

    pub fn set_unpark(&mut self) {
        self.park_until = None;
        self.update_thread_object();
    }

    pub fn set_interrupted(&mut self){
        self.interrupt = true;
        self.update_thread_object();
    }

    pub fn set_uninterrupted(&mut self){
        self.interrupt = false;
        self.update_thread_object();
    }

    pub fn terminate(&self) -> bool {
        self.terminate
    }

    pub fn alive(&self) -> bool {
        self.alive
    }

    pub fn gc_suspend(&self) -> bool {
        self.gc_suspend
    }

    pub fn interrupt(&self) -> bool {
        self.interrupt
    }

    pub fn suspended(&self) -> bool {
        self.suspended
    }

    pub fn parks(&self) -> i64 {
        self.parks
    }

    pub fn park_until(&self) -> Option<Instant> {
        self.park_until
    }

    pub fn sleep_until(&self) -> Option<Instant> {
        self.sleep_until
    }

    pub fn joining_thread(&self) -> Option<JavaThreadId> {
        self.joining_thread
    }

    pub fn waiting_monitor_lock(&self) -> Option<MonitorID> {
        self.waiting_monitor_lock
    }

    pub fn waiting_monitor_notify(&self) -> Option<MonitorWait> {
        self.waiting_monitor_notify
    }
}

