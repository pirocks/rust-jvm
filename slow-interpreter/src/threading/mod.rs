use std::cell::RefCell;
use std::collections::HashMap;
use std::mem::transmute;
use std::ops::Deref;
use std::os::raw::c_void;
use std::ptr::null_mut;
use std::sync::{Arc, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::sync::atomic::Ordering;
use std::sync::mpsc::{channel, Sender};
use std::thread::LocalKey;
use std::time::Duration;

use crossbeam::thread::Scope;
use libloading::Symbol;
use num_integer::Integer;
use wtf8::Wtf8Buf;
use another_jit_vm::stack::CannotAllocateStack;

use another_jit_vm_ir::ir_stack::{IRStackMut, OwnedIRStack};
use another_jit_vm_ir::WasException;
use jvmti_jni_bindings::*;
use rust_jvm_common::JavaThreadId;
use rust_jvm_common::compressed_classfile::names::{CClassName, MethodName};
use rust_jvm_common::loading::LoaderName;
use threads::{Thread, Threads};

use crate::{InterpreterStateGuard, JVMState, NewJavaValue, run_main, set_properties};
use crate::better_java_stack::JavaStack;
use crate::better_java_stack::thread_remote_read_mechanism::{sigaction_setup, SignalAccessibleJavaStackData, ThreadSignalBasedInterrupter};
use crate::class_loading::{assert_inited_or_initing_class, check_initing_or_inited_class, check_loaded_class};
use crate::interpreter::{run_function, safepoint_check};
use crate::interpreter_state::InterpreterState;
use crate::interpreter_util::new_object_full;
use crate::invoke_interface::get_invoke_interface;
use crate::java::lang::class_loader::ClassLoader;
use crate::java::lang::string::JString;
use crate::java::lang::system::System;
use crate::java::lang::thread::JThread;
use crate::java::lang::thread_group::JThreadGroup;
use crate::jit::MethodResolverImpl;
use crate::jvmti::event_callbacks::ThreadJVMTIEnabledStatus;
use crate::new_java_values::NewJavaValueHandle;
use crate::stack_entry::StackEntryPush;
use crate::threading::safepoints::{Monitor2, SafePoint};

pub struct ThreadState<'gc> {
    pub threads: Threads<'gc>,
    interrupter: ThreadSignalBasedInterrupter,
    // threads_locals: RwLock<HashMap<ThreadId, Arc<FastPerThreadData>>>,
    main_thread: RwLock<Option<Arc<JavaThread<'gc>>>>,
    pub(crate) all_java_threads: RwLock<HashMap<JavaThreadId, Arc<JavaThread<'gc>>>>,
    current_java_thread: &'static LocalKey<RefCell<Option<Arc<JavaThread<'static>>>>>,
    pub system_thread_group: RwLock<Option<JThreadGroup<'gc>>>,
    monitors: RwLock<Vec<Arc<Monitor2>>>,
    pub(crate) int_state_guard: &'static LocalKey<RefCell<Option<*mut InterpreterStateGuard<'static, 'static>>>>,
    pub(crate) int_state_guard_valid: &'static LocalKey<RefCell<bool>>,
}

thread_local! {
    static INT_STATE_GUARD: RefCell<Option<*mut InterpreterStateGuard<'static,'static>>> = RefCell::new(None);
}

thread_local! {
    static INT_STATE_GUARD_VALID: RefCell<bool> = RefCell::new(false);
}

pub struct MainThreadStartInfo {
    pub args: Vec<String>,
}

impl<'gc> ThreadState<'gc> {
    pub fn new(scope: Scope<'gc>) -> Self {
        Self {
            threads: Threads::new(scope),
            interrupter: sigaction_setup(),
            main_thread: RwLock::new(None),
            all_java_threads: RwLock::new(HashMap::new()),
            current_java_thread: &CURRENT_JAVA_THREAD,
            system_thread_group: RwLock::new(None),
            monitors: RwLock::new(vec![]),
            int_state_guard: &INT_STATE_GUARD,
            int_state_guard_valid: &INT_STATE_GUARD_VALID,
        }
    }

