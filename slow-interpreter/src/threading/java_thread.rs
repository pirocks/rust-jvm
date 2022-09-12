use std::ffi::c_void;
use std::ops::Deref;
use std::ptr::null_mut;
use std::sync::{Arc, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::Duration;

use num_integer::Integer;

use another_jit_vm::stack::CannotAllocateStack;
use another_jit_vm_ir::ir_stack::OwnedIRStack;
use jvmti_jni_bindings::jint;
use rust_jvm_common::JavaThreadId;
use threads::Thread;

use crate::{JVMState, OpaqueFrame, pushable_frame_todo, WasException};
use crate::better_java_stack::frames::HasJavaStack;
use crate::better_java_stack::java_stack_guard::JavaStackGuard;
use crate::better_java_stack::JavaStack;
use crate::better_java_stack::remote_frame::RemoteFrame;
use crate::better_java_stack::thread_remote_read_mechanism::SignalAccessibleJavaStackData;
use crate::interpreter::safepoint_check;
use crate::rust_jni::jvmti_interface::event_callbacks::ThreadJVMTIEnabledStatus;
use crate::stdlib::java::lang::thread::JThread;
use crate::threading::safepoints::SafePoint;

pub struct JavaThread<'vm> {
    pub java_tid: JavaThreadId,
    java_stack: Mutex<JavaStack<'vm>>,
    stack_signal_safe_data: Arc<SignalAccessibleJavaStackData>,
    pub safepoint_state: SafePoint<'vm>,
    underlying_thread: Thread<'vm>,
    pub(crate) thread_object: RwLock<Option<JThread<'vm>>>,
    pub invisible_to_java: bool,
    jvmti_events_enabled: RwLock<ThreadJVMTIEnabledStatus>,
    pub thread_local_storage: RwLock<*mut c_void>,
    pub thread_status: RwLock<ThreadStatus>,
}

impl<'gc> JavaThread<'gc> {
    pub fn new_with_stack_on_this_thread<T: 'gc>(
        jvm: &'gc JVMState<'gc>,
        thread_obj: Option<JThread<'gc>>,
        invisible_to_java: bool,
        to_run: impl for<'l, 'k> FnOnce(Arc<JavaThread<'gc>>, &'l mut OpaqueFrame<'gc, 'k>) -> Result<T, WasException<'gc>> + 'gc,
    ) -> Result<T, CannotAllocateStack> {
        let java_thread = Self::new(jvm, thread_obj, invisible_to_java)?;
        let java_stack = &unsafe { Arc::into_raw(java_thread.clone()).as_ref() }.unwrap().java_stack;
        Ok(JavaStackGuard::new_from_empty_stack(jvm, java_thread.clone(), java_stack, move |opaque_frame| {
            jvm.thread_state.set_current_thread(java_thread.clone());
            java_thread.notify_alive(jvm);
            let res = to_run(java_thread.clone(), opaque_frame);
            java_thread.notify_terminated(jvm);
            res
        }).unwrap())
    }

    pub fn background_new_with_stack(
        jvm: &'gc JVMState<'gc>,
        thread_obj: Option<JThread<'gc>>,
        invisible_to_java: bool,
        to_run: impl for<'l, 'k> FnOnce(Arc<JavaThread<'gc>>, &'l mut OpaqueFrame<'gc, 'k>) -> Result<(), WasException<'gc>> + 'gc,
    ) -> Result<Arc<JavaThread<'gc>>, CannotAllocateStack> {
        let java_thread = Self::new(jvm, thread_obj, invisible_to_java)?;
        let java_stack = &unsafe { Arc::into_raw(java_thread.clone()).as_ref() }.unwrap().java_stack;
        //todo should run on actual thread.
        let java_thread_clone = java_thread.clone();
        java_thread_clone.get_underlying().start_thread(box move |_| {
            JavaStackGuard::new_from_empty_stack(jvm, java_thread.clone(), java_stack, move |opaque_frame| {
                jvm.thread_state.set_current_thread(java_thread.clone());
                java_thread.notify_alive(jvm);
                let res = to_run(java_thread.clone(), opaque_frame);
                java_thread.notify_terminated(jvm);
                res
            }).unwrap();
        }, box ());
        Ok(java_thread_clone)
    }

    pub fn is_alive(&self) -> bool {
        self.thread_status.read().unwrap().alive
    }

    fn new(jvm: &'gc JVMState<'gc>, thread_obj: Option<JThread<'gc>>, invisible_to_java: bool) -> Result<Arc<JavaThread<'gc>>, CannotAllocateStack> {
        let stack_signal_safe_data = Arc::new(SignalAccessibleJavaStackData::new());
        let (java_tid, name) = match thread_obj.as_ref() {
            None => (0, "Bootstrap Thread".to_string()),
            Some(thread_obj) => {
                (thread_obj.tid(jvm), thread_obj.name(jvm).to_rust_string(jvm))
            }
        };
        let underlying = jvm.thread_state.threads.create_thread(name.into());
        let java_stack = Mutex::new(JavaStack::new(OwnedIRStack::new()?, stack_signal_safe_data.clone()));
        let res = Arc::new(JavaThread {
            java_tid,
            java_stack,
            stack_signal_safe_data,
            underlying_thread: underlying,
            thread_object: RwLock::new(thread_obj),
            invisible_to_java,
            jvmti_events_enabled: RwLock::new(ThreadJVMTIEnabledStatus::default()),
            thread_local_storage: RwLock::new(null_mut()),
            safepoint_state: SafePoint::new(),
            thread_status: RwLock::new(ThreadStatus { terminated: false, alive: false, interrupted: false }),
        });
        jvm.thread_state.all_java_threads.write().unwrap().insert(res.java_tid, res.clone());
        Ok(res)
    }

    pub fn jvmti_event_status(&self) -> RwLockReadGuard<ThreadJVMTIEnabledStatus> {
        self.jvmti_events_enabled.read().unwrap()
    }

    pub fn jvmti_event_status_mut(&self) -> RwLockWriteGuard<ThreadJVMTIEnabledStatus> {
        self.jvmti_events_enabled.write().unwrap()
    }

    pub fn get_underlying(&self) -> &Thread<'gc> {
        &self.underlying_thread
    }

    pub fn thread_object(&self) -> JThread<'gc> {
        self.try_thread_object().unwrap()
    }

    pub fn try_thread_object(&self) -> Option<JThread<'gc>> {
        self.thread_object.read().unwrap().clone()
    }

    pub fn notify_alive(&self, jvm: &'gc JVMState<'gc>) {
        let mut status = self.thread_status.write().unwrap();
        status.alive = true;
        self.update_thread_object(jvm, status)
    }

    fn update_thread_object(&self, jvm: &'gc JVMState<'gc>, status: RwLockWriteGuard<ThreadStatus>) {
        if self.thread_object.read().unwrap().is_some() {
            let obj = self.thread_object();
            obj.set_thread_status(jvm, self.safepoint_state.get_thread_status_number(status.deref()))
        }
    }

    pub fn notify_terminated(&self, jvm: &'gc JVMState<'gc>) {
        let mut status = self.thread_status.write().unwrap();

        status.terminated = true;
        self.update_thread_object(jvm, status)
    }

    pub fn status_number(&self) -> jint {
        let status_guard = self.thread_status.read().unwrap();
        self.safepoint_state.get_thread_status_number(status_guard.deref())
    }

    pub fn park<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl HasJavaStack<'gc>, time_nanos: Option<u128>) -> Result<(), WasException<'gc>> {
        unsafe { assert!(self.underlying_thread.is_this_thread()) }
        const NANOS_PER_SEC: u128 = 1_000_000_000u128;
        self.safepoint_state.set_park(time_nanos.map(|time_nanos| {
            let (secs, nanos) = time_nanos.div_mod_floor(&NANOS_PER_SEC);
            Duration::new(secs as u64, nanos as u32)
        }));
        self.safepoint_state.check(jvm, int_state)
    }

    pub fn unpark<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl HasJavaStack<'gc>) -> Result<(), WasException<'gc>> {
        self.safepoint_state.set_unpark();
        self.safepoint_state.check(jvm, int_state)
    }

    pub unsafe fn gc_suspend(&self) {
        self.safepoint_state.set_gc_suspended().unwrap(); //todo should use gc flag for this
    }

    pub unsafe fn suspend_thread<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl HasJavaStack<'gc>, without_self_suspend: bool) -> Result<(), SuspendError> {
        if !self.is_alive() {
            return Err(SuspendError::NotAlive);
        }
        self.safepoint_state.set_suspended()?;
        if self.underlying_thread.is_this_thread() {
            todo!();/*assert_eq!(self.java_tid, int_state.thread().java_tid);*/
            if !without_self_suspend {
                safepoint_check(jvm, pushable_frame_todo()/*int_state*/)?;
            }
        }
        Ok(())
    }

    pub unsafe fn resume_thread(&self) -> Result<(), ResumeError> {
        self.safepoint_state.set_unsuspended()
    }

    pub unsafe fn gc_resume_thread(&self) -> Result<(), ResumeError> {
        self.safepoint_state.set_gc_unsuspended()
    }

    pub fn is_this_thread(&self) -> bool {
        unsafe { self.underlying_thread.is_this_thread() }
    }

    pub fn pause_and_remote_view<T>(&self, with_frame: impl FnOnce(&RemoteFrame) -> T) -> T {
        unsafe { self.gc_suspend(); }
        let thread_stack_guard = self.java_stack.lock().unwrap();
        let signal_safe_data = thread_stack_guard.signal_safe_data();
        let res = with_frame(todo!());
        unsafe { self.gc_resume_thread().unwrap(); }
        res
    }
}

#[derive(Debug)]
pub struct ThreadStatus {
    pub terminated: bool,
    pub alive: bool,
    pub interrupted: bool,
}

#[derive(Debug)]
pub enum SuspendError<'gc> {
    AlreadySuspended,
    NotAlive,
    WasException(WasException<'gc>),
}

#[derive(Debug)]
pub enum ResumeError {
    NotSuspended,
}

impl<'gc> From<WasException<'gc>> for SuspendError<'gc> {
    fn from(we: WasException<'gc>) -> Self {
        Self::WasException(we)
    }
}
