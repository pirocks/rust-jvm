use parking_lot::RawFairMutex;
use lock_api::{RawMutex, Mutex};
use std::sync::Condvar;

#[derive(Debug)]
pub struct Monitor {
    pub mutex: Mutex<RawFairMutex, i32>,
    pub monitor_i : usize,
    pub condvar: Condvar,
    pub condvar_mutex : std::sync::Mutex<()>
}

impl Monitor {
    pub fn lock(&self) {
        unsafe { self.mutex.raw().lock(); }
    }

    pub fn unlock(&self) {
        unsafe { self.mutex.raw().unlock(); }//todo maybe find something better than force_unlock
    }

    pub fn wait(&self) {
        self.condvar.wait(self.condvar_mutex.lock().unwrap()).unwrap();
    }
}