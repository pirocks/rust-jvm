use std::ops::Add;
use std::sync::{Condvar, Mutex};
use std::time::{Duration, Instant};


use crate::better_java_stack::frames::PushableFrame;
use crate::exceptions::WasException;
use crate::jvm_state::JVMState;
use crate::new_java_values::owned_casts::OwnedCastAble;
use crate::new_sync_point_state::inner::{MonitorWait, SafePointStateInner};
use crate::stdlib::java::lang::interrupted_exception::InterruptedException;
use crate::stdlib::java::lang::thread::JThread;
use crate::threading::java_thread::{ResumeError, SuspendError};
use crate::threading::safepoints::MonitorID;

pub enum ThreadOrBootstrap<'gc> {
    Bootstrap,
    Thread(JThread<'gc>),
}


pub mod inner;

pub struct NewSafePointState<'gc> {
    inner: Mutex<SafePointStateInner<'gc>>,
    condvar: Condvar,
}

pub struct TimedOut{}

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
            if guard.terminate() && !guard.alive(){
                break;
            }
            guard = self.condvar.wait(guard).unwrap();
        }
    }

    pub fn check(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<Result<(), TimedOut>, WasException<'gc>> {
        let mut guard = self.inner.lock().unwrap();
        loop {
            if guard.terminate() {
                todo!("exit the whole thread")
            }

            if guard.gc_suspend() {
                todo!("blocking on gc impl")
            }

            if guard.interrupt() {
                guard.set_uninterrupted();//needed as to not interrupt the exception constructor
                let exception = InterruptedException::new(jvm, int_state).expect("todo handle exceptions during exception creation");
                return Err(WasException { exception_obj: exception.normal_object.cast_throwable() })
            }

            if guard.suspended() {
                guard = self.condvar.wait(guard).unwrap();
                continue;
            }

            if guard.parks() > 0 {
                match guard.park_until() {
                    None => {
                        guard = self.condvar.wait(guard).unwrap();
                    }
                    Some(park_until) => {
                        let duration = match park_until.checked_duration_since(Instant::now()) {
                            Some(x) => x,
                            None => {
                                return Ok(Err(TimedOut {}))
                            }
                        };
                        let (guard_tmp, _) = self.condvar.wait_timeout(guard, duration).unwrap();
                        guard = guard_tmp;
                    }
                }
                continue;
            }

            if let Some(sleep_until) = guard.sleep_until() {
                let duration = match sleep_until.checked_duration_since(Instant::now()) {
                    Some(x) => x,
                    None => {
                        return Ok(Err(TimedOut {}))
                    }
                };
                let (guard_tmp, _) = self.condvar.wait_timeout(guard, duration).unwrap();
                guard = guard_tmp;
                continue;
            }

            if let Some(joining_thread) = guard.joining_thread() {
                todo!()
            }

            if let Some(waiting_monitor_lock) = guard.waiting_monitor_lock() {
                let guard_tmp = self.condvar.wait(guard).unwrap();
                guard = guard_tmp;
                continue;
            }

            if let Some(MonitorWait{ wait_until, monitor:_ }) = guard.waiting_monitor_notify() {
                let duration = match wait_until {
                    None => None,
                    Some(wait_until) => {
                        match wait_until.checked_duration_since(Instant::now()) {
                            Some(duration) => Some(duration),
                            None => {
                                return Ok(Err(TimedOut {}))
                            }
                        }
                    }
                };

                let guard_tmp = match duration {
                    None => {
                        // int_state.debug_print_stack_trace(jvm);
                        // println!("wait {}", std::thread::current().name().unwrap());
                        self.condvar.wait(guard).unwrap()
                    }
                    Some(duration) => {
                        // println!("timed wait {} {}", std::thread::current().name().unwrap(), duration.as_secs_f64());
                        let res = self.condvar.wait_timeout(guard, duration).unwrap().0;
                        // println!("timed wait finish {} {}", std::thread::current().name().unwrap(), duration.as_secs_f64());
                        res
                    }
                };
                guard = guard_tmp;
                continue;
            }

            return Ok(Ok(()));
        }
    }

    pub fn set_monitor_unlocked(&self) {
        let mut guard = self.inner.lock().unwrap();
        guard.set_monitor_unlocked();
        self.condvar.notify_all();
    }

    pub fn set_monitor_lock(&self, to: MonitorID) {
        let mut guard = self.inner.lock().unwrap();
        assert!(guard.waiting_monitor_lock().is_none());
        guard.set_monitor_lock(to);
        self.condvar.notify_all();
    }

    pub fn set_waiting_notify(&self, monitor: MonitorID, wait_until: Option<Instant>) {
        let mut guard = self.inner.lock().unwrap();
        assert!(guard.waiting_monitor_notify().is_none());
        guard.set_waiting_notify(monitor, wait_until);
        self.condvar.notify_all();
    }

    pub fn set_notified(&self, monitor_id: MonitorID) {
        let mut guard = self.inner.lock().unwrap();
        // assert!(guard.waiting_monitor_notify().is_some());/// not true b/c timeouts exist
        // assert_eq!(guard.waiting_monitor_notify().unwrap().monitor, monitor_id);
        guard.set_notified();
        self.condvar.notify_all();
    }

    pub fn set_suspended(&self) -> Result<(), SuspendError> {
        let mut guard = self.inner.lock().unwrap();
        if guard.suspended() {
            return Err(SuspendError::AlreadySuspended);
        }
        guard.set_suspended()?;
        self.condvar.notify_all();
        Ok(())
    }

    pub fn set_gc_suspended(&self) -> Result<(), SuspendError> {
        let mut guard = self.inner.lock().unwrap();
        if guard.gc_suspend() {
            return Err(SuspendError::AlreadySuspended);
        }
        guard.set_gc_suspended()?;
        self.condvar.notify_all();
        Ok(())
    }

    pub fn set_gc_unsuspended(&self) -> Result<(), ResumeError> {
        let mut guard = self.inner.lock().unwrap();
        if !guard.gc_suspend() {
            return Err(ResumeError::NotSuspended);
        }
        guard.set_gc_unsuspended()?;
        self.condvar.notify_all();
        Ok(())
    }

    pub fn set_unsuspended(&self) -> Result<(), ResumeError> {
        let mut guard = self.inner.lock().unwrap();
        if !guard.suspended() {
            return Err(ResumeError::NotSuspended);
        }
        guard.set_unsuspended()?;
        self.condvar.notify_all();
        Ok(())
    }

    pub fn set_sleeping(&self, to_sleep: Duration) {
        let mut guard = self.inner.lock().unwrap();
        guard.set_sleeping(to_sleep);
        self.condvar.notify_all();
    }

    pub fn set_park(&self, time: Option<Duration>) {
        let park_until = time.map(|time| Instant::now().add(time));
        let mut guard = self.inner.lock().unwrap();
        assert!(guard.waiting_monitor_notify().is_none());
        assert!(guard.waiting_monitor_lock().is_none());
        guard.set_park(time);
        self.condvar.notify_all()
    }

    pub fn set_unpark(&self) {
        let mut guard = self.inner.lock().unwrap();
        assert!(guard.parks() <= 1);
        guard.set_unpark();
        self.condvar.notify_all()
    }

    pub fn set_interrupted(&self){
        let mut guard = self.inner.lock().unwrap();
        guard.set_interrupted();
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
        self.inner.lock().unwrap().interrupt()
    }

    pub fn is_alive(&self) -> bool{
        self.inner.lock().unwrap().alive()
    }

    pub fn is_waiting_notify(&self) -> bool{
        self.inner.lock().unwrap().waiting_monitor_notify().is_some()
    }
}
