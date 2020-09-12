use std::ffi::CString;
use std::sync::Arc;

use jvmti_jni_bindings::*;

use crate::{InterpreterStateGuard, JVMState, SuspendedStatus};
use crate::interpreter::suspend_check;
use crate::java_values::JavaValue;
use crate::jvmti::{get_interpreter_state, get_state};
use crate::rust_jni::interface::local_frame::new_local_ref_public;
use crate::rust_jni::native_util::{from_object, to_object};

#[macro_export]
macro_rules! get_thread_or_error {
    ($raw_thread: expr) => {
    match crate::JavaValue::Object(from_object($raw_thread)).try_cast_thread() {
        None => return jvmti_jni_bindings::jvmtiError_JVMTI_ERROR_INVALID_THREAD,
        Some(jt) => jt
    }
    };
}

///Get Top Thread Groups
///
///     jvmtiError
///     GetTopThreadGroups(jvmtiEnv* env,
///                 jint* group_count_ptr,
///                 jthreadGroup** groups_ptr)
///
/// Return all top-level (parentless) thread groups in the VM.
///
/// Phase	Callback Safe	Position	Since
/// may only be called during the live phase 	No 	13	1.0
///
/// Capabilities
/// Required Functionality
///
/// Parameters
/// Name 	Type 	Description
/// group_count_ptr	jint*	On return, points to the number of top-level thread groups.
///
/// Agent passes a pointer to a jint. On return, the jint has been set.
/// groups_ptr	jthreadGroup**	On return, refers to a pointer to the top-level thread group array.
///
/// Agent passes a pointer to a jthreadGroup*. On return, the jthreadGroup* points to a newly allocated array of size *group_count_ptr. The array should be freed with Deallocate. The objects returned by groups_ptr are JNI local references and must be managed.
///
/// Errors
/// This function returns either a universal error or one of the following errors
/// Error 	Description
/// JVMTI_ERROR_NULL_POINTER	group_count_ptr is NULL.
/// JVMTI_ERROR_NULL_POINTER	groups_ptr is NULL.
pub unsafe extern "C" fn get_top_thread_groups(env: *mut jvmtiEnv, group_count_ptr: *mut jint, groups_ptr: *mut *mut jthreadGroup) -> jvmtiError {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let tracing_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "GetTopThreadGroups");
    null_check!(group_count_ptr);
    null_check!(groups_ptr);
    assert!(jvm.vm_live());
    //There is only one top level thread group in this JVM.
    group_count_ptr.write(1);
    let system_j_thread_group = jvm.thread_state.get_system_thread_group();
    let thread_group_object = system_j_thread_group.object();
    let res = new_local_ref_public(thread_group_object.into(), int_state);

    jvm.native_interface_allocations.allocate_and_write_vec(vec![res], group_count_ptr, groups_ptr);
    jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}

///Get All Threads
///
///     jvmtiError
///     GetAllThreads(jvmtiEnv* env,
///                 jint* threads_count_ptr,
///                 jthread** threads_ptr)
///
/// Get all live threads. The threads are Java programming language threads; that is, threads that are attached to the VM.
/// A thread is live if java.lang.Thread.isAlive() would return true, that is, the thread has been started and has not yet died.
/// The universe of threads is determined by the context of the JVM TI environment, which typically is all threads attached to the VM.
/// Note that this includes JVM TI agent threads (see RunAgentThread).
///
/// Phase	Callback Safe	Position	Since
/// may only be called during the live phase 	No 	4	1.0
///
/// Capabilities
/// Required Functionality
///
/// Parameters
/// Name 	Type 	Description
/// threads_count_ptr	jint*	On return, points to the number of running threads.
///
/// Agent passes a pointer to a jint. On return, the jint has been set.
/// threads_ptr	jthread**	On return, points to an array of references, one for each running thread.
///
/// Agent passes a pointer to a jthread*. On return, the jthread* points to a newly allocated array of size *threads_count_ptr.
/// The array should be freed with Deallocate. The objects returned by threads_ptr are JNI local references and must be managed.
///
/// Errors
/// This function returns either a universal error or one of the following errors
/// Error 	Description
/// JVMTI_ERROR_NULL_POINTER	threads_count_ptr is NULL.
/// JVMTI_ERROR_NULL_POINTER	threads_ptr is NULL.
pub unsafe extern "C" fn get_all_threads(env: *mut jvmtiEnv, threads_count_ptr: *mut jint, threads_ptr: *mut *mut jthread) -> jvmtiError {
    let jvm = get_state(env);
    let tracing_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "GetAllThreads");
    null_check!(threads_count_ptr);
    null_check!(threads_ptr);
    assert!(jvm.vm_live());
    let res_ptrs = jvm.thread_state.get_all_threads().values().filter(|thread| {
        let int_state = get_interpreter_state(env);
        thread.thread_object().is_alive(jvm, int_state) != 0
    }).map(|thread| {
        let int_state = get_interpreter_state(env);
        new_local_ref_public(thread.thread_object().object().into(), int_state)
    }).collect::<Vec<jobject>>();
    jvm.native_interface_allocations.allocate_and_write_vec(res_ptrs, threads_count_ptr, threads_ptr);
    jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}