    pub fn setup_main_thread(&self, jvm: &'gc JVMState<'gc>, main_thread: &'gc Arc<JavaThread<'gc>>) -> Sender<MainThreadStartInfo> {
        *self.main_thread.write().unwrap() = main_thread.clone().into();
        let (main_send, main_recv) = channel();
        main_thread.clone().underlying_thread.start_thread(
            box move |_| {
                jvm.thread_state.set_current_thread(main_thread.clone());
                main_thread.thread_object.read().unwrap().as_ref().unwrap().set_priority(JVMTI_THREAD_NORM_PRIORITY as i32);
                // assert!(match main_thread.interpreter_state.read().unwrap().deref() {
                //     InterpreterState::LegacyInterpreter { call_stack, .. } => call_stack.is_empty(),
                //     InterpreterState::Jit { .. } => {}//todo!()
                // });
                let mut guard = main_thread.interpreter_state.lock().unwrap();
                let mut int_state = InterpreterStateGuard::LocalInterpreterState {
                    int_state: IRStackMut::from_stack_start(&mut guard.call_stack.inner),
                    thread: main_thread.clone(),
                    registered: false,
                    jvm,
                    current_exited_pc: None,
                    throw: None,
                }/*InterpreterStateGuard::new(jvm, main_thread.clone(), &mut main_thread.interpreter_state.lock().unwrap())*/;
                main_thread.notify_alive(jvm); //is this too early?
                let _old = int_state.register_interpreter_state_guard(jvm);
                jvm.jvmti_state().map(|jvmti| jvmti.built_in_jdwp.agent_load(jvm, &mut int_state)); // technically this is to late and should have been called earlier, but needs to be on this thread.
                ThreadState::jvm_init_from_main_thread(jvm, &mut int_state);

                assert!(!jvm.live.load(Ordering::SeqCst));
                jvm.live.store(true, Ordering::SeqCst);
                if let Some(jvmti) = jvm.jvmti_state() {
                    jvmti.built_in_jdwp.vm_inited(jvm, &mut int_state, main_thread.clone())
                }
                let MainThreadStartInfo { args } = main_recv.recv().unwrap();
                //from the jvmti spec:
                //"The thread start event for the main application thread is guaranteed not to occur until after the handler for the VM initialization event returns. "
                if let Some(jvmti) = jvm.jvmti_state() {
                    jvmti.built_in_jdwp.thread_start(jvm, &mut int_state, main_thread.thread_object())
                }
                let push_guard = int_state.push_frame(StackEntryPush::new_completely_opaque_frame(jvm, LoaderName::BootstrapLoader, vec![], "main thread temp stack frame")); //todo think this is correct, check
                //handle any exceptions from here
                int_state.pop_frame(jvm, push_guard, false);
                let main_frame_guard = int_state.push_frame(StackEntryPush::new_completely_opaque_frame(jvm, LoaderName::BootstrapLoader, vec![], "main thread main frame"));
                run_main(args, jvm, &mut int_state).unwrap();
                //todo handle exception exit from main
                int_state.pop_frame(jvm, main_frame_guard, false);
                main_thread.notify_terminated(jvm)
            },
            box (),
        );
        main_send
    }

    pub(crate) fn debug_assertions<'l>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc, 'l>, loader_obj: ClassLoader<'gc>) {
        // for _ in 0..100{
        //     let list_cpdtype = CPDType::from_ptype(&PType::from_class(ClassName::Str("java/util/ArrayList".to_string())), &jvm.string_pool);
        //     let list_class_object = get_or_create_class_object(jvm,list_cpdtype,int_state).unwrap();
        //     let list_class = list_class_object.cast_class();
        //     let array_types = list_class.get_generic_interfaces(jvm,int_state).unwrap();
        //     for array_elem in array_types.unwrap_object_nonnull().unwrap_array().array_iterator() {
        //         assert!(array_elem.unwrap_object().is_some());
        //         // array_elem.unwrap_object_nonnull();
        //     }
        // }

        // let list_class = check_initing_or_inited_class(jvm, int_state, list_cpdtype).unwrap();
        // let input = vec![0,2328,1316134912];
        // let res = jvm.allocate_object(UnAllocatedObject::Array(UnAllocatedObjectArray{
        //     whole_array_runtime_class: check_initing_or_inited_class(jvm,int_state, CPDType::array(CPDType::IntType)).unwrap(),
        //     elems: input.clone().into_iter().map(|int|NewJavaValue::Int(int)).collect_vec()
        // }));
        // BigInteger::destructive_mul_add(jvm,int_state,res.new_java_value(),1000000000,0).unwrap();
        // assert_eq!(res.unwrap_array().array_iterator().map(|njv|njv.unwrap_int()).collect_vec(),vec![542,434162106,-1304428544]);
        // dbg!("start");
        // let input = vec![0, 0, 10000];
        // let res = jvm.allocate_object(UnAllocatedObject::Array(UnAllocatedObjectArray{
        //     whole_array_runtime_class: check_initing_or_inited_class(jvm,int_state, CPDType::array(CPDType::IntType)).unwrap(),
        //     elems: input.clone().into_iter().map(|int|NewJavaValue::Int(int)).collect_vec()
        // }));
        // BigInteger::destructive_mul_add(jvm,int_state,res.new_java_value(),1000000000,0).unwrap();
        // assert_eq!(res.unwrap_array().array_iterator().map(|njv|njv.unwrap_int()).collect_vec(),vec![0, 2328, 1316134912]);
        // panic!();
        // for i in [0,10,10000,10000000,1000000000000u128,10000000000000000000000u128]{
        //     let jstring = JString::from_rust(jvm, int_state, Wtf8Buf::from_string(format!("{}", i))).unwrap();
        //     let biginteger = BigInteger::new(jvm, int_state, jstring,10).unwrap();
        //     dbg!(biginteger.mag(jvm).unwrap_object_nonnull().unwrap_array().array_iterator().collect_vec());
        //     dbg!("start");
        //     let res = biginteger.to_string(jvm,int_state).unwrap().unwrap().to_rust_string(jvm);
        //     assert_eq!(res, format!("{}", i));
        // }
        //
        // let jstring = JString::from_rust(jvm, int_state, Wtf8Buf::from_string("+3660456486484698469816897408490640604354".to_string())).unwrap();
        // let biginteger = BigInteger::new(jvm, int_state, jstring,10).unwrap();
        // let res = biginteger.to_string(jvm,int_state).unwrap().unwrap().to_rust_string(jvm);
        // dbg!(res);
        // dbg!("here");
        // dbg!(loader_obj.hash_code(jvm, int_state).unwrap());
        // dbg!(loader_obj.hash_code(jvm, int_state).unwrap());
        // dbg!(loader_obj.to_string(jvm, int_state).unwrap().unwrap().to_rust_string(jvm));
        // dbg!(loader_obj.to_string(jvm, int_state).unwrap().unwrap().to_rust_string(jvm));
        // let jstring = JString::from_rust(jvm, int_state, Wtf8Buf::from_string("utf-8".to_string())).unwrap();
        // let res = jstring.hash_code(jvm, int_state).unwrap();
        // let mut hash_map = ConcurrentHashMap::new(jvm, int_state);
        // let first_key = JString::from_rust(jvm, int_state, Wtf8Buf::from_string("test".to_string())).unwrap();
        // let first_value = JString::from_rust(jvm, int_state, Wtf8Buf::from_string("test".to_string())).unwrap();
        // hash_map.put_if_absent(jvm, int_state, first_key.new_java_value(),first_value.new_java_value());
        //
        // let keys = ["test1",
        //     "test2",
        //     "test3",
        //     "test4",
        //     "test5",
        //     "test6",
        //     "test7",
        //     "test8",
        //     "test9",
        //     "test10",
        //     "test11",
        // ];
        // for key in keys{
        //     let key = JString::from_rust(jvm, int_state, Wtf8Buf::from_string(key.to_string())).unwrap();
        //     eprintln!("PUT START");
        //     hash_map.put_if_absent(jvm, int_state, key.new_java_value(),first_value.new_java_value());
        // }
        // dbg!(hash_map.size_ctl(jvm));
        // hash_map.debug_print_table(jvm);
        // let first_key = JString::from_rust(jvm, int_state, Wtf8Buf::from_string("sun.misc.Launcher$AppClassLoader@18b4aac2".to_string())).unwrap();
        // let first_value = JString::from_rust(jvm, int_state, Wtf8Buf::from_string("test".to_string())).unwrap();
        // hash_map.put_if_absent(jvm, int_state, first_key.new_java_value(), first_value.new_java_value());
        // let res = hash_map.get(jvm, int_state, first_key.new_java_value());
        // dbg!(res.unwrap_object().is_some());
        // hash_map.debug_print_table(jvm);
        // panic!()
        //print sizeCtl after put if absent
    }

    fn jvm_init_from_main_thread<'l>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc, 'l>) {
        let main_thread = jvm.thread_state.get_main_thread();
        main_thread.thread_object.read().unwrap().as_ref().unwrap().set_priority(JVMTI_THREAD_NORM_PRIORITY as i32);
        let system_class = assert_inited_or_initing_class(jvm, CClassName::system().into());


        let system = &system_class;
        let system_view = system.view();
        let method_views = system_view.lookup_method_name(MethodName::method_initializeSystemClass());
        let init_method_view = method_views.first().unwrap().clone();
        let method_id = jvm.method_table.write().unwrap().get_method_id(system_class.clone(), init_method_view.method_i());
        jvm.java_vm_state.add_method_if_needed(jvm, &MethodResolverImpl { jvm, loader: LoaderName::BootstrapLoader }, method_id, false);
        let mut locals = vec![];
        for _ in 0..init_method_view.code_attribute().unwrap().max_locals {
            locals.push(NewJavaValue::Top);
        }
        let initialize_system_frame = StackEntryPush::new_java_frame(jvm, system_class.clone(), init_method_view.method_i() as u16, locals);
        let init_frame_guard = int_state.push_frame(initialize_system_frame);
        assert!(Arc::ptr_eq(&main_thread, &jvm.thread_state.get_current_thread()));
        let _old = int_state.register_interpreter_state_guard(jvm);
        match run_function(&jvm, int_state) {
            Ok(_) => {}
            Err(_) => todo!(),
        }
        if int_state.throw().is_some() {
            unimplemented!()
        }
        set_properties(jvm, int_state).expect("todo");
        //todo read and copy props here
        let key = JString::from_rust(jvm, int_state, Wtf8Buf::from_string("java.home".to_string())).expect("todo");
        let value = JString::from_rust(jvm, int_state, Wtf8Buf::from_string("/home/francis/build/openjdk-debug/jdk8u/build/linux-x86_64-normal-server-slowdebug/jdk/".to_string())).expect("todo");
        System::props(jvm, int_state).set_property(jvm, int_state, key, value).expect("todo");

        let key = JString::from_rust(jvm, int_state, Wtf8Buf::from_string("log4j2.disable.jmx".to_string())).expect("todo");
        let value = JString::from_rust(jvm, int_state, Wtf8Buf::from_string("true".to_string())).expect("todo");
        System::props(jvm, int_state).set_property(jvm, int_state, key, value).expect("todo");

        //todo should handle exceptions here
        int_state.pop_frame(jvm, init_frame_guard, false);
        if !jvm.config.compiled_mode_active {}
    }

    pub fn get_main_thread(&self) -> Arc<JavaThread<'gc>> {
        self.main_thread.read().unwrap().as_ref().unwrap().clone()
    }

    pub(crate) fn set_current_thread(&'_ self, thread: Arc<JavaThread<'gc>>) {
        self.current_java_thread.with(|refcell| {
            assert!(refcell.borrow().is_none());
            unsafe {
                *refcell.borrow_mut() = transmute(Some(thread));
            }
        })
    }

    pub fn bootstrap_main_thread<'vm>(jvm: &'vm JVMState<'vm>, threads: &'vm Threads<'vm>) -> Arc<JavaThread<'vm>> {
        let bootstrap_underlying_thread = threads.create_thread("Bootstrap Thread".to_string().into());
        let stack_signal_safe_data = Arc::new(SignalAccessibleJavaStackData::new());
        let java_stack = Mutex::new(JavaStack::new(jvm, OwnedIRStack::new().expect("todo"), stack_signal_safe_data.clone()));
        let bootstrap_thread = Arc::new(JavaThread {
            java_tid: 0,
            java_stack,
            stack_signal_safe_data,
            underlying_thread: bootstrap_underlying_thread,
            thread_object: RwLock::new(None),
            interpreter_state: Mutex::new(InterpreterState::new(jvm).unwrap()),
            invisible_to_java: true,
            jvmti_events_enabled: Default::default(),
            thread_local_storage: RwLock::new(null_mut()),
            safepoint_state: SafePoint::new(),
            thread_status: RwLock::new(ThreadStatus { terminated: false, alive: false, interrupted: false }),
        });
        jvm.thread_state.set_current_thread(bootstrap_thread.clone());
        bootstrap_thread.notify_alive(jvm);
        let mut interpreter_state_guard = bootstrap_thread.interpreter_state.lock().unwrap();
        let mut new_int_state = InterpreterStateGuard::LocalInterpreterState {
            int_state: IRStackMut::from_stack_start(&mut interpreter_state_guard.call_stack.inner),
            thread: jvm.thread_state.get_current_thread(),
            registered: false,
            jvm,
            current_exited_pc: None,
            throw: None,
        };
        let _old = new_int_state.register_interpreter_state_guard(jvm);
        unsafe {
            jvm.native_libaries.load(jvm, &mut new_int_state, &jvm.native_libaries.libjava_path, "java".to_string());
            {
                let native_libs_guard = jvm.native_libaries.native_libs.read().unwrap();
                let libjava_native_lib = native_libs_guard.get("java").unwrap();
                let setup_hack_symbol: Symbol<unsafe extern "system" fn(*const JNIInvokeInterface_)> = libjava_native_lib.library.get("setup_jvm_pointer_hack".as_bytes()).unwrap();
                (*setup_hack_symbol.deref())(get_invoke_interface(jvm, &mut new_int_state))
            }
        }
        let frame = StackEntryPush::new_completely_opaque_frame(jvm, LoaderName::BootstrapLoader, vec![], "bootstrapping opaque frame");
        let frame_for_bootstrapping = new_int_state.push_frame(frame);
        let object_rc = check_loaded_class(jvm, &mut new_int_state, CClassName::object().into()).expect("This should really never happen, since it is equivalent to a class not found exception on java/lang/Object");
        jvm.verify_class_and_object(object_rc, jvm.classes.read().unwrap().class_class.clone());
        let thread_classfile = check_initing_or_inited_class(jvm, &mut new_int_state, CClassName::thread().into()).expect("couldn't load thread class");

        let thread_object = NewJavaValueHandle::Object(new_object_full(jvm, &mut new_int_state, &thread_classfile)).cast_thread();
        thread_object.set_priority(JVMTI_THREAD_NORM_PRIORITY as i32);
        *bootstrap_thread.thread_object.write().unwrap() = thread_object.into();
        let thread_group_class = check_initing_or_inited_class(jvm, &mut new_int_state, CClassName::thread_group().into()).expect("couldn't load thread group class");
        let system_thread_group = JThreadGroup::init(jvm, &mut new_int_state, thread_group_class).expect("todo");
        *jvm.thread_state.system_thread_group.write().unwrap() = system_thread_group.clone().into();
        let main_jthread = JThread::new(jvm, &mut new_int_state, system_thread_group, "Main".to_string()).expect("todo");
        new_int_state.pop_frame(jvm, frame_for_bootstrapping, false);
        bootstrap_thread.notify_terminated(jvm);
        JavaThread::new(jvm, main_jthread, threads.create_thread("Main Java Thread".to_string().into()), false).expect("todo")
    }

    pub fn get_current_thread_name(&self, jvm: &'gc JVMState<'gc>) -> String {
        let current_thread = self.get_current_thread();
        let thread_object = current_thread.thread_object.read().unwrap();
        thread_object.as_ref().map(|jthread| jthread.name(jvm).to_rust_string(jvm)).unwrap_or(std::thread::current().name().unwrap_or("unknown").to_string())
    }

    pub fn try_get_current_thread(&self) -> Option<Arc<JavaThread<'gc>>> {
        self.current_java_thread.with(|thread_refcell| unsafe { transmute(thread_refcell.borrow().clone()) })
    }

    pub fn new_monitor(&self, _name: String) -> Arc<Monitor2> {
        let mut monitor_guard = self.monitors.write().unwrap();
        let index = monitor_guard.len();
        let res = Arc::new(Monitor2::new(index));
        monitor_guard.push(res.clone());
        res
    }

    pub fn get_current_thread(&self) -> Arc<JavaThread<'gc>> {
        self.try_get_current_thread().unwrap()
    }

    pub fn get_current_thread_tid_or_invalid(&self) -> jlong {
        match self.try_get_current_thread() {
            None => -1,
            Some(current_thread) => current_thread.java_tid,
        }
    }

    pub fn get_monitor(&self, monitor: jrawMonitorID) -> Arc<Monitor2> {
        self.try_get_monitor(monitor).unwrap()
    }

    pub fn try_get_monitor(&self, monitor: jrawMonitorID) -> Option<Arc<Monitor2>> {
        let monitors_read_guard = self.monitors.read().unwrap();
        let monitor = monitors_read_guard.get(monitor as usize).cloned();
        drop(monitors_read_guard);
        monitor
    }

    pub fn get_thread_by_tid(&self, tid: JavaThreadId) -> Arc<JavaThread<'gc>> {
        self.try_get_thread_by_tid(tid).unwrap()
    }

    pub fn try_get_thread_by_tid(&self, tid: JavaThreadId) -> Option<Arc<JavaThread<'gc>>> {
        self.all_java_threads.read().unwrap().get(&tid).cloned()
    }

    pub fn start_thread_from_obj<'l>(&'gc self, jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc, 'l>, obj: JThread<'gc>, invisible_to_java: bool) -> Arc<JavaThread<'gc>> {
        let underlying = self.threads.create_thread(obj.name(jvm).to_rust_string(jvm).into());

        let (send, recv) = channel();
        let java_thread: Arc<JavaThread<'gc>> = JavaThread::new(jvm, obj, underlying, invisible_to_java).expect("todo");
        let loader_name = java_thread.thread_object.read().unwrap().as_ref().unwrap().get_context_class_loader(jvm, int_state).expect("todo").map(|class_loader| class_loader.to_jvm_loader(jvm)).unwrap_or(LoaderName::BootstrapLoader);
        java_thread.clone().underlying_thread.start_thread(
            box move |_data| {
                send.send(java_thread.clone()).unwrap();
                jvm.thread_state.set_current_thread(java_thread.clone());
                java_thread.notify_alive(jvm);
                Self::thread_start_impl(jvm, java_thread, loader_name)
            },
            box (),
        ); //todo is this Data really needed since we have a closure
        recv.recv().unwrap()
    }

    fn thread_start_impl<'l>(jvm: &'gc JVMState<'gc>, java_thread: Arc<JavaThread<'gc>>, loader_name: LoaderName) {
        let java_thread_clone: Arc<JavaThread<'gc>> = java_thread.clone();
        let mut state = java_thread_clone.interpreter_state.lock().unwrap();
        // let mut interpreter_state_guard: InterpreterStateGuard = InterpreterStateGuard::new(jvm, java_thread_clone.clone(), state); // { int_state: , thread: &java_thread };
        let mut interpreter_state_guard = InterpreterStateGuard::LocalInterpreterState {
            int_state: IRStackMut::from_stack_start(&mut state.call_stack.inner),
            thread: java_thread.clone(),
            registered: false,
            jvm,
            current_exited_pc: None,
            throw: None,
        };
        let should_be_nothing = interpreter_state_guard.register_interpreter_state_guard(jvm);
        assert!(should_be_nothing.old.is_none());

        if let Some(jvmti) = jvm.jvmti_state() {
            jvmti.built_in_jdwp.thread_start(jvm, &mut interpreter_state_guard, java_thread.clone().thread_object())
        }

        //todo fix loader
        let frame_for_run_call = interpreter_state_guard.push_frame(StackEntryPush::new_completely_opaque_frame(jvm, LoaderName::BootstrapLoader, vec![], "frame for calling run on a new thread"));
        if let Err(WasException {}) = java_thread.thread_object.read().unwrap().as_ref().unwrap().run(jvm, &mut interpreter_state_guard) {
            /*            JavaValue::Object(todo!() /*interpreter_state_guard.throw()*/)
                            .cast_throwable()
                            .print_stack_trace(jvm, &mut interpreter_state_guard)
                            .expect("Exception occured while printing exception. Something is pretty messed up");*/
            todo!();
            interpreter_state_guard.set_throw(None);
        };
        if let Err(WasException {}) = java_thread.thread_object.read().unwrap().as_ref().unwrap().exit(jvm, &mut interpreter_state_guard) {
            eprintln!("Exception occurred exiting thread, something is pretty messed up");
            panic!()
        }

        interpreter_state_guard.pop_frame(jvm, frame_for_run_call, false);
        java_thread.notify_terminated(jvm);
    }

    pub fn get_all_threads(&self) -> RwLockReadGuard<HashMap<JavaThreadId, Arc<JavaThread<'gc>>>> {
        self.all_java_threads.read().unwrap()
    }

    pub fn get_all_alive_threads(&self) -> Vec<Arc<JavaThread<'gc>>> {
        self.all_java_threads
            .read()
            .unwrap()
            .values()
            .filter(|_thread| {
                //don't use is_alive for this
                // todo!()
                true
                // thread.thread_object().is_alive(jvm, int_state) != 0
            })
            .cloned()
            .collect::<Vec<_>>()
    }

    pub fn get_system_thread_group(&self) -> JThreadGroup<'gc> {
        todo!()
        /*self.system_thread_group.read().unwrap().as_ref().unwrap().clone()*/
    }
}

thread_local! {
    static CURRENT_JAVA_THREAD: RefCell<Option<Arc<JavaThread<'static>>>> = RefCell::new(None);
}


pub struct JavaThread<'vm> {
    pub java_tid: JavaThreadId,
    java_stack: Mutex<JavaStack<'vm>>,
    stack_signal_safe_data: Arc<SignalAccessibleJavaStackData>,
    underlying_thread: Thread<'vm>,
    thread_object: RwLock<Option<JThread<'vm>>>,
    pub interpreter_state: Mutex<InterpreterState<'vm>>,
    pub invisible_to_java: bool,
    jvmti_events_enabled: RwLock<ThreadJVMTIEnabledStatus>,
    pub thread_local_storage: RwLock<*mut c_void>,
    pub safepoint_state: SafePoint<'vm>,
    pub thread_status: RwLock<ThreadStatus>,
}

impl<'gc> JavaThread<'gc> {
    pub fn is_alive(&self) -> bool {
        self.thread_status.read().unwrap().alive
    }

    pub fn new(jvm: &'gc JVMState<'gc>, thread_obj: JThread<'gc>, underlying: Thread<'gc>, invisible_to_java: bool) -> Result<Arc<JavaThread<'gc>>,CannotAllocateStack> {
        let stack_signal_safe_data = Arc::new(SignalAccessibleJavaStackData::new());
        let java_stack = Mutex::new(JavaStack::new(jvm, OwnedIRStack::new()?, stack_signal_safe_data.clone()));
        let res = Arc::new(JavaThread {
            java_tid: thread_obj.tid(jvm),
            java_stack,
            stack_signal_safe_data,
            underlying_thread: underlying,
            thread_object: RwLock::new(thread_obj.into()),
            interpreter_state: Mutex::new(InterpreterState::new(jvm).unwrap()),
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

    pub fn park<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc, 'l>, time_nanos: Option<u128>) -> Result<(), WasException> {
        unsafe { assert!(self.underlying_thread.is_this_thread()) }
        const NANOS_PER_SEC: u128 = 1_000_000_000u128;
        self.safepoint_state.set_park(time_nanos.map(|time_nanos| {
            let (secs, nanos) = time_nanos.div_mod_floor(&NANOS_PER_SEC);
            Duration::new(secs as u64, nanos as u32)
        }));
        self.safepoint_state.check(jvm, int_state)
    }

    pub fn unpark<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc, 'l>) -> Result<(), WasException> {
        self.safepoint_state.set_unpark();
        self.safepoint_state.check(jvm, int_state)
    }

    pub unsafe fn gc_suspend(&self) {
        self.safepoint_state.set_gc_suspended().unwrap(); //todo should use gc flag for this
    }

    pub unsafe fn suspend_thread<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc, 'l>, without_self_suspend: bool) -> Result<(), SuspendError> {
        if !self.is_alive() {
            return Err(SuspendError::NotAlive);
        }
        self.safepoint_state.set_suspended()?;
        if self.underlying_thread.is_this_thread() {
            assert_eq!(self.java_tid, int_state.thread().java_tid);
            if !without_self_suspend {
                safepoint_check(jvm, int_state)?;
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
}

#[derive(Debug)]
pub struct ThreadStatus {
    pub terminated: bool,
    pub alive: bool,
    pub interrupted: bool,
}

#[derive(Debug)]
pub enum SuspendError {
    AlreadySuspended,
    NotAlive,
    WasException(WasException),
}

#[derive(Debug)]
pub enum ResumeError {
    NotSuspended,
}

impl From<WasException> for SuspendError {
    fn from(we: WasException) -> Self {
        Self::WasException(we)
    }
}

pub mod monitors;
pub mod safepoints;