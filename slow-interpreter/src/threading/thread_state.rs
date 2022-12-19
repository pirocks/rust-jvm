use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::mem::transmute;
use std::sync::{Arc, RwLock, RwLockReadGuard};
use std::sync::mpsc::channel;
use std::thread::{LocalKey, Scope};
use itertools::{Itertools};

use jvmti_jni_bindings::{jlong, jrawMonitorID};
use rust_jvm_common::JavaThreadId;
use rust_jvm_common::loading::LoaderName;
use threads::Threads;

use crate::{JVMState, OpaqueFrame, PushableFrame, WasException};
use crate::better_java_stack::thread_remote_read_mechanism::{ThreadSignalBasedInterrupter};
use crate::stdlib::java::lang::class_loader::ClassLoader;
use crate::stdlib::java::lang::thread::JThread;
use crate::stdlib::java::lang::thread_group::JThreadGroup;
use crate::threading::java_thread::JavaThread;
use crate::threading::safepoints::Monitor2;

thread_local! {
    static CURRENT_JAVA_THREAD: RefCell<Option<Arc<JavaThread<'static>>>> = RefCell::new(None);
}


pub struct ThreadState<'gc> {
    pub threads: Threads<'gc>,
    pub interrupter: ThreadSignalBasedInterrupter,
    // threads_locals: RwLock<HashMap<ThreadId, Arc<FastPerThreadData>>>,
    pub(crate) main_thread: RwLock<Option<Arc<JavaThread<'gc>>>>,
    pub(crate) all_java_threads: RwLock<HashMap<JavaThreadId, Arc<JavaThread<'gc>>>>,
    current_java_thread: &'static LocalKey<RefCell<Option<Arc<JavaThread<'static>>>>>,
    pub system_thread_group: RwLock<Option<JThreadGroup<'gc>>>,
    pub(crate) monitors: RwLock<Vec<Arc<Monitor2>>>,
}


impl<'gc> ThreadState<'gc> {
    pub fn new(scope: &'gc Scope<'gc, 'gc>) -> Self {
        Self {
            threads: Threads::new(scope),
            interrupter: ThreadSignalBasedInterrupter::sigaction_setup(),
            main_thread: RwLock::new(None),
            all_java_threads: RwLock::new(HashMap::new()),
            current_java_thread: &CURRENT_JAVA_THREAD,
            system_thread_group: RwLock::new(None),
            monitors: RwLock::new(vec![]),
        }
    }

    pub(crate) fn debug_assert(&self, jvm: &'gc JVMState<'gc>){
        self.all_java_threads.read().unwrap().values().for_each(|thread|{
            let normal_object = match thread.thread_object.read().unwrap().as_ref() {
                Some(x) => x,
                None => return,
            }.normal_object.duplicate_discouraged();
            normal_object.new_java_handle().cast_thread(jvm);
        });
    }

    pub(crate) fn debug_assertions<'l>(_jvm: &'gc JVMState<'gc>, _int_state: &mut impl PushableFrame<'gc>, _loader_obj: ClassLoader<'gc>) {
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
        //     let jstring = JString::from_rust(jvm, pushable_frame_todo(), Wtf8Buf::from_string(format!("{}", i))).unwrap();
        //     let biginteger = BigInteger::new(jvm, int_state, jstring,10).unwrap();
        //     dbg!(biginteger.mag(jvm).unwrap_object_nonnull().unwrap_array().array_iterator().collect_vec());
        //     dbg!("start");
        //     let res = biginteger.to_string(jvm,int_state).unwrap().unwrap().to_rust_string(jvm);
        //     assert_eq!(res, format!("{}", i));
        // }
        //
        // let jstring = JString::from_rust(jvm, pushable_frame_todo(), Wtf8Buf::from_string("+3660456486484698469816897408490640604354".to_string())).unwrap();
        // let biginteger = BigInteger::new(jvm, int_state, jstring,10).unwrap();
        // let res = biginteger.to_string(jvm,int_state).unwrap().unwrap().to_rust_string(jvm);
        // dbg!(res);
        // dbg!("here");
        // dbg!(loader_obj.hash_code(jvm, int_state).unwrap());
        // dbg!(loader_obj.hash_code(jvm, int_state).unwrap());
        // dbg!(loader_obj.to_string(jvm, int_state).unwrap().unwrap().to_rust_string(jvm));
        // dbg!(loader_obj.to_string(jvm, int_state).unwrap().unwrap().to_rust_string(jvm));
        // let jstring = JString::from_rust(jvm, pushable_frame_todo(), Wtf8Buf::from_string("utf-8".to_string())).unwrap();
        // let res = jstring.hash_code(jvm, int_state).unwrap();
        // let mut hash_map = ConcurrentHashMap::new(jvm, int_state);
        // let first_key = JString::from_rust(jvm, pushable_frame_todo(), Wtf8Buf::from_string("test".to_string())).unwrap();
        // let first_value = JString::from_rust(jvm, pushable_frame_todo(), Wtf8Buf::from_string("test".to_string())).unwrap();
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
        //     let key = JString::from_rust(jvm, pushable_frame_todo(), Wtf8Buf::from_string(key.to_string())).unwrap();
        //     eprintln!("PUT START");
        //     hash_map.put_if_absent(jvm, int_state, key.new_java_value(),first_value.new_java_value());
        // }
        // dbg!(hash_map.size_ctl(jvm));
        // hash_map.debug_print_table(jvm);
        // let first_key = JString::from_rust(jvm, pushable_frame_todo(), Wtf8Buf::from_string("sun.misc.Launcher$AppClassLoader@18b4aac2".to_string())).unwrap();
        // let first_value = JString::from_rust(jvm, pushable_frame_todo(), Wtf8Buf::from_string("test".to_string())).unwrap();
        // hash_map.put_if_absent(jvm, int_state, first_key.new_java_value(), first_value.new_java_value());
        // let res = hash_map.get(jvm, int_state, first_key.new_java_value());
        // dbg!(res.unwrap_object().is_some());
        // hash_map.debug_print_table(jvm);
        // panic!()
        //print sizeCtl after put if absent
    }

    pub fn get_main_thread(&self) -> Arc<JavaThread<'gc>> {
        self.main_thread.read().unwrap().as_ref().unwrap().clone()
    }

    pub fn set_current_thread(&'_ self, thread: Arc<JavaThread<'gc>>) {
        self.current_java_thread.with(|refcell| {
            assert!(refcell.borrow().is_none());
            unsafe {
                *refcell.borrow_mut() = transmute(Some(thread));
            }
        })
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

    pub fn start_thread_from_obj<'l>(&'gc self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, obj: JThread<'gc>, invisible_to_java: bool) -> Arc<JavaThread<'gc>> {
        let (send, recv) = channel();
        let loader_name = obj
            .get_context_class_loader(jvm, int_state)
            .expect("todo")
            .map(|class_loader| class_loader.to_jvm_loader(jvm))
            .unwrap_or(LoaderName::BootstrapLoader);
        let _java_thread: Arc<JavaThread<'gc>> = JavaThread::background_new_with_stack(jvm, Some(obj), invisible_to_java, move |java_thread, frame| {
            send.send(java_thread.clone()).unwrap();
            Self::thread_start_impl(jvm, java_thread, loader_name, frame)
        }).expect("todo");
        recv.recv().unwrap()
    }

    fn thread_start_impl<'l>(jvm: &'gc JVMState<'gc>, java_thread: Arc<JavaThread<'gc>>, _loader_name: LoaderName, opaque_frame: &mut OpaqueFrame<'gc, 'l>)  -> Result<(), WasException<'gc>>{
        if let Some(jvmti) = jvm.jvmti_state() {
            jvmti.built_in_jdwp.thread_start(jvm, opaque_frame, java_thread.clone().thread_object())
        }

        //todo fix loader
        if let Err(WasException { exception_obj }) = java_thread.thread_object().run(jvm, opaque_frame) {
            exception_obj.print_stack_trace(jvm, opaque_frame)
                .expect("Exception occurred while printing exception. Something is pretty messed up");
            todo!()
        };
        if let Err(WasException { .. }) = java_thread.thread_object().exit(jvm, opaque_frame) {
            eprintln!("Exception occurred exiting thread, something is pretty messed up");
            panic!()
        }
        Ok(())
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

    pub fn wait_all_non_daemon_threads(&self, jvm: &'gc JVMState<'gc>) {
        loop {
            //technically should be daemon threads.
            let all_threads = self.all_java_threads.read().unwrap().values().cloned().collect_vec();
            drop(self.all_java_threads.read().unwrap());
            for thread in all_threads.iter(){
                if !thread.invisible_to_java && !thread.is_daemon(jvm) && thread.is_alive(){
                    thread.wait_thread_exit();
                }
            }
            let new_all_threads = self.all_java_threads.read().unwrap().values().cloned().collect_vec();
            let all_threads_ids = all_threads.into_iter().map(|thread|thread.java_tid).collect::<HashSet<_>>();
            let new_all_threads_ids = new_all_threads.into_iter().map(|thread|thread.java_tid).collect::<HashSet<_>>();
            let all_threads_done = all_threads_ids == new_all_threads_ids;
            if all_threads_done{
                break
            }
        }
    }

}