///Get Thread Info
///
///     typedef struct {
///         char* name;
///         jint priority;
///         jboolean is_daemon;
///         jthreadGroup thread_group;
///         jobject context_class_loader;
///     } jvmtiThreadInfo;
///
///     jvmtiError
///     GetThreadInfo(jvmtiEnv* env,
///                 jthread thread,
///                 jvmtiThreadInfo* info_ptr)
///
/// Get thread information. The fields of the jvmtiThreadInfo structure are filled in with details of the specified thread.
///
/// Phase	Callback Safe	Position	Since
/// may only be called during the live phase 	No 	9	1.0
///
/// Capabilities
/// Required Functionality
///
/// jvmtiThreadInfo - Thread information structure
/// Field 	Type 	Description
/// name	char*	The thread name, encoded as a modified UTF-8 string.
/// priority	jint	The thread priority. See the thread priority constants: jvmtiThreadPriority.
/// is_daemon	jboolean	Is this a daemon thread?
/// thread_group	jthreadGroup	The thread group to which this thread belongs. NULL if the thread has died.
/// context_class_loader	jobject	The context class loader associated with this thread.
///
/// Parameters
/// Name 	Type 	Description
/// thread	jthread	The thread to query. If thread is NULL, the current thread is used.
/// info_ptr	jvmtiThreadInfo*	On return, filled with information describing the specified thread.
///
/// For JDK 1.1 implementations that don't recognize context class loaders, the context_class_loader field will be NULL.
///
/// Agent passes a pointer to a jvmtiThreadInfo. On return, the jvmtiThreadInfo has been set.
/// The pointer returned in the field name of jvmtiThreadInfo is a newly allocated array.
/// The array should be freed with Deallocate.
/// The object returned in the field thread_group of jvmtiThreadInfo is a JNI local reference and must be managed.
/// The object returned in the field context_class_loader of jvmtiThreadInfo is a JNI local reference and must be managed.
///
/// Errors
/// This function returns either a universal error or one of the following errors
/// Error 	Description
/// JVMTI_ERROR_INVALID_THREAD	thread is not a thread object.
/// JVMTI_ERROR_NULL_POINTER	info_ptr is NULL.
///
pub unsafe extern "C" fn get_thread_info(env: *mut jvmtiEnv, thread: jthread, info_ptr: *mut jvmtiThreadInfo) -> jvmtiError {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    null_check!(info_ptr);
    assert!(jvm.vm_live());
    let tracing_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "GetThreadInfo");
    let thread_object = match JavaValue::Object(from_object(thread)).try_cast_thread() {
        None => return jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_INVALID_THREAD),
        Some(thread) => thread,
    };

    //todo get thread groups other than system thread group working at some point
    (*info_ptr).thread_group = new_local_ref_public(jvm.thread_state.get_system_thread_group().object().into(), int_state);
    //todo deal with this whole context loader situation
    let thread_class_object = thread_object
        .get_class(jvm, int_state);
    let class_loader = thread_class_object
        .get_class_loader(jvm, int_state);
    // .expect("Expected thread class to have a class loader");
    let context_class_loader = new_local_ref_public(class_loader.map(|x| x.object()), int_state);
    (*info_ptr).context_class_loader = context_class_loader;
    (*info_ptr).name = jvm.native_interface_allocations.allocate_cstring(CString::new(thread_object.name().to_rust_string()).unwrap());
    (*info_ptr).is_daemon = thread_object.daemon() as u8;
    (*info_ptr).priority = thread_object.priority();
    jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}

/// Get Thread State
///
///     jvmtiError
///     GetThreadState(jvmtiEnv* env,
///                 jthread thread,
///                 jint* thread_state_ptr)
///
/// Get the state of a thread. The state of the thread is represented by the answers to the hierarchical set of questions below:
///
///     Alive?
///         Not alive.
///             Why not alive?
///                 New.
///                 Terminated (JVMTI_THREAD_STATE_TERMINATED)
///         Alive (JVMTI_THREAD_STATE_ALIVE)
///             Suspended?
///                 Suspended (JVMTI_THREAD_STATE_SUSPENDED)
///                 Not suspended
///             Interrupted?
///                 Interrupted (JVMTI_THREAD_STATE_INTERRUPTED)
///                 Not interrupted.
///             In native?
///                 In native code (JVMTI_THREAD_STATE_IN_NATIVE)
///                 In Java programming language code
///             What alive state?
///                 Runnable (JVMTI_THREAD_STATE_RUNNABLE)
///                 Blocked (JVMTI_THREAD_STATE_BLOCKED_ON_MONITOR_ENTER)
///                 Waiting (JVMTI_THREAD_STATE_WAITING)
///                     Timed wait?
///                         Indefinite (JVMTI_THREAD_STATE_WAITING_INDEFINITELY
///                         Timed (JVMTI_THREAD_STATE_WAITING_WITH_TIMEOUT)
///                     Why waiting?
///                         Object.wait (JVMTI_THREAD_STATE_IN_OBJECT_WAIT)
///                         LockSupport.park (JVMTI_THREAD_STATE_PARKED)
///                         Sleeping (JVMTI_THREAD_STATE_SLEEPING)
///
/// The answers are represented by the following bit vector.
///
///     Thread State Flags
///     Constant 	Value 	Description
///     JVMTI_THREAD_STATE_ALIVE	0x0001	Thread is alive. Zero if thread is new (not started) or terminated.
///     JVMTI_THREAD_STATE_TERMINATED	0x0002	Thread has completed execution.
///     JVMTI_THREAD_STATE_RUNNABLE	0x0004	Thread is runnable.
///     JVMTI_THREAD_STATE_BLOCKED_ON_MONITOR_ENTER	0x0400	Thread is waiting to enter a synchronization block/method or, after an Object.wait(), waiting to re-enter a synchronization block/method.
///     JVMTI_THREAD_STATE_WAITING	0x0080	Thread is waiting.
///     JVMTI_THREAD_STATE_WAITING_INDEFINITELY	0x0010	Thread is waiting without a timeout. For example, Object.wait().
///     JVMTI_THREAD_STATE_WAITING_WITH_TIMEOUT	0x0020	Thread is waiting with a maximum time to wait specified. For example, Object.wait(long).
///     JVMTI_THREAD_STATE_SLEEPING	0x0040	Thread is sleeping -- Thread.sleep(long).
///     JVMTI_THREAD_STATE_IN_OBJECT_WAIT	0x0100	Thread is waiting on an object monitor -- Object.wait.
///     JVMTI_THREAD_STATE_PARKED	0x0200	Thread is parked, for example: LockSupport.park, LockSupport.parkUtil and LockSupport.parkNanos.
///     JVMTI_THREAD_STATE_SUSPENDED	0x100000	Thread suspended. java.lang.Thread.suspend() or a JVM TI suspend function (such as SuspendThread) has been called on the thread. If this bit is set, the other bits refer to the thread state before suspension.
///     JVMTI_THREAD_STATE_INTERRUPTED	0x200000	Thread has been interrupted.
///     JVMTI_THREAD_STATE_IN_NATIVE	0x400000	Thread is in native code--that is, a native method is running which has not called back into the VM or Java programming language code.
///
///     This flag is not set when running VM compiled Java programming language code nor is it set when running VM code or VM support code.
///     Native VM interface functions, such as JNI and JVM TI functions, may be implemented as VM code.
///     JVMTI_THREAD_STATE_VENDOR_1	0x10000000	Defined by VM vendor.
///     JVMTI_THREAD_STATE_VENDOR_2	0x20000000	Defined by VM vendor.
///     JVMTI_THREAD_STATE_VENDOR_3	0x40000000	Defined by VM vendor.
///
/// The following definitions are used to convert JVM TI thread state to java.lang.Thread.State style states.
///
///     java.lang.Thread.State Conversion Masks
///     Constant 	Value 	Description
///     JVMTI_JAVA_LANG_THREAD_STATE_MASK	JVMTI_THREAD_STATE_TERMINATED | JVMTI_THREAD_STATE_ALIVE | JVMTI_THREAD_STATE_RUNNABLE | JVMTI_THREAD_STATE_BLOCKED_ON_MONITOR_ENTER | JVMTI_THREAD_STATE_WAITING | JVMTI_THREAD_STATE_WAITING_INDEFINITELY | JVMTI_THREAD_STATE_WAITING_WITH_TIMEOUT	Mask the state with this before comparison
///     JVMTI_JAVA_LANG_THREAD_STATE_NEW	0	java.lang.Thread.State.NEW
///     JVMTI_JAVA_LANG_THREAD_STATE_TERMINATED	JVMTI_THREAD_STATE_TERMINATED	java.lang.Thread.State.TERMINATED
///     JVMTI_JAVA_LANG_THREAD_STATE_RUNNABLE	JVMTI_THREAD_STATE_ALIVE | JVMTI_THREAD_STATE_RUNNABLE	java.lang.Thread.State.RUNNABLE
///     JVMTI_JAVA_LANG_THREAD_STATE_BLOCKED	JVMTI_THREAD_STATE_ALIVE | JVMTI_THREAD_STATE_BLOCKED_ON_MONITOR_ENTER	java.lang.Thread.State.BLOCKED
///     JVMTI_JAVA_LANG_THREAD_STATE_WAITING	JVMTI_THREAD_STATE_ALIVE | JVMTI_THREAD_STATE_WAITING | JVMTI_THREAD_STATE_WAITING_INDEFINITELY	java.lang.Thread.State.WAITING
///     JVMTI_JAVA_LANG_THREAD_STATE_TIMED_WAITING	JVMTI_THREAD_STATE_ALIVE | JVMTI_THREAD_STATE_WAITING | JVMTI_THREAD_STATE_WAITING_WITH_TIMEOUT	java.lang.Thread.State.TIMED_WAITING
///
/// Rules
///
/// There can be no more than one answer to a question, although there can be no answer (because the answer is unknown, does not apply, or none of the answers is correct).
/// An answer is set only when the enclosing answers match. That is, no more than one of
///
///     JVMTI_THREAD_STATE_RUNNABLE
///     JVMTI_THREAD_STATE_BLOCKED_ON_MONITOR_ENTER
///     JVMTI_THREAD_STATE_WAITING
///
/// can be set (a J2SETM compliant implementation will always set one of these if JVMTI_THREAD_STATE_ALIVE is set).
/// And if any of these are set, the enclosing answer JVMTI_THREAD_STATE_ALIVE is set.
/// No more than one of
///
///     JVMTI_THREAD_STATE_WAITING_INDEFINITELY
///     JVMTI_THREAD_STATE_WAITING_WITH_TIMEOUT
///
/// can be set (a J2SETM compliant implementation will always set one of these if JVMTI_THREAD_STATE_WAITING is set).
/// And if either is set, the enclosing answers JVMTI_THREAD_STATE_ALIVE and JVMTI_THREAD_STATE_WAITING are set.
/// No more than one of
///
///     JVMTI_THREAD_STATE_IN_OBJECT_WAIT
///     JVMTI_THREAD_STATE_PARKED
///     JVMTI_THREAD_STATE_SLEEPING
///
/// can be set. And if any of these is set, the enclosing answers JVMTI_THREAD_STATE_ALIVE and JVMTI_THREAD_STATE_WAITING are set.
/// Also, if JVMTI_THREAD_STATE_SLEEPING is set, then JVMTI_THREAD_STATE_WAITING_WITH_TIMEOUT is set.
/// If a state A is implemented using the mechanism of state B then it is state A which is returned by this function.
/// For example, if Thread.sleep(long) is implemented using Object.wait(long) then it is still JVMTI_THREAD_STATE_SLEEPING which is returned.
/// More than one of
///
///     JVMTI_THREAD_STATE_SUSPENDED
///     JVMTI_THREAD_STATE_INTERRUPTED
///     JVMTI_THREAD_STATE_IN_NATIVE
///
/// can be set, but if any is set, JVMTI_THREAD_STATE_ALIVE is set.
///
/// And finally, JVMTI_THREAD_STATE_TERMINATED cannot be set unless JVMTI_THREAD_STATE_ALIVE is not set.
///
/// The thread state representation is designed for extension in future versions of the specification; thread state values should be used accordingly, that is they should not be used as ordinals.
/// Most queries can be made by testing a single bit, if use in a switch statement is desired, the state bits should be masked with the interesting bits.
/// All bits not defined above are reserved for future use.
/// A VM, compliant to the current specification, must set reserved bits to zero.
/// An agent should ignore reserved bits -- they should not be assumed to be zero and thus should not be included in comparisons.
///
/// Examples
///
/// Note that the values below exclude reserved and vendor bits.
///
/// The state of a thread blocked at a synchronized-statement would be:
///
///                 JVMTI_THREAD_STATE_ALIVE + JVMTI_THREAD_STATE_BLOCKED_ON_MONITOR_ENTER
///
///
/// The state of a thread which hasn't started yet would be:
///
///                 0
///
///
/// The state of a thread at a Object.wait(3000) would be:
///
///                 JVMTI_THREAD_STATE_ALIVE + JVMTI_THREAD_STATE_WAITING +
///                     JVMTI_THREAD_STATE_WAITING_WITH_TIMEOUT +
///                     JVMTI_THREAD_STATE_MONITOR_WAITING
///
///
/// The state of a thread suspended while runnable would be:
///
///                 JVMTI_THREAD_STATE_ALIVE + JVMTI_THREAD_STATE_RUNNABLE + JVMTI_THREAD_STATE_SUSPENDED
///
///
/// Testing the State
///
/// In most cases, the thread state can be determined by testing the one bit corresponding to that question. For example, the code to test if a thread is sleeping:
///
///     	jint state;
///     	jvmtiError err;
///
///     	err = (*jvmti)->GetThreadState(jvmti, thread, &state);
///     	if (err == JVMTI_ERROR_NONE) {
///     	   if (state & JVMTI_THREAD_STATE_SLEEPING) {  ...
///
///
/// For waiting (that is, in Object.wait, parked, or sleeping) it would be:
///
///     	   if (state & JVMTI_THREAD_STATE_WAITING) {  ...
///
///
/// For some states, more than one bit will need to be tested as is the case when testing if a thread has not yet been started:
///
///     	   if ((state & (JVMTI_THREAD_STATE_ALIVE | JVMTI_THREAD_STATE_TERMINATED)) == 0)  {  ...
///
///
/// To distinguish timed from untimed Object.wait:
///
///     	   if (state & JVMTI_THREAD_STATE_IN_OBJECT_WAIT)  {
///                  if (state & JVMTI_THREAD_STATE_WAITING_WITH_TIMEOUT)  {
///                    printf("in Object.wait(long timeout)\n");
///                  } else {
///                    printf("in Object.wait()\n");
///                  }
///                }
///
///
/// Relationship to java.lang.Thread.State
///
/// The thread state represented by java.lang.Thread.State returned from java.lang.Thread.getState() is a subset of the information returned from this function.
/// The corresponding java.lang.Thread.State can be determined by using the provided conversion masks.
/// For example, this returns the name of the java.lang.Thread.State thread state:
///
///     	    err = (*jvmti)->GetThreadState(jvmti, thread, &state);
///     	    abortOnError(err);
///                 switch (state & JVMTI_JAVA_LANG_THREAD_STATE_MASK) {
///                 case JVMTI_JAVA_LANG_THREAD_STATE_NEW:
///                   return "NEW";
///                 case JVMTI_JAVA_LANG_THREAD_STATE_TERMINATED:
///                   return "TERMINATED";
///                 case JVMTI_JAVA_LANG_THREAD_STATE_RUNNABLE:
///                   return "RUNNABLE";
///                 case JVMTI_JAVA_LANG_THREAD_STATE_BLOCKED:
///                   return "BLOCKED";
///                 case JVMTI_JAVA_LANG_THREAD_STATE_WAITING:
///                   return "WAITING";
///                 case JVMTI_JAVA_LANG_THREAD_STATE_TIMED_WAITING:
///                   return "TIMED_WAITING";
///                 }
///
///
/// Phase	Callback Safe	Position	Since
/// may only be called during the live phase 	No 	17	1.0
///
/// Capabilities
/// Required Functionality
///
/// Parameters
/// Name 	Type 	Description
/// thread	jthread	The thread to query. If thread is NULL, the current thread is used.
/// thread_state_ptr	jint*	On return, points to state flags, as defined by the Thread State Flags.
///
/// Agent passes a pointer to a jint. On return, the jint has been set.
///
/// Errors
/// This function returns either a universal error or one of the following errors
/// Error 	Description
/// JVMTI_ERROR_INVALID_THREAD	thread is not a thread object.
/// JVMTI_ERROR_NULL_POINTER	thread_state_ptr is NULL.
///
pub unsafe extern "C" fn get_thread_state(env: *mut jvmtiEnv, thread: jthread, thread_state_ptr: *mut jint) -> jvmtiError {
    let jvm = get_state(env);
    let tracing_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "GetThreadState");
    null_check!(thread_state_ptr);
    assert!(jvm.vm_live());
    let jthread = match JavaValue::Object(from_object(thread)).try_cast_thread() {
        None => return jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_INVALID_THREAD),
        Some(thread) => thread,
    };
    let thread = jthread.get_java_thread(jvm);
    let state = thread.status_number();
    thread_state_ptr.write(state);
    jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}


///Suspend Thread List
///
///     jvmtiError
///     SuspendThreadList(jvmtiEnv* env,
///                 jint request_count,
///                 const jthread* request_list,
///                 jvmtiError* results)
///
/// Suspend the request_count threads specified in the request_list array. Threads may be resumed with ResumeThreadList or ResumeThread.
/// If the calling thread is specified in the request_list array, this function will not return until some other thread resumes it.
/// Errors encountered in the suspension of a thread are returned in the results array, not in the return value of this function.
/// Threads that are currently suspended do not change state.
///
/// Phase	Callback Safe	Position	Since
/// may only be called during the live phase 	No 	92	1.0
///
/// Capabilities
/// Optional Functionality: might not be implemented for all virtual machines. The following capability (as returned by GetCapabilities) must be true to use this function.
/// Capability 	Effect
/// can_suspend	Can suspend and resume threads
///
/// Parameters
/// Name 	Type 	Description
/// request_count	jint	The number of threads to suspend.
/// request_list	const jthread*	The list of threads to suspend.
///
/// Agent passes in an array of request_count elements of jthread.
/// results	jvmtiError*	An agent supplied array of request_count elements. On return, filled with the error code for the suspend of the corresponding thread.
/// The error code will be JVMTI_ERROR_NONE if the thread was suspended by this call. Possible error codes are those specified for SuspendThread.
///
/// Agent passes an array large enough to hold request_count elements of jvmtiError.
/// The incoming values of the elements of the array are ignored. On return, the elements are set.
///
/// Errors
/// This function returns either a universal error or one of the following errors
/// Error 	Description
/// JVMTI_ERROR_MUST_POSSESS_CAPABILITY 	The environment does not possess the capability can_suspend. Use AddCapabilities.
/// JVMTI_ERROR_ILLEGAL_ARGUMENT	request_count is less than 0.
/// JVMTI_ERROR_NULL_POINTER	request_list is NULL.
/// JVMTI_ERROR_NULL_POINTER	results is NULL.
pub unsafe extern "C" fn suspend_thread_list(env: *mut jvmtiEnv, request_count: jint, request_list: *const jthread, results: *mut jvmtiError) -> jvmtiError {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let tracing_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "SuspendThreadList");
    null_check!(request_list);
    null_check!(results);
    assert!(jvm.vm_live());
    if request_count < 0 {
        return jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_ILLEGAL_ARGUMENT);
    }
    //todo handle checking capabilities
    for i in 0..request_count {
        let thread_object_raw = request_list.offset(i as isize).read();
        let suspend_res = suspend_thread_impl(thread_object_raw, jvm, int_state);
        results.offset(i as isize).write(suspend_res);
    }
    jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}

unsafe fn suspend_thread_impl(thread_object_raw: jthread, jvm: &'static JVMState, int_state: &mut InterpreterStateGuard) -> jvmtiError {
    let jthread = get_thread_or_error!(thread_object_raw);
    let java_thread = jthread.get_java_thread(jvm);
    let SuspendedStatus { suspended, suspend_condvar } = &java_thread.suspended;
    let mut suspended_guard = suspended.lock().unwrap();
    let res = if *suspended_guard {
        jvmtiError_JVMTI_ERROR_THREAD_SUSPENDED
    } else {
        *suspended_guard = true;
        jvmtiError_JVMTI_ERROR_NONE
    };
    if Arc::ptr_eq(&java_thread, int_state.thread) {
        assert_eq!(java_thread.java_tid, int_state.thread.java_tid);
        suspend_check(int_state);
    }
    if !java_thread.is_alive() {
        jvmtiError_JVMTI_ERROR_THREAD_NOT_ALIVE
    } else {
        res
    }
}

pub unsafe extern "C" fn interrupt_thread(env: *mut jvmtiEnv, thread: jthread) -> jvmtiError {
    let jvm = get_state(env);
    let tracing_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "SuspendThread");
    unimplemented!();
    // jvm.tracing.trace_jdwp_function_exit(tracing_guard, suspend_thread(env, thread))//todo this is an ugly hack.
}

///Suspend Thread
///
///     jvmtiError
///     SuspendThread(jvmtiEnv* env,
///                 jthread thread)
///
/// Suspend the specified thread.
/// If the calling thread is specified, this function will not return until some other thread calls ResumeThread.
/// If the thread is currently suspended, this function does nothing and returns an error.
///
/// Phase	Callback Safe	Position	Since
/// may only be called during the live phase 	No 	5	1.0
///
/// Capabilities
/// Optional Functionality: might not be implemented for all virtual machines. The following capability (as returned by GetCapabilities) must be true to use this function.
/// Capability 	Effect
/// can_suspend	Can suspend and resume threads
///
/// Parameters
/// Name 	Type 	Description
/// thread	jthread	The thread to suspend. If thread is NULL, the current thread is used.
///
/// Errors
/// This function returns either a universal error or one of the following errors
/// Error 	Description
/// JVMTI_ERROR_MUST_POSSESS_CAPABILITY 	The environment does not possess the capability can_suspend. Use AddCapabilities.
/// JVMTI_ERROR_THREAD_SUSPENDED	Thread already suspended.
/// JVMTI_ERROR_INVALID_THREAD	thread is not a thread object.
/// JVMTI_ERROR_THREAD_NOT_ALIVE	thread is not live (has not been started or is now dead).
pub unsafe extern "C" fn suspend_thread(env: *mut jvmtiEnv, thread: jthread) -> jvmtiError {
    //todo check capabilities
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let tracing_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "SuspendThread");
    let res = suspend_thread_impl(thread, jvm, int_state);
    jvm.tracing.trace_jdwp_function_exit(tracing_guard, res)
}

///Resume Thread
///
///     jvmtiError
///     ResumeThread(jvmtiEnv* env,
///                 jthread thread)
///
/// Resume a suspended thread. Any threads currently suspended through a JVM TI suspend function (eg. SuspendThread) or java.lang.Thread.suspend() will resume execution; all other threads are unaffected.
///
/// Phase	Callback Safe	Position	Since
/// may only be called during the live phase 	No 	6	1.0
///
/// Capabilities
/// Optional Functionality: might not be implemented for all virtual machines. The following capability (as returned by GetCapabilities) must be true to use this function.
/// Capability 	Effect
/// can_suspend	Can suspend and resume threads
///
/// Parameters
/// Name 	Type 	Description
/// thread	jthread	The thread to resume.
///
/// Errors
/// This function returns either a universal error or one of the following errors
/// Error 	Description
/// JVMTI_ERROR_MUST_POSSESS_CAPABILITY 	The environment does not possess the capability can_suspend. Use AddCapabilities.
/// JVMTI_ERROR_THREAD_NOT_SUSPENDED	Thread was not suspended.
/// JVMTI_ERROR_INVALID_TYPESTATE	The state of the thread has been modified, and is now inconsistent.
/// JVMTI_ERROR_INVALID_THREAD	thread is not a thread object.
/// JVMTI_ERROR_THREAD_NOT_ALIVE	thread is not live (has not been started or is now dead).
pub unsafe extern "C" fn resume_thread(env: *mut jvmtiEnv, thread: jthread) -> jvmtiError {
    let jvm = get_state(env);
    //todo handle capabilities for this
    assert!(jvm.vm_live());
    let tracing_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "ResumeThread");
    let res = resume_thread_impl(jvm, thread);
    jvm.tracing.trace_jdwp_function_exit(tracing_guard, res)
}

/// Resume Thread List
///
/// jvmtiError
/// ResumeThreadList(jvmtiEnv* env,
/// jint request_count,
/// const jthread* request_list,
/// jvmtiError* results)
///
/// Resume the request_count threads specified in the request_list array. Any thread suspended through a JVM TI suspend function (eg. SuspendThreadList) or java.lang.Thread.suspend() will resume execution.
///
/// Phase	Callback Safe	Position	Since
/// may only be called during the live phase 	No 	93	1.0
///
/// Capabilities
/// Optional Functionality: might not be implemented for all virtual machines. The following capability (as returned by GetCapabilities) must be true to use this function.
/// Capability 	Effect
/// can_suspend	Can suspend and resume threads
///
/// Parameters
/// Name 	Type 	Description
/// request_count	jint	The number of threads to resume.
/// request_list	const jthread*	The threads to resume.
///
/// Agent passes in an array of request_count elements of jthread.
/// results	jvmtiError*	An agent supplied array of request_count elements.
/// On return, filled with the error code for the resume of the corresponding thread.
/// The error code will be JVMTI_ERROR_NONE if the thread was suspended by this call. Possible error codes are those specified for ResumeThread.
///
/// Agent passes an array large enough to hold request_count elements of jvmtiError. The incoming values of the elements of the array are ignored. On return, the elements are set.
///
/// Errors
/// This function returns either a universal error or one of the following errors
/// Error 	Description
/// JVMTI_ERROR_MUST_POSSESS_CAPABILITY 	The environment does not possess the capability can_suspend. Use AddCapabilities.
/// JVMTI_ERROR_ILLEGAL_ARGUMENT	request_count is less than 0.
/// JVMTI_ERROR_NULL_POINTER	request_list is NULL.
/// JVMTI_ERROR_NULL_POINTER	results is NULL.
pub unsafe extern "C" fn resume_thread_list(env: *mut jvmtiEnv, request_count: jint, request_list: *const jthread, results: *mut jvmtiError) -> jvmtiError {
    let jvm = get_state(env);
    let tracing_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "ResumeThreadList");
    assert!(jvm.vm_live());
    null_check!(request_list);
    null_check!(results);
    if request_count < 0 {
        return jvmtiError_JVMTI_ERROR_ILLEGAL_ARGUMENT;
    }
    //todo handle capabilities;
    for i in 0..request_count {
        let jthreadp = request_list.offset(i as isize).read();
        results.offset(i as isize).write(resume_thread_impl(jvm, jthreadp));
    }
    jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}


unsafe fn resume_thread_impl(jvm: &'static JVMState, thread_raw: jthread) -> jvmtiError {
    let thread_object_raw = from_object(thread_raw);
    let jthread = match JavaValue::Object(thread_object_raw).try_cast_thread() {
        None => {
            return jvmtiError_JVMTI_ERROR_INVALID_THREAD;
        }
        Some(jthread) => jthread,
    };
    let java_thread = jthread.get_java_thread(jvm);
    let SuspendedStatus { suspended, suspend_condvar } = &java_thread.suspended;
    let mut suspend_guard = suspended.lock().unwrap();
    if !*suspend_guard {
        jvmtiError_JVMTI_ERROR_THREAD_NOT_SUSPENDED
    } else {
        *suspend_guard = false;
        suspend_condvar.notify_one();//notify one and notify all should be the same here
        jvmtiError_JVMTI_ERROR_NONE
    }
}

///Get Thread Group Info
///
///     typedef struct {
///         jthreadGroup parent;
///         char* name;
///         jint max_priority;
///         jboolean is_daemon;
///     } jvmtiThreadGroupInfo;
///
///     jvmtiError
///     GetThreadGroupInfo(jvmtiEnv* env,
///                 jthreadGroup group,
///                 jvmtiThreadGroupInfo* info_ptr)
///
/// Get information about the thread group. The fields of the jvmtiThreadGroupInfo structure are filled in with details of the specified thread group.
///
/// Phase	Callback Safe	Position	Since
/// may only be called during the live phase 	No 	14	1.0
///
/// Capabilities
/// Required Functionality
///
/// jvmtiThreadGroupInfo - Thread group information structure
/// Field 	Type 	Description
/// parent	jthreadGroup	The parent thread group.
/// name	char*	The thread group's name, encoded as a modified UTF-8 string.
/// max_priority	jint	The maximum priority for this thread group.
/// is_daemon	jboolean	Is this a daemon thread group?
///
/// Parameters
/// Name 	Type 	Description
/// group	jthreadGroup	The thread group to query.
/// info_ptr	jvmtiThreadGroupInfo*	On return, filled with information describing the specified thread group.
///
/// Agent passes a pointer to a jvmtiThreadGroupInfo. On return, the jvmtiThreadGroupInfo has been set.
/// The object returned in the field parent of jvmtiThreadGroupInfo is a JNI local reference and must be managed.
/// The pointer returned in the field name of jvmtiThreadGroupInfo is a newly allocated array.
/// The array should be freed with Deallocate.
///
/// Errors
/// This function returns either a universal error or one of the following errors
/// Error 	Description
/// JVMTI_ERROR_INVALID_THREAD_GROUP	group is not a thread group object.
/// JVMTI_ERROR_NULL_POINTER	info_ptr is NULL.
pub unsafe extern "C" fn get_thread_group_info(env: *mut jvmtiEnv, group: jthreadGroup, info_ptr: *mut jvmtiThreadGroupInfo) -> jvmtiError {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let tracing_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "GetThreadGroupInfo");
    assert!(jvm.vm_live());
    let thread_group = match JavaValue::Object(from_object(group)).try_cast_thread_group() {
        None => return jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_INVALID_THREAD_GROUP),
        Some(thread_group) => thread_group,
    };
    null_check!(info_ptr);

    let name = jvm.native_interface_allocations.allocate_cstring(CString::new(thread_group.name().to_rust_string()).unwrap());
    let info_pointer_writer = info_ptr.as_mut().unwrap();
    info_pointer_writer.name = name;
    info_pointer_writer.is_daemon = thread_group.daemon();
    info_pointer_writer.max_priority = thread_group.max_priority();
    info_pointer_writer.parent = new_local_ref_public(thread_group.parent().map(|x| x.object()), int_state);
    jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}