use std::any::Any;
use std::cell::RefCell;
use std::ffi::c_void;
use std::mem::transmute;
use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::thread::{Builder, LocalKey, Scope, ScopedJoinHandle};


use crate::signal::{pthread_self, pthread_t};

// type TID = usize;

pub struct Threads<'vm> {
    this_thread: &'static LocalKey<RefCell<Option<Arc<Thread<'static>>>>>,
    scope: &'vm Scope<'vm, 'vm>,
}

static mut THERE_CAN_ONLY_BE_ONE_THREADS: bool = false;

thread_local! {
    static THIS_THREAD: RefCell<Option<Arc<Thread<'static>>>> = RefCell::new(None);
}

impl<'vm> Threads<'vm> {
    pub fn this_thread(&self) -> Arc<Thread> {
        self.this_thread.with(|thread| unsafe { transmute(thread.borrow().as_ref().unwrap().clone()) })
    }

    pub fn new(scope: &'vm Scope<'vm, 'vm>) -> Threads<'vm> {
        unsafe {
            if THERE_CAN_ONLY_BE_ONE_THREADS {
                panic!()
            }
            THERE_CAN_ONLY_BE_ONE_THREADS = true;
        }

        Threads { this_thread: &THIS_THREAD, scope }
    }

    pub fn create_thread(&'vm self, name: Option<String>) -> Thread<'vm> {
        let join_status = Arc::new(RwLock::new(JoinStatus {
            finished_mutex: Mutex::new(()),
            alive: AtomicBool::new(false),
            thread_finished: Condvar::new(),
        }));
        let started = AtomicBool::new(false);
        let mut res = Thread {
            started,
            join_status: join_status.clone(),
            pause: PauseStatus { paused_mutex: Mutex::new(false), paused: Condvar::new() },
            pthread_id: None,
            rust_join_handle: None,
            thread_start_channel_send: None,
        };
        let (thread_info_channel_send, thread_info_channel_recv) = std::sync::mpsc::channel();
        let (thread_start_channel_send, thread_start_channel_recv) = std::sync::mpsc::channel();
        let mut builder = Builder::new();
        builder = match name.as_ref() {
            None => builder,
            Some(name) => builder.name(name.chars().take(15).collect::<String>()),
        };
        let join_handle = builder
            .stack_size(1024 * 1024 * 256) // verifier makes heavy use of recursion.
            .spawn_scoped(self.scope, move || unsafe {
                join_status.write().unwrap().alive.store(true, Ordering::SeqCst);
                thread_info_channel_send.send(pthread_self()).unwrap();
                let ThreadStartInfo { func, data } = thread_start_channel_recv.recv().unwrap();
                func(data);
                join_status.read().unwrap().thread_finished.notify_all();
            })
            .unwrap();
        res.thread_start_channel_send = Mutex::new(thread_start_channel_send).into();
        res.pthread_id = thread_info_channel_recv.recv().unwrap().into();
        res.rust_join_handle = Some(join_handle);
        res
    }
}

pub struct ThreadStartInfo<'vm> {
    func: Box<dyn FnOnce(Box<dyn Any>) + 'vm>,
    data: Box<dyn Any>,
}

unsafe impl Send for ThreadStartInfo<'_> {}

unsafe impl Sync for ThreadStartInfo<'_> {}

#[derive(Debug)]
pub struct Thread<'vm> {
    started: AtomicBool,
    join_status: Arc<RwLock<JoinStatus>>,
    pause: PauseStatus,
    pthread_id: Option<pthread_t>,
    rust_join_handle: Option<ScopedJoinHandle<'vm, ()>>,
    thread_start_channel_send: Option<Mutex<Sender<ThreadStartInfo<'vm>>>>,
}

#[derive(Debug)]
pub struct PauseStatus {
    paused_mutex: Mutex<bool>,
    paused: Condvar, //todo maybe use rust park() for this
}

#[derive(Debug)]
pub struct JoinStatus {
    alive: AtomicBool,
    finished_mutex: Mutex<()>,
    //todo combine alive AtomicBool and Mutex
    thread_finished: Condvar,
}

impl<'vm> Thread<'vm> {
    pub fn start_thread<T: 'vm>(&self, func: Box<T>, data: Box<dyn Any>)
        where
            T: FnOnce(Box<dyn Any>),
    {
        self.thread_start_channel_send.as_ref().unwrap().lock().unwrap().send(ThreadStartInfo { func, data }).unwrap();
        self.started.store(true, Ordering::SeqCst);
    }

    pub fn pause(&self) {
        unsafe {
            assert_eq!(self.pthread_id.unwrap(), pthread_self());
        }
        drop(self.pause.paused.wait(self.pause.paused_mutex.lock().unwrap()).unwrap());
    }

    pub fn is_paused(&self) -> bool {
        *self.pause.paused_mutex.lock().unwrap()
    }

    pub fn resume(&self) {
        self.pause.paused.notify_one();
    }

    pub fn is_alive(&self) -> bool {
        let guard = self.join_status.read().unwrap();
        guard.alive.load(Ordering::SeqCst)
    }

    pub fn join(&self) {
        let guard = self.join_status.read().unwrap();
        assert!(guard.alive.load(Ordering::SeqCst));
        drop(guard.thread_finished.wait(guard.finished_mutex.lock().unwrap()).unwrap());
    }

    pub unsafe fn is_this_thread(&self) -> bool {
        self.pthread_id == pthread_self().into()
    }

    pub fn pthread_id(&self) -> pthread_t{
        self.pthread_id.unwrap()
    }
}

pub enum SignalReason<'vm> {
    Pause(*const Threads<'vm>),
    Event(AnEvent),
}

pub struct AnEvent {
    pub event_handler: unsafe extern "C" fn(data: *mut c_void),
    pub data: *mut c_void,
}

pub mod handlers;
pub mod context;
pub mod signal;