use parking_lot::{RawFairMutex, const_fair_mutex};
use lock_api::{RawMutex, Mutex};

#[derive(Debug)]
pub struct Monitor {
    mutex: Mutex<RawFairMutex, i32>,
}

impl Monitor {
    pub fn lock(&self) {
        unsafe { self.mutex.raw().lock(); }
    }

    pub fn unlock(&self) {
        unsafe { self.mutex.raw().unlock(); }//todo maybe find something better than force_unlock
    }

    pub fn new() -> Self {
        Self {
            mutex: const_fair_mutex(0)
        }
    }
}