use parking_lot::{RawFairMutex, RawThreadId};
use lock_api::{RawMutex, Mutex, ReentrantMutex, GetThreadId};
use std::sync::Condvar;

#[derive(Debug)]
pub struct Monitor {
    pub mutex: ReentrantMutex<RawFairMutex, RawThreadId, ()>,//todo should prob check w/ java thread
    pub monitor_i: usize,
    pub condvar: Condvar,
    pub condvar_mutex: std::sync::Mutex<()>,
}

impl Monitor {
    pub fn lock(&self) {
         std::mem::forget(self.mutex.lock());
    }

    pub fn unlock(&self) {
        unsafe { self.mutex.force_unlock_fair(); }//todo maybe find something better than force_unlock
    }

    pub fn wait(&self) {
        self.condvar.wait(self.condvar_mutex.lock().unwrap()).unwrap();
    }

    pub fn notify_all(&self) {
        self.condvar.notify_all();
    }
}