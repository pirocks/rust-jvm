use parking_lot::RawFairMutex;
use lock_api::{RawMutex, Mutex};

#[derive(Debug)]
pub struct Monitor {
    pub mutex: Mutex<RawFairMutex, i32>,
    pub monitor_i : usize
}

impl Monitor {
    pub fn lock(&self) {
        unsafe { self.mutex.raw().lock(); }
    }

    pub fn unlock(&self) {
        unsafe { self.mutex.raw().unlock(); }//todo maybe find something better than force_unlock
    }
}