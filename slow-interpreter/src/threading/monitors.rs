use std::fmt::{Debug, Error, Formatter};
use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::time::Duration;

use parking_lot::{const_fair_mutex, FairMutex};

use crate::JVMState;

#[derive(Debug)]
pub struct OwningThreadAndCount {
    owner: Option<usize>,
    count: usize,
}

pub struct Monitor {
    //metadata:
    pub monitor_i: usize,
    pub name: String,
    //essentially a reentrant lock:
    pub owned: RwLock<OwningThreadAndCount>,
    pub mutex: Arc<FairMutex<()>>,

    //condvar for notify/wait
    pub condvar: Condvar,
    pub condvar_mutex: std::sync::Mutex<()>,
}

impl Debug for Monitor {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "[Monitor:{}]", self.name)
    }
}

impl Monitor {
    pub fn new(name: String, i: usize) -> Self {
        Self {
            owned: RwLock::new(OwningThreadAndCount { owner: None, count: 0 }),
            mutex: Arc::new(const_fair_mutex(())),
            monitor_i: i,
            condvar: Condvar::new(),
            condvar_mutex: Mutex::new(()),
            name,
        }
    }

    pub fn lock(&self, jvm: &'gc_life JVMState<'gc_life>) {
        jvm.config.tracing.trace_monitor_lock(self, jvm);
        self.lock_impl(jvm)
    }

    fn lock_impl(&self, jvm: &'gc_life JVMState<'gc_life>) {
        let mut current_owners_guard = self.owned.write().unwrap();
        //first we check if we currently own the lock. If we do increment and return.
        //If we do not currently hold the lock then we will continue to not own the lock until
        // std::mem::forget(self.mutex.lock()); returns.
        if current_owners_guard.owner == Monitor::get_tid(jvm).into() {
            current_owners_guard.count += 1;
        } else {
            std::mem::drop(current_owners_guard);
            std::mem::forget(self.mutex.lock());
            let mut new_guard = self.owned.write().unwrap();
            assert_eq!(new_guard.count, 0);
            new_guard.count = 1;
            new_guard.owner = Monitor::get_tid(jvm).into();
        }
    }

    pub fn unlock(&self, jvm: &'gc_life JVMState<'gc_life>) {
        jvm.config.tracing.trace_monitor_unlock(self, jvm);
        let mut current_owners_guard = self.owned.write().unwrap();
        assert_eq!(current_owners_guard.owner, Monitor::get_tid(jvm).into());
        current_owners_guard.count -= 1;
        if current_owners_guard.count == 0 {
            current_owners_guard.owner = None;
            unsafe {
                self.mutex.force_unlock_fair();
            }
        }
    }

    pub fn wait(&self, millis: i64, jvm: &'gc_life JVMState<'gc_life>) {
        jvm.config.tracing.trace_monitor_wait(self, jvm);
        let mut count_and_owner = self.owned.write().unwrap();
        if count_and_owner.owner != Monitor::get_tid(jvm).into() {
            // in javaspace this throws an illegal monitor exception.
            unimplemented!()
        }
        // wait requires us to release hold on reentrant lock, but reacquire same count on notify
        // instead of repeatedly unlocking, just set count to 0 and unlock.
        let count = count_and_owner.count;
        count_and_owner.count = 0;
        count_and_owner.owner = None;
        let guard1 = self.condvar_mutex.lock().unwrap();
        unsafe {
            self.mutex.force_unlock_fair();
        }
        std::mem::drop(count_and_owner);
        //after this line any other thread can now lock.
        // assert!(millis >= 0);// would throw an illegal argument exception, however the java agent seems to use -1 instead of 0
        if millis <= 0 {
            std::mem::drop(self.condvar.wait(guard1).unwrap());
        } else {
            std::mem::drop(self.condvar.wait_timeout(guard1, Duration::from_millis(millis as u64)).unwrap());
        }
        //now reacquire the same count as earlier:
        std::mem::forget(self.mutex.lock());
        let mut write_guard = self.owned.write().unwrap();
        write_guard.owner = Monitor::get_tid(jvm).into();
        write_guard.count = count;
    }

    pub fn destroy(&self, jvm: &'gc_life JVMState<'gc_life>) -> Result<(), MonitorOwnedBySomeoneElse> {
        let mut current_owners_guard = self.owned.write().unwrap();
        if current_owners_guard.owner != Monitor::get_tid(jvm).into() {
            return Result::Err(MonitorOwnedBySomeoneElse {});
        }
        current_owners_guard.count = 0;
        current_owners_guard.owner = None;
        unsafe {
            self.mutex.force_unlock_fair();
        }
        Result::Ok(())
    }

    pub fn get_tid(jvm: &'gc_life JVMState<'gc_life>) -> usize {
        jvm.thread_state.get_current_thread().java_tid as usize
    }

    pub fn notify_all(&self, jvm: &'gc_life JVMState<'gc_life>) {
        jvm.config.tracing.trace_monitor_notify_all(self, jvm);
        self.condvar.notify_all();
    }

    pub fn notify(&self, jvm: &'gc_life JVMState<'gc_life>) {
        jvm.config.tracing.trace_monitor_notify(self, jvm);
        self.condvar.notify_one();
    }

    pub fn this_thread_holds_lock(&self, jvm: &'gc_life JVMState<'gc_life>) -> bool {
        match self.owned.read().unwrap().owner.as_ref() {
            None => false,
            Some(owner_tid) => *owner_tid == Monitor::get_tid(jvm),
        }
    }
}

pub struct MonitorOwnedBySomeoneElse {}
