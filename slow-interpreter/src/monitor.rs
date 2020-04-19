use parking_lot::{FairMutex, const_fair_mutex};
use std::sync::{Condvar, RwLock, Mutex};
use std::time::Duration;
use std::fmt::{Debug, Formatter, Error};
use crate::JVMState;

#[derive(Debug)]
pub struct OwningThreadAndCount{
    owner : usize,
    count : usize
}

pub struct Monitor {
    pub owned: RwLock<OwningThreadAndCount>,
    pub mutex: FairMutex<()>,
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
            mutex: const_fair_mutex(()),
            monitor_i: i,
            condvar: Condvar::new(),
            condvar_mutex: Mutex::new(()),
            name
        }
    }

    pub fn lock(&self, jvm : &JVMState) {
        println!("Monitor lock:{}/{}, thread:{} {}",self.name,self.monitor_i, std::thread::current().name().unwrap_or("unknown"),Monitor::get_tid(jvm));
        let mut current_owners_guard = self.owned.write().unwrap();
        if current_owners_guard.owner == Monitor::get_tid(jvm){
            current_owners_guard.count += 1;
        }else {
            std::mem::drop(current_owners_guard);//todo I don;t think there should be two guards here
            std::mem::forget(self.mutex.lock());
            let mut new_guard = self.owned.write().unwrap();
            assert_eq!(new_guard.count, 0);
            new_guard.count = 1;
            new_guard.owner = Monitor::get_tid(jvm);
        }
    }

    pub fn unlock(&self, jvm : &JVMState) {
        println!("Monitor unlock:{}/{}, thread:{} {}",self.name,self.monitor_i, std::thread::current().name().unwrap_or("unknown"),Monitor::get_tid(jvm));
        let mut current_owners_guard = self.owned.write().unwrap();
        assert_eq!(current_owners_guard.owner,Monitor::get_tid(jvm));
        current_owners_guard.count -= 1;
        if current_owners_guard.count == 0 {
            current_owners_guard.owner = 0;
            unsafe {self.mutex.force_unlock_fair();}
        }
    }

    pub fn wait(&self, millis: i64, jvm : &JVMState) {
        println!("Monitor wait:{}, thread:{}",self.name, std::thread::current().name().unwrap_or("unknown"));
        let mut guard = self.owned.write().unwrap();
        let count = guard.count;
        guard.count = 0;
        guard.owner = 0;
        let guard1 = self.condvar_mutex.lock().unwrap();
        unsafe {self.mutex.force_unlock_fair();}
        std::mem::drop(guard);
        if millis < 0 {
            self.condvar.wait(guard1).unwrap();
        } else {
            self.condvar.wait_timeout(guard1, Duration::from_millis(millis as u64)).unwrap();
        }
        std::mem::forget(self.mutex.lock());
        let mut write_guard = self.owned.write().unwrap();
        write_guard.owner = Monitor::get_tid(jvm);
        write_guard.count = count;
    }

    fn get_tid(jvm: &JVMState) -> usize {
        jvm.get_current_thread().java_tid as usize
    }

    pub fn notify_all(&self, jvm : &JVMState) {
        println!("Monitor notify all:{}, thread:{}",self.name, std::thread::current().name().unwrap_or("unknown"));
        self.condvar.notify_all();
    }

    pub fn notify(&self, jvm : &JVMState) {
        println!("Monitor notify:{}, thread:{}",self.name, std::thread::current().name().unwrap_or("unknown"));
        self.condvar.notify_one();
    }
}