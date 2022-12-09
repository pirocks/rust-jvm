use std::collections::HashSet;
use std::ops::Add;
use std::sync::{Condvar, Mutex, RwLock};
use std::thread::current;
use std::time::{Duration, Instant};

use jvmti_jni_bindings::{jint, JVMTI_THREAD_STATE_ALIVE, JVMTI_THREAD_STATE_BLOCKED_ON_MONITOR_ENTER, JVMTI_THREAD_STATE_IN_OBJECT_WAIT, JVMTI_THREAD_STATE_INTERRUPTED, JVMTI_THREAD_STATE_PARKED, JVMTI_THREAD_STATE_RUNNABLE, JVMTI_THREAD_STATE_SLEEPING, JVMTI_THREAD_STATE_SUSPENDED, JVMTI_THREAD_STATE_TERMINATED, JVMTI_THREAD_STATE_WAITING, JVMTI_THREAD_STATE_WAITING_INDEFINITELY, JVMTI_THREAD_STATE_WAITING_WITH_TIMEOUT};
use rust_jvm_common::JavaThreadId;

use crate::WasException;
use crate::better_java_stack::frames::{HasFrame};
use crate::interpreter::safepoint_check;
use crate::jvm_state::JVMState;
use crate::stdlib::java::lang::throwable::Throwable;
use crate::threading::java_thread::{ResumeError, SuspendError, ThreadStatus};

// new approach to safepoints
// Needs to handle:
// 1. gc suspend/ stacktrace suspend
// 2. monitor lock/unlock/wait/join
// 3. park/unpark
// 4. remote exception throw// todo?
// 5. sleep blocking
// 6. nio blocking?
// 7. interrupt interrupting wait/lock/unlock/join/sleep
// 8. need to record what waiting on somehow
// 9. gc suspend wants a response from safepoint about stack position etc.
// 10.
//
//
// each change to safepoint state sets should check?
// this makes safepoint state tied to should check? and how?
// should be signal safe? - can't be b/c having to wait on mutex and to big/complex
//
//


pub type MonitorID = usize;

#[derive(Debug)]
pub struct MonitorWait {
    wait_until: Option<Instant>,
    monitor: MonitorID,
    prev_count: usize,
}

pub struct SafePointStopReasonState<'gc> {
    waiting_monitor_lock: Option<MonitorID>,
    pub(crate) waiting_monitor_notify: Option<MonitorWait>,
    suspended: bool,
    gc_suspended: bool,
    parks: isize,
    park_until: Option<Instant>,
    throw_exception: Option<Throwable<'gc>>,
    sleep_until: Option<Instant>,
}

impl<'gc> Default for SafePointStopReasonState<'gc> {
    fn default() -> Self {
        Self {
            waiting_monitor_lock: None,
            waiting_monitor_notify: None,
            suspended: false,
            gc_suspended: false,
            parks: 0,
            park_until: None,
            throw_exception: None,
            sleep_until: None,
        }
    }
}

pub struct SafePoint<'gc> {
    pub(crate) state: Mutex<SafePointStopReasonState<'gc>>,
    waiton: Condvar,
}

impl<'gc> SafePoint<'gc> {
    pub fn new() -> Self {
        Self { state: Mutex::new(Default::default()), waiton: Default::default() }
    }

    pub fn set_monitor_unlocked(&self) {
        let mut guard = self.state.lock().unwrap();
        // assert!(guard.waiting_monitor_lock.is_some());
        guard.waiting_monitor_lock = None;
        self.waiton.notify_one();
    }

    pub fn set_monitor_lock(&self, to: MonitorID) {
        let mut guard = self.state.lock().unwrap();
        assert!(guard.waiting_monitor_lock.is_none());
        guard.waiting_monitor_lock = Some(to);
        self.waiton.notify_one();
    }

    pub fn set_waiting_notify(&self, monitor: MonitorID, wait_until: Option<Instant>, prev_count: usize) {
        let mut guard = self.state.lock().unwrap();
        assert!(guard.waiting_monitor_notify.is_none());
        guard.waiting_monitor_notify = Some(MonitorWait { wait_until, monitor, prev_count });
        self.waiton.notify_one();
    }

    pub fn set_notified_once(&self) {
        let mut guard = self.state.lock().unwrap();
        assert!(guard.waiting_monitor_notify.is_some());
        let waiting_monitor_notify = guard.waiting_monitor_notify.as_mut().unwrap();
        let prev_count = &mut waiting_monitor_notify.prev_count; //todo wtf is this, we need more types for monitor wait
        *prev_count -= 1;
        if *prev_count == 0 {
            guard.waiting_monitor_notify = None
        }
        self.waiton.notify_one();
    }

    pub fn set_notified_all(&self) {
        let mut guard = self.state.lock().unwrap();
        assert!(guard.waiting_monitor_notify.is_some());
        guard.waiting_monitor_notify = None;
        self.waiton.notify_one();
    }

    pub fn set_suspended(&self) -> Result<(), SuspendError> {
        let mut guard = self.state.lock().unwrap();
        if guard.suspended {
            return Err(SuspendError::AlreadySuspended);
        }
        guard.suspended = true;
        self.waiton.notify_one();
        Ok(())
    }

    pub fn set_gc_suspended(&self) -> Result<(), SuspendError> {
        let mut guard = self.state.lock().unwrap();
        if guard.gc_suspended {
            return Err(SuspendError::AlreadySuspended);
        }
        guard.gc_suspended = true;
        self.waiton.notify_one();
        Ok(())
    }

    pub fn set_gc_unsuspended(&self) -> Result<(), ResumeError> {
        let mut guard = self.state.lock().unwrap();
        if !guard.gc_suspended {
            return Err(ResumeError::NotSuspended);
        }
        guard.gc_suspended = false;
        self.waiton.notify_one();
        Ok(())
    }

    pub fn set_unsuspended(&self) -> Result<(), ResumeError> {
        let mut guard = self.state.lock().unwrap();
        if !guard.suspended {
            return Err(ResumeError::NotSuspended);
        }
        guard.suspended = false;
        self.waiton.notify_one(); // technically I don't need this to be a condvar since I only use notify_one I could have this be a lock
        Ok(())
    }

    pub fn set_sleeping(&self, to_sleep: Duration) {
        let mut guard = self.state.lock().unwrap();
        guard.sleep_until = Some(Instant::now().add(to_sleep));
        self.waiton.notify_one();
    }

    pub fn set_park(&self, time: Option<Duration>) {
        let park_until = time.map(|time| Instant::now().add(time));
        let mut guard = self.state.lock().unwrap();
        assert!(guard.waiting_monitor_notify.is_none());
        assert!(guard.waiting_monitor_lock.is_none());
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
            if thread_status.terminated {
                res |= JVMTI_THREAD_STATE_TERMINATED;
            }
        }
        res as jint
    }
}

impl<'gc> SafePoint<'gc> {
    pub fn check<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl HasFrame<'gc>) -> Result<(), WasException<'gc>> {
        let guard = self.state.lock().unwrap();

        if guard.gc_suspended {
            // dbg!("gc suspended");
            let guard = self.waiton.wait(guard).unwrap();
            drop(guard);
            return self.check(jvm, int_state);
        }

        if let Some(exception) = &guard.throw_exception {
            todo!();
            // int_state.set_throw(Some(exception.clone().to_allocated_object().into()));
            return Err(WasException { exception_obj: todo!() });
        }
        if guard.suspended {
            // dbg!("regular suspended");
            let _unused = self.waiton.wait(guard).unwrap();
            // let current_thread = jvm.thread_state.get_current_thread();
            drop(_unused);
            return self.check(jvm, int_state);
        }
        if guard.parks > 0 {
            // dbg!(guard.parks);
            // dbg!("parked wait");
            // assert!(guard.waiting_monitor_notify.is_none());
            // assert!(guard.waiting_monitor_lock.is_none());
            let _unused = self.waiton.wait(guard).unwrap();
            // dbg!(_unused.parks);
            // assert!(_unused.waiting_monitor_notify.is_none());
            // assert!(_unused.waiting_monitor_lock.is_none());
            // dbg!("recheck");
            drop(_unused);
            let guard = self.state.lock().unwrap();
            // assert!(guard.waiting_monitor_notify.is_none());
            // assert!(guard.waiting_monitor_lock.is_none());
            drop(guard);
            return self.check(jvm, int_state);
        }
        if let Some(_) = &guard.waiting_monitor_lock {
            let guard = self.waiton.wait(guard).unwrap();
            drop(guard);
            return self.check(jvm, int_state);
        }
        if let Some(MonitorWait { wait_until, monitor, prev_count }) = &guard.waiting_monitor_notify {
            // int_state.debug_print_stack_trace(jvm);
            // dbg!("monitor wait");
            let wait_until = *wait_until;
            let monitor = *monitor;
            let prev_count = *prev_count;
            let time_to_wait = wait_until.map(|wait_until| match wait_until.checked_duration_since(Instant::now()) {
                None => Duration::new(0, 0),
                Some(time_to_wait) => time_to_wait,
            });
            let (mut guard, should_reacquire) = match time_to_wait {
                None => {
                    let guard = self.waiton.wait(guard).unwrap();
                    let should_reacquire = guard.waiting_monitor_notify.is_none();
                    (guard, should_reacquire)
                }
                Some(time_to_wait) => {
                    let (guard, timeout) = self.waiton.wait_timeout(guard, time_to_wait).unwrap();
                    let should_reacquire = if timeout.timed_out() { wait_until.unwrap().checked_duration_since(Instant::now()).is_none() } else { guard.waiting_monitor_notify.is_none() };
                    (guard, should_reacquire)
                }
            };

            guard.waiting_monitor_notify = None;
            return if should_reacquire {
                let monitors_gaurd = jvm.thread_state.monitors.read().unwrap();
                let monitor = &monitors_gaurd[monitor].clone();
                drop(guard);
                drop(monitors_gaurd);
                monitor.notify_reacquire(jvm, int_state, prev_count)?;
                self.check(jvm, int_state)
            } else {
                drop(guard); //shouldn't need these but they are here for now b/c I'm paranoid
                self.check(jvm, int_state)
            };
        }
        Ok(())
    }
}

pub struct Monitor2 {
    pub id: MonitorID,
    monitor2_priv: RwLock<Monitor2Priv>,
}

pub struct Monitor2Priv {
    pub owner: Option<JavaThreadId>,
    pub count: usize,
    pub waiting_notify: HashSet<JavaThreadId>,
    pub waiting_lock: HashSet<JavaThreadId>,
}

impl Monitor2 {
    pub fn new(id: MonitorID) -> Self {
        Self {
            id,
            monitor2_priv: RwLock::new(Monitor2Priv { owner: None, count: 0, waiting_notify: HashSet::new(), waiting_lock: HashSet::new() }),
        }
    }

    pub fn lock<'l, 'gc>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl HasFrame<'gc>) -> Result<(), WasException<'gc>> {
        let mut guard = self.monitor2_priv.write().unwrap();
        let current_thread = jvm.thread_state.get_current_thread();
        if let Some(owner) = guard.owner.as_ref() {
            if *owner == current_thread.java_tid {
                guard.count += 1;
            } else {
                guard.waiting_lock.insert(current_thread.java_tid);
                current_thread.safepoint_state.set_monitor_lock(self.id);
                drop(guard);
                safepoint_check(jvm, int_state).unwrap();
                let mut guard = self.monitor2_priv.write().unwrap();
                if guard.owner.is_none() {
                    guard.waiting_lock.remove(&current_thread.java_tid);
                    guard.owner = Some(current_thread.java_tid);
                    guard.count = 1;
                    current_thread.safepoint_state.set_monitor_unlocked();
                    drop(guard);
                } else {
                    drop(guard);
                    return self.lock(jvm, int_state);
                }
            }
        } else {
            guard.owner = Some(current_thread.java_tid);
            guard.count = 1;
            drop(guard);
        }
        safepoint_check(jvm, int_state).unwrap();
        Ok(())
    }

    pub fn unlock<'gc, 'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl HasFrame<'gc>) -> Result<(), WasException<'gc>> {
        let mut guard = self.monitor2_priv.write().unwrap();
        let current_thread = jvm.thread_state.get_current_thread();
        if guard.owner == current_thread.java_tid.into() {
            guard.count -= 1;
            if guard.count == 0 {
                guard.owner = None;
                if let Some(tid) = guard.waiting_lock.drain().next() {
                    let to_unlock_thread = jvm.thread_state.get_thread_by_tid(tid);
                    to_unlock_thread.safepoint_state.set_monitor_unlocked();
                }
            }
        } else {
            dbg!(guard.owner);
            int_state.debug_print_stack_trace(jvm);
            dbg!(guard.owner);
            todo!("illegal monitor state")
        }
        drop(guard);
        safepoint_check(jvm, int_state).unwrap();
        Ok(())
    }

    pub fn notify<'gc>(&self, jvm: &'gc JVMState<'gc>) -> Result<(), WasException<'gc>> {
        if jvm.thread_tracing_options.trace_monitor_notify {
            eprintln!("[{}] Notify: {}", current().name().unwrap_or("Unknown Thread"), self.id);
        }
        let mut guard = self.monitor2_priv.write().unwrap();
        if let Some(to_notify) = guard.waiting_notify.drain().next() {
            let to_notify_thread = jvm.thread_state.get_thread_by_tid(to_notify);
            to_notify_thread.safepoint_state.set_notified_once();
        }
        Ok(())
    }

    pub fn notify_all<'gc>(&self, jvm: &'gc JVMState<'gc>) -> Result<(), WasException<'gc>> {
        if jvm.thread_tracing_options.trace_monitor_notify_all {
            eprintln!("[{}] Notify All: {}", current().name().unwrap_or("Unknown Thread"), self.id);
        }
        let mut guard = self.monitor2_priv.write().unwrap();
        for to_notify in guard.waiting_notify.drain() {
            let to_notify_thread = jvm.thread_state.get_thread_by_tid(to_notify);
            to_notify_thread.safepoint_state.set_notified_all();
        }
        Ok(())
    }

    pub fn wait<'gc, 'k>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl HasFrame<'gc>, wait_duration: Option<Duration>) -> Result<(), WasException<'gc>> {
        if jvm.thread_tracing_options.trace_monitor_wait_enter {
            eprintln!("[{}] Monitor Wait: {}", current().name().unwrap_or("Unknown Thread"), self.id);
        }
        let mut guard = self.monitor2_priv.write().unwrap();
        let now = Instant::now();
        let wait_until = wait_duration.map(|wait_duration| match now.checked_add(wait_duration) {
            None => panic!("If you are reading this there something wrong with the amount of time you are calling a wait for"),
            Some(wait_until) => wait_until,
        });
        let current_thread = jvm.thread_state.get_current_thread();
        let prev_count = guard.count;
        if guard.owner == current_thread.java_tid.into() {
            guard.owner = None;
            guard.waiting_notify.insert(current_thread.java_tid);
            current_thread.safepoint_state.set_waiting_notify(self.id, wait_until, prev_count);
        } else {
            todo!();// int_state.debug_print_stack_trace(jvm);
            todo!("throw illegal monitor state")
        }
        drop(guard);
        safepoint_check(jvm, int_state).unwrap();
        assert_eq!(self.monitor2_priv.read().unwrap().owner, current_thread.java_tid.into());
        assert_eq!(self.monitor2_priv.read().unwrap().count, prev_count);
        if jvm.thread_tracing_options.trace_monitor_wait_exit {
            eprintln!("[{}] Monitor Wait Exit: {}", current().name().unwrap_or("Unknown Thread"), self.id);
        }
        Ok(())
    }

    pub fn notify_reacquire<'gc, 'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl HasFrame<'gc>, prev_count: usize) -> Result<(), WasException<'gc>> {
        self.lock(jvm, int_state)?;
        let current_thread = jvm.thread_state.get_current_thread();
        let mut guard = self.monitor2_priv.write().unwrap(); //todo likely race here
        guard.count = prev_count;
        guard.owner = Some(current_thread.java_tid);
        Ok(())
    }

    pub fn this_thread_holds_lock<'gc>(&self, jvm: &'gc JVMState<'gc>) -> bool {
        let current_thread = jvm.thread_state.get_current_thread();
        self.monitor2_priv.read().unwrap().owner == Some(current_thread.java_tid)
    }
}