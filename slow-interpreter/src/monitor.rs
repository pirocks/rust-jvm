use parking_lot::{RawFairMutex, RawThreadId};
use lock_api::ReentrantMutex;
use std::sync::Condvar;
use std::time::Duration;

#[derive(Debug)]
pub struct Monitor {
    pub mutex: ReentrantMutex<RawFairMutex, RawThreadId, ()>,
    //todo should prob check w/ java thread
    pub monitor_i: usize,
    pub condvar: Condvar,
    pub condvar_mutex: std::sync::Mutex<()>,
    pub name: String
}

impl Monitor {
    pub fn lock(&self) {
        std::mem::forget(self.mutex.lock());
    }

    pub fn unlock(&self) {
        unsafe { self.mutex.force_unlock_fair(); }//todo maybe find something better than force_unlock
    }

    pub fn wait(&self, millis: i64) {
        if millis < 0 {
            self.condvar.wait(self.condvar_mutex.lock().unwrap()).unwrap();
        } else {
            self.condvar.wait_timeout(self.condvar_mutex.lock().unwrap(), Duration::from_millis(millis as u64)).unwrap();
        }
    }

    pub fn notify_all(&self) {
        self.condvar.notify_all();
    }

    pub fn notify(&self) {
        self.condvar.notify_one();
    }
}