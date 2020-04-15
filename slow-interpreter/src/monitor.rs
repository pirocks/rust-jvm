use parking_lot::{RawFairMutex, RawThreadId, FairMutex};
use lock_api::{ReentrantMutex, GetThreadId, RawMutex};
use std::sync::{Condvar, RwLock, Mutex};
use std::time::Duration;
use std::cell::RefCell;
use std::sync::atomic::{AtomicUsize, Ordering};
use futures_intrusive::sync::Semaphore;
use std::fmt::{Debug, Formatter, Error};

#[derive(Debug)]
pub struct OwningThreadAndCount{
    owner : usize,
    count : usize
}

pub struct Monitor {
    pub owned: RwLock<OwningThreadAndCount>,
    pub mutex: RawFairMutex,
    pub monitor_i: usize,
    pub condvar: Condvar,
    pub condvar_mutex: std::sync::Mutex<()>,
    pub name: String
}
impl Debug for Monitor{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(),Error> {
        write!(f, "[Monitor:{}]",self.name)
    }
}


impl Monitor {
    pub fn new(name : String, i: usize) -> Self{
        Self{
            owned: RwLock::new(OwningThreadAndCount{ owner: 0, count: 0 }),
            mutex: RawFairMutex::INIT,
            monitor_i: i,
            condvar: Condvar::new(),
            condvar_mutex: Mutex::new(()),
            name
        }
    }

    fn get_thread(&self)-> usize{
        RawThreadId{}.nonzero_thread_id().get()
    }

    pub fn lock(&self) {
        println!("Monitor lock:{}, thread:{}",self.name, self.get_thread());
        let mut current_owners_guard = self.owned.write().unwrap();
        if current_owners_guard.owner == self.get_thread(){
            current_owners_guard.count += 1;
        }else {
            std::mem::drop(current_owners_guard);//todo I don;t think there should be two guards here
            self.mutex.lock();
            let mut new_guard = self.owned.write().unwrap();
            new_guard.count = 1;
            new_guard.owner = self.get_thread();
        }
    }

    pub fn unlock(&self) {
        println!("Monitor unlock:{}, thread:{}",self.name, self.get_thread());
        let mut current_owners_guard = self.owned.write().unwrap();
        assert_eq!(current_owners_guard.owner,self.get_thread());
        current_owners_guard.count -= 1;
        if current_owners_guard.count == 0 {
            self.mutex.unlock();
        }
    }

    pub fn wait(&self, millis: i64) {
        println!("Monitor wait:{}, thread:{}",self.name, self.get_thread());
        self.owned.write().unwrap().owner = 0;
        self.mutex.unlock();
        if millis < 0 {
            self.condvar.wait(self.condvar_mutex.lock().unwrap()).unwrap();
        } else {
            self.condvar.wait_timeout(self.condvar_mutex.lock().unwrap(), Duration::from_millis(millis as u64)).unwrap();
        }
    }

    pub fn notify_all(&self) {
        println!("Monitor notify all:{}, thread:{}",self.name, self.get_thread());
        self.condvar.notify_all();
    }

    pub fn notify(&self) {
        println!("Monitor notify:{}, thread:{}",self.name, self.get_thread());
        self.condvar.notify_one();
    }
}