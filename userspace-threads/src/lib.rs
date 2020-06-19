#![feature(box_syntax)]

use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::ffi::c_void;
use std::mem::transmute;
use std::ptr::null_mut;
use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::LocalKey;

use nix::errno::errno;
use nix::sys::signal::{SigAction, sigaction, SigHandler, SigSet};
use nix::sys::signal::Signal;
use nix::unistd::{gettid, Pid};

use crate::handlers::{handle_event, handle_pause};
use crate::signal::{pthread_self, pthread_sigqueue, siginfo_t, sigval};
use std::any::Any;
use std::sync::mpsc::Sender;

type TID = usize;

pub struct Threads {
    all_threads: RwLock<HashMap<TID, Arc<Thread>>>,
    this_thread: &'static LocalKey<RefCell<Option<Arc<Thread>>>>,
}

impl Threads {
    pub fn this_thread(&self) -> Arc<Thread> {
        self.this_thread.with(|thread| {
            thread.borrow().as_ref().unwrap().clone()
        })
    }

    thread_local! {
        static THIS_THREAD: RefCell<Option<Arc<Thread>>> = RefCell::new(None);
    }


    pub fn create_thread(&self) -> Thread {
        let mut res = Thread {
            started: AtomicBool::new(false),
            alive: AtomicBool::new(false),
            paused_mutex: Mutex::new(()),
            paused: Condvar::new(),
            unix_tid: None,
            rust_join_handle: None,
            thread_start_channel_send: None
        };
        let (thread_info_channel_send,thread_info_channel_recv) = std::sync::mpsc::channel();
        let (thread_start_channel_send, thread_start_channel_recv) = std::sync::mpsc::channel();
        let join_handle = std::thread::spawn(||{
            thread_info_channel_send.send(gettid());
            let (func, data): (fn(Box<dyn Any>), Box<dyn Any>) = thread_start_channel_recv.recv().unwrap();
            func(data)
        });
        res.thread_start_channel_send = thread_start_channel_send.into();
        res.unix_tid  = thread_info_channel_recv.recv().unwrap().into();
        res.rust_join_handle = join_handle.into();
        res
    }
}

pub struct Thread {
    started: AtomicBool,
    alive: AtomicBool,
    paused_mutex: Mutex<()>,//todo maybe use rust park() for this
    paused: Condvar,
    unix_tid: Option<Pid>,
    rust_join_handle: Option<std::thread::JoinHandle<()>>,
    thread_start_channel_send: Option<Sender<(fn(Box<dyn Any>), Box<dyn Any>)>>
}

impl Thread {
    pub fn start_thread(&self, func: fn(Box<dyn Any>), data: Box<dyn Any>) {
        self.thread_start_channel_send.as_ref().unwrap().send((func, data)).unwrap();
    }

    pub fn pause(&self) {
        assert_eq!(self.unix_tid, gettid());
        self.paused.wait(self.paused_mutex.lock().unwrap());
    }

    pub fn resume(&self) {
        self.paused.notify_one();
    }

    pub fn is_alive(&self) -> bool {
        self.alive.load(Ordering::SeqCst)
    }

    pub fn join(&self) {
        self.rust_join_handle.join().unwrap();
    }

    fn rust_thread(&self) -> &std::thread::Thread{
        self.rust_join_handle.thread()
    }
}

pub enum SignalReason {
    Pause(*const Threads),
    Event(AnEvent),
}

pub struct AnEvent {
    pub event_handler: unsafe extern fn(data: *mut c_void),
    pub data: *mut c_void,
}

impl Threads {
    fn init_signal_handler(&self) {
        unsafe {
            let sa = SigAction::new(SigHandler::SigAction(handler), transmute(0 as libc::c_int), SigSet::empty());
            sigaction(Signal::SIGUSR1, &sa).unwrap();
        };
    }

    unsafe fn trigger_signal(&self, t: &Thread, reason: SignalReason) {
        let metadata_void_ptr = Box::leak(box reason) as *mut SignalReason as *mut c_void;
        let sigval_ = sigval { sival_ptr: metadata_void_ptr };
        let tid = t.unix_tid.as_raw();

        let res = pthread_sigqueue(pthread_self(), transmute(Signal::SIGUSR1), sigval_);
        if res != 0 {
            dbg!(gettid());
            dbg!(errno());
            dbg!(res);
            panic!()
        }
    }

    pub fn await_all_threads_death(&self) {
        let all_threads_read_guard = self.all_threads.read().unwrap();
        all_threads_read_guard.values().find(|thread| { thread.is_alive() });
    }
}

extern fn handler(signal_number: libc::c_int, siginfo: *mut libc::siginfo_t, _data: *mut libc::c_void) {
    unsafe {
        assert_eq!(Signal::try_from(signal_number).unwrap(), Signal::SIGUSR1);

        let siginfo_signals_h = (siginfo as *mut siginfo_t).read();
        let signal_reason_ptr = siginfo_signals_h._sifields._rt.si_sigval.sival_ptr;
        assert_ne!(signal_reason_ptr, null_mut());
        // assert_eq!(siginfo_signals_h.si_code, SI_QUEUE);
        let reason = (signal_reason_ptr as *mut SignalReason).read();

        match reason {
            SignalReason::Pause(threads) => handle_pause(threads.as_ref().unwrap()),
            SignalReason::Event(e) => handle_event(e)
        }
    }
}

pub mod handlers {
    use crate::{AnEvent, Threads};

    pub fn handle_pause(threads: &Threads) {
        let this = threads.this_thread();
        this.pause();
    }

    pub fn handle_event(e: AnEvent) {
        let AnEvent { event_handler, data } = e;
        event_handler(data);
    }
}


pub mod signal;
pub mod context;