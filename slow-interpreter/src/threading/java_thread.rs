use std::ffi::c_void;
use std::mem::transmute;
use std::ops::Deref;
use std::ptr::{NonNull, null_mut};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::Duration;

use num_integer::Integer;

use another_jit_vm::stack::CannotAllocateStack;
use another_jit_vm_ir::ir_stack::OwnedIRStack;
use jvmti_jni_bindings::jint;
use rust_jvm_common::JavaThreadId;
use thread_signal_handler::{SignalAccessibleJavaStackData};
use thread_signal_handler::remote_queries::{GetGuestFrameStackInstructionPointer, RemoteQuery, RemoteQueryAnswer};
use threads::Thread;

use crate::{JVMState, OpaqueFrame, pushable_frame_todo, WasException};
use crate::better_java_stack::{FramePointer, JavaStack};
use crate::better_java_stack::frames::{HasFrame};
use crate::better_java_stack::java_stack_guard::JavaStackGuard;
use crate::better_java_stack::remote_frame::RemoteFrame;
use crate::interpreter::safepoint_check;
use crate::rust_jni::jvmti::ThreadJVMTIEnabledStatus;
use crate::stdlib::java::lang::thread::JThread;
use crate::threading::safepoints::SafePoint;


pub struct JavaThread<'vm> {
    pub java_tid: JavaThreadId,
    pub java_stack: Mutex<JavaStack<'vm>>,
    stack_signal_safe_data: Arc<SignalAccessibleJavaStackData>,
    pub safepoint_state: SafePoint<'vm>,
    underlying_thread: Thread<'vm>,
    pub(crate) thread_object: RwLock<Option<JThread<'vm>>>,
    pub invisible_to_java: bool,
    jvmti_events_enabled: RwLock<ThreadJVMTIEnabledStatus>,
    pub thread_local_storage: RwLock<*mut c_void>,
    pub thread_status: Mutex<ThreadStatus>,
    pub thread_status_change_condvar: Condvar
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
        self.thread_status.lock().unwrap().alive
    }

    fn new(jvm: &'gc JVMState<'gc>, thread_obj: Option<JThread<'gc>>, invisible_to_java: bool) -> Result<Arc<JavaThread<'gc>>, CannotAllocateStack> {
        let owned_ir_stack = OwnedIRStack::new()?;
        let stack_top = owned_ir_stack.native.mmaped_top.as_ptr();
        let stack_bottom = unsafe { owned_ir_stack.native.mmaped_top.as_ptr().sub(owned_ir_stack.native.max_stack) };
        let stack_signal_safe_data = Arc::new(SignalAccessibleJavaStackData::new(stack_top, stack_bottom));
        let (java_tid, name) = match thread_obj.as_ref() {
            None => (0, "Bootstrap Thread".to_string()),
            Some(thread_obj) => {
                (thread_obj.tid(jvm), thread_obj.name(jvm).to_rust_string(jvm))
            }
        };
        let underlying = jvm.thread_state.threads.create_thread(name.clone().into());
        let java_stack = Mutex::new(JavaStack::new(jvm, owned_ir_stack, stack_signal_safe_data.clone(), name.clone()));
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
            thread_status: Mutex::new(ThreadStatus { terminated: false, alive: false, interrupted: false }),
            thread_status_change_condvar: Condvar::new()
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
        let mut status = self.thread_status.lock().unwrap();
        status.alive = true;
        self.update_thread_object(jvm, status)
    }

    fn update_thread_object(&self, jvm: &'gc JVMState<'gc>, status: MutexGuard<ThreadStatus>) {
        self.thread_status_change_condvar.notify_all();
        if self.thread_object.read().unwrap().is_some() {
            let obj = self.thread_object();
            obj.set_thread_status(jvm, self.safepoint_state.get_thread_status_number(status.deref()));
            obj.notify_object_change(jvm);
        }
    }

    pub fn notify_terminated(&self, jvm: &'gc JVMState<'gc>) {
        let mut status = self.thread_status.lock().unwrap();

        status.terminated = true;
        status.alive = false;
        self.update_thread_object(jvm, status)
    }

    pub fn status_number(&self) -> jint {
        let status_guard = self.thread_status.lock().unwrap();
        self.safepoint_state.get_thread_status_number(status_guard.deref())
    }

    pub fn park<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl HasFrame<'gc>, time_nanos: Option<u128>) -> Result<(), WasException<'gc>> {
        unsafe { assert!(self.underlying_thread.is_this_thread()) }
        const NANOS_PER_SEC: u128 = 1_000_000_000u128;
        assert!(self.safepoint_state.state.lock().unwrap().waiting_monitor_notify.as_ref().is_none());
        self.safepoint_state.set_park(time_nanos.map(|time_nanos| {
            let (secs, nanos) = time_nanos.div_mod_floor(&NANOS_PER_SEC);
            Duration::new(secs as u64, nanos as u32)
        }));
        self.safepoint_state.check(jvm, int_state)
    }

    pub fn unpark<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl HasFrame<'gc>) -> Result<(), WasException<'gc>> {
        self.safepoint_state.set_unpark();
        self.safepoint_state.check(jvm, int_state)
    }

    pub unsafe fn gc_suspend(&self) {
        self.safepoint_state.set_gc_suspended().unwrap(); //todo should use gc flag for this
    }

    pub unsafe fn suspend_thread<'l>(&self, _jvm: &'gc JVMState<'gc>, _int_state: &mut impl HasFrame<'gc>, _without_self_suspend: bool) -> Result<(), SuspendError> {
        if !self.is_alive() {
            return Err(SuspendError::NotAlive);
        }
        self.safepoint_state.set_suspended()?;
        if self.underlying_thread.is_this_thread() {
            todo!();/*assert_eq!(self.java_tid, int_state.thread().java_tid);*/
            if !_without_self_suspend {
                safepoint_check(_jvm, pushable_frame_todo()/*int_state*/)?;
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

    pub fn pause_and_remote_view<T>(self: Arc<Self>, jvm: &'gc JVMState<'gc>, with_frame: impl for<'k> FnOnce(RemoteFrame<'gc,'k>) -> T) /*-> T*/ {
        let pthread_id = self.underlying_thread.pthread_id();
        let signal_safe_data = self.stack_signal_safe_data.deref();
        jvm.thread_state.interrupter.perform_remote_query(pthread_id, RemoteQuery::GetGuestFrameStackInstructionPointer { }, signal_safe_data, |answer| {
            match answer {
                RemoteQueryAnswer::GetGuestFrameStackInstructionPointer(inner) => {
                    match inner {
                        GetGuestFrameStackInstructionPointer::InGuest { rbp, .. } => {
                            let frame_pointer = FramePointer(NonNull::new(rbp as *mut c_void).unwrap());
                            let mut java_stack = JavaStackGuard::new_remote_with_frame_pointer(jvm, unsafe { transmute(&self.java_stack) }, self.clone(), frame_pointer);
                            let remote_frame = RemoteFrame::new(&mut java_stack, frame_pointer);
                            with_frame(remote_frame);
                        }
                        GetGuestFrameStackInstructionPointer::InVM { .. } => {
                            // dbg!("in vm");
                            // dbg!(rbp);
                            // dbg!(rsp);
                            // dbg!(rip);
                        }
                        GetGuestFrameStackInstructionPointer::Transitioning {} => {
                            // dbg!("transitioning");
                        }
                        GetGuestFrameStackInstructionPointer::FrameBeingCreated { .. } => {
                            // dbg!("frame being created");
                        }
                    }
                }
            }
        });
    }

    pub fn wait_thread_exit(&self) {
        loop {
            let mut thread_status_guard = self.thread_status.lock().unwrap();
            if !thread_status_guard.alive {
                break
            }
            //todo threads need fixing again
            if thread_status_guard.alive {
                //todo this is jank
                thread_status_guard = self.thread_status_change_condvar.wait_timeout(thread_status_guard, Duration::new(1,0)).unwrap().0;
            }
            drop(thread_status_guard);
        }
    }

    pub fn is_daemon(&self, jvm: &'gc JVMState<'gc>) -> bool{
        self.thread_object.read().unwrap().as_ref().unwrap().daemon(jvm)
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
