//#![feature(asm)]

#![allow(non_snake_case)]
#![allow(unused)]

extern crate log;
extern crate simple_logger;
extern crate regex;

use std::str::from_utf8;
use std::borrow::Borrow;
use runtime_common::{InterpreterState, StackEntry};
use rust_jvm_common::classnames::{ClassName, class_name};
use slow_interpreter::{get_or_create_class_object, array_of_type_class};
use std::rc::Rc;
use std::intrinsics::transmute;
use slow_interpreter::rust_jni::native_util::{get_state, get_frame, to_object, from_object};
use jni_bindings::{JNIEnv, jclass, jstring, jobject, jlong, jint, jboolean, jobjectArray, jvalue, jbyte, jsize, jbyteArray, jfloat, jdouble, jmethodID, sockaddr, jintArray, jvm_version_info, getc, __va_list_tag, FILE, JVM_ExceptionTableEntryType, vsnprintf, JVM_CALLER_DEPTH, JavaVM, JNI_VERSION_1_8};
use log::trace;
use slow_interpreter::interpreter_util::{check_inited_class, push_new_object, run_function, run_constructor};
use slow_interpreter::instructions::ldc::{load_class_constant_by_name, create_string_on_stack};
use slow_interpreter::instructions::invoke::{invoke_virtual_method_i, invoke_special, actually_virtual};
use classfile_parser::types::{MethodDescriptor, parse_field_descriptor, parse_method_descriptor};
use rust_jvm_common::unified_types::{ParsedType, ClassWithLoader};
use runtime_common::java_values::{JavaValue, Object, ArrayObject};
use slow_interpreter::rust_jni::value_conversion::{native_to_runtime_class, runtime_class_to_native};
use std::sync::Arc;
use std::cell::RefCell;
use runtime_common::runtime_class::RuntimeClass;
use std::thread::Thread;
use slow_interpreter::rust_jni::string::new_string_with_string;
use std::ffi::{CStr, c_void};
use std::ops::Deref;
use std::collections::HashMap;
use std::collections::hash_map::RandomState;
use slow_interpreter::rust_jni::string::intern_impl;
use slow_interpreter::rust_jni::interface::runtime_class_from_object;
use rust_jvm_common::classfile::{ACC_INTERFACE, ACC_PUBLIC};
use std::os::raw::{c_int, c_char};
//so in theory I need something like this:
//    asm!(".symver JVM_GetEnclosingMethodInfo JVM_GetEnclosingMethodInfo@@SUNWprivate_1.1");
//but in reality I don't?

#[no_mangle]
unsafe extern "system" fn JVM_GetClassName(env: *mut JNIEnv, cls: jclass) -> jstring {
    let obj = runtime_class_from_object(cls).unwrap();
    let full_name = class_name(&obj.classfile).get_referred_name().replace("/", ".");
//    use regex::Regex;
//    let rg = Regex::new("/[A-Za-z_][A-Za-z_0-9]*");//todo use a correct regex
//    let class_name = rg.unwrap().captures(full_name.as_str()).unwrap().iter().last().unwrap().unwrap().as_str();
    new_string_with_string(env, full_name)
}

#[no_mangle]
unsafe extern "system" fn JVM_GetInterfaceVersion() -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_IHashCode(env: *mut JNIEnv, obj: jobject) -> jint {
    let _64bit: u64 = transmute(obj);
    ((_64bit >> 32) as i32 | _64bit as i32)
}


#[no_mangle]
unsafe extern "system" fn JVM_MonitorWait(env: *mut JNIEnv, obj: jobject, ms: jlong) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_MonitorNotify(env: *mut JNIEnv, obj: jobject) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_MonitorNotifyAll(env: *mut JNIEnv, obj: jobject) {
    //todo unimpl for now, since we don't support mutlithreading anyway
}

#[no_mangle]
unsafe extern "system" fn JVM_Clone(env: *mut JNIEnv, obj: jobject) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_InternString(env: *mut JNIEnv, str_unsafe: jstring) -> jstring {
    intern_impl(str_unsafe)
}

#[no_mangle]
unsafe extern "system" fn JVM_CurrentTimeMillis(env: *mut JNIEnv, ignored: jclass) -> jlong {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_NanoTime(env: *mut JNIEnv, ignored: jclass) -> jlong {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ArrayCopy(env: *mut JNIEnv, ignored: jclass, src: jobject, src_pos: jint, dst: jobject, dst_pos: jint, length: jint) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_InitProperties(env: *mut JNIEnv, p0: jobject) -> jobject {
//sun.boot.library.path
    let p1 = add_prop(env, p0, "sun.boot.library.path".to_string(), "/home/francis/Clion/rust-jvm/target/debug/deps:/home/francis/Desktop/jdk8u232-b09/jre/lib/amd64".to_string());
    let p2 = add_prop(env, p1, "java.library.path".to_string(), "/usr/java/packages/lib/amd64:/usr/lib64:/lib64:/lib:/usr/lib".to_string());
//    dbg!(from_object(p2).unwrap().unwrap_normal_object().fields.borrow().deref().get("table").unwrap());
    p2
}

unsafe fn add_prop(env: *mut JNIEnv, p: jobject, key: String, val: String) -> jobject {
    let frame = get_frame(env);
    let state = get_state(env);
    create_string_on_stack(state, &frame, key);
    let key = frame.pop();
    create_string_on_stack(state, &frame, val);
    let val = frame.pop();
    let prop_obj = from_object(p).unwrap();
    let runtime_class = &prop_obj.unwrap_normal_object().class_pointer;
    let classfile = &runtime_class.classfile;
    let candidate_meth = classfile.lookup_method_name(&"setProperty".to_string());
    let (meth_i, meth) = candidate_meth.iter().next().unwrap();
    let md = parse_method_descriptor(&runtime_class.loader, meth.descriptor_str(classfile).as_str()).unwrap();
    frame.push(JavaValue::Object(prop_obj.clone().into()));
    frame.push(key);
    frame.push(val);
    invoke_virtual_method_i(state, frame.clone(), md, runtime_class.clone(), *meth_i, meth);
    frame.pop();
    p
}


#[no_mangle]
unsafe extern "system" fn JVM_OnExit(func: ::std::option::Option<unsafe extern "C" fn()>) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_Exit(code: jint) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_Halt(code: jint) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GC() {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_MaxObjectInspectionAge() -> jlong {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_TraceInstructions(on: jboolean) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_TraceMethodCalls(on: jboolean) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_TotalMemory() -> jlong {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_FreeMemory() -> jlong {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_MaxMemory() -> jlong {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ActiveProcessorCount() -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_LoadLibrary(name: *const ::std::os::raw::c_char) -> *mut ::std::os::raw::c_void {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_UnloadLibrary(handle: *mut ::std::os::raw::c_void) {
    unimplemented!()
}

unsafe extern "system" fn provide_jni_version(jvm: *mut *mut JavaVM, something: *mut c_void) -> c_int {
    //todo I'm confused as to why this is returned from JVM_FindLibraryEntry, and I wrote this
    JNI_VERSION_1_8 as c_int
}

#[no_mangle]
unsafe extern "system" fn JVM_FindLibraryEntry(handle: *mut ::std::os::raw::c_void, name: *const ::std::os::raw::c_char) -> *mut ::std::os::raw::c_void {
//    unimplemented!();
    //todo not implemented for now

    transmute(provide_jni_version as *mut c_void)
}

#[no_mangle]
unsafe extern "system" fn JVM_IsSupportedJNIVersion(version: jint) -> jboolean {
    //todo for now we support everything?
    true as jboolean
}

#[no_mangle]
unsafe extern "system" fn JVM_IsNaN(d: jdouble) -> jboolean {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_FillInStackTrace(env: *mut JNIEnv, throwable: jobject) {
    //todo no stacktraces for now.
//    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetStackTraceDepth(env: *mut JNIEnv, throwable: jobject) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetStackTraceElement(env: *mut JNIEnv, throwable: jobject, index: jint) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_InitializeCompiler(env: *mut JNIEnv, compCls: jclass) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_IsSilentCompiler(env: *mut JNIEnv, compCls: jclass) -> jboolean {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_CompileClass(env: *mut JNIEnv, compCls: jclass, cls: jclass) -> jboolean {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_CompileClasses(env: *mut JNIEnv, cls: jclass, jname: jstring) -> jboolean {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_CompilerCommand(env: *mut JNIEnv, compCls: jclass, arg: jobject) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_EnableCompiler(env: *mut JNIEnv, compCls: jclass) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_DisableCompiler(env: *mut JNIEnv, compCls: jclass) {
    unimplemented!()
}

static mut MAIN_ALIVE: bool = false;

#[no_mangle]
unsafe extern "system" fn JVM_StartThread(env: *mut JNIEnv, thread: jobject) {
//    assert!(Arc::ptr_eq(MAIN_THREAD.as_ref().unwrap(),&from_object(thread).unwrap()));//todo why does this not pass?
    MAIN_ALIVE = true
}

#[no_mangle]
unsafe extern "system" fn JVM_StopThread(env: *mut JNIEnv, thread: jobject, exception: jobject) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_IsThreadAlive(env: *mut JNIEnv, thread: jobject) -> jboolean {
    MAIN_ALIVE as jboolean // todo we don't do threads atm.
}

#[no_mangle]
unsafe extern "system" fn JVM_SuspendThread(env: *mut JNIEnv, thread: jobject) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ResumeThread(env: *mut JNIEnv, thread: jobject) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_SetThreadPriority(env: *mut JNIEnv, thread: jobject, prio: jint) {
    //todo threads not implemented, noop
}

#[no_mangle]
unsafe extern "system" fn JVM_Yield(env: *mut JNIEnv, threadClass: jclass) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_Sleep(env: *mut JNIEnv, threadClass: jclass, millis: jlong) {
    unimplemented!()
}

static mut MAIN_THREAD: Option<Arc<Object>> = None;

#[no_mangle]
unsafe extern "system" fn JVM_CurrentThread(env: *mut JNIEnv, threadClass: jclass) -> jobject {
    match MAIN_THREAD.clone() {
        None => {
            let runtime_thread_class = runtime_class_from_object(threadClass).unwrap();
            let state = get_state(env);
            let frame = get_frame(env);
            make_thread(&runtime_thread_class, state, &frame);
            let thread_object = frame.pop().unwrap_object();
            MAIN_THREAD = thread_object.clone();
            to_object(thread_object)
        }
        Some(_) => {
            to_object(MAIN_THREAD.clone())
        }
    }
    //threads are not a thing atm.
    //todo
}


static mut SYSTEM_THREAD_GROUP: Option<Arc<Object>> = None;

fn init_system_thread_group(state: &mut InterpreterState, frame: &Rc<StackEntry>) {
    let thread_group_class = check_inited_class(state, &ClassName::Str("java/lang/ThreadGroup".to_string()), frame.clone().into(), frame.class_pointer.loader.clone());
    push_new_object(frame.clone(), &thread_group_class);
    let object = frame.pop();
    let (init_i, init) = thread_group_class.classfile.lookup_method("<init>".to_string(), "()V".to_string()).unwrap();
    let new_frame = StackEntry {
        last_call_stack: frame.clone().into(),
        class_pointer: thread_group_class.clone(),
        method_i: init_i as u16,
        local_vars: RefCell::new(vec![object.clone()]),
        operand_stack: RefCell::new(vec![]),
        pc: RefCell::new(0),
        pc_offset: RefCell::new(0),
    };
    unsafe { SYSTEM_THREAD_GROUP = object.unwrap_object(); }
    run_function(state, Rc::new(new_frame));
    if state.throw.is_some() || state.terminate {
        unimplemented!()
    }
    if state.function_return {
        state.function_return = false;
    }
}

unsafe fn make_thread(runtime_thread_class: &Arc<RuntimeClass>, state: &mut InterpreterState, frame: &Rc<StackEntry>) {
    //first create thread group
    let thread_group_object = match SYSTEM_THREAD_GROUP.clone() {
        None => {
            init_system_thread_group(state, frame);
            SYSTEM_THREAD_GROUP.clone()
        }
        Some(_) => SYSTEM_THREAD_GROUP.clone(),
    };


    let thread_class = check_inited_class(state, &ClassName::Str("java/lang/Thread".to_string()), frame.clone().into(), frame.class_pointer.loader.clone());
    if !Arc::ptr_eq(&thread_class, &runtime_thread_class) {
        frame.print_stack_trace();
    }
    assert!(Arc::ptr_eq(&thread_class, &runtime_thread_class));
    push_new_object(frame.clone(), &thread_class);
    let object = frame.pop();
    let (init_i, init) = thread_class.classfile.lookup_method("<init>".to_string(), "()V".to_string()).unwrap();
    let new_frame = StackEntry {
        last_call_stack: frame.clone().into(),
        class_pointer: thread_class.clone(),
        method_i: init_i as u16,
        local_vars: RefCell::new(vec![object.clone()]),
        operand_stack: RefCell::new(vec![]),
        pc: RefCell::new(0),
        pc_offset: RefCell::new(0),
    };
    MAIN_THREAD = object.unwrap_object().clone();
    MAIN_THREAD.clone().unwrap().unwrap_normal_object().fields.borrow_mut().insert("group".to_string(), JavaValue::Object(thread_group_object));
    //for some reason the constructor doesn't handle priority.
    let NORM_PRIORITY = 5;
    MAIN_THREAD.clone().unwrap().unwrap_normal_object().fields.borrow_mut().insert("priority".to_string(), JavaValue::Int(NORM_PRIORITY));
    run_function(state, Rc::new(new_frame));
    frame.push(JavaValue::Object(MAIN_THREAD.clone()));
//    dbg!(&frame.operand_stack);
    if state.throw.is_some() || state.terminate {
        unimplemented!()
    }
    if state.function_return {
        state.function_return = false;
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_CountStackFrames(env: *mut JNIEnv, thread: jobject) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_Interrupt(env: *mut JNIEnv, thread: jobject) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_IsInterrupted(env: *mut JNIEnv, thread: jobject, clearInterrupted: jboolean) -> jboolean {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_HoldsLock(env: *mut JNIEnv, threadClass: jclass, obj: jobject) -> jboolean {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_DumpAllStacks(env: *mut JNIEnv, unused: jclass) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetAllThreads(env: *mut JNIEnv, dummy: jclass) -> jobjectArray {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_SetNativeThreadName(env: *mut JNIEnv, jthread: jobject, name: jstring) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_DumpThreads(env: *mut JNIEnv, threadClass: jclass, threads: jobjectArray) -> jobjectArray {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_CurrentLoadedClass(env: *mut JNIEnv) -> jclass {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_CurrentClassLoader(env: *mut JNIEnv) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassContext(env: *mut JNIEnv) -> jobjectArray {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ClassDepth(env: *mut JNIEnv, name: jstring) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ClassLoaderDepth(env: *mut JNIEnv) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetSystemPackage(env: *mut JNIEnv, name: jstring) -> jstring {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetSystemPackages(env: *mut JNIEnv) -> jobjectArray {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_AllocateNewObject(env: *mut JNIEnv, obj: jobject, currClass: jclass, initClass: jclass) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_AllocateNewArray(env: *mut JNIEnv, obj: jobject, currClass: jclass, length: jint) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_LatestUserDefinedLoader(env: *mut JNIEnv) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_LoadClass0(env: *mut JNIEnv, obj: jobject, currClass: jclass, currClassName: jstring) -> jclass {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetArrayLength(env: *mut JNIEnv, arr: jobject) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetArrayElement(env: *mut JNIEnv, arr: jobject, index: jint) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetPrimitiveArrayElement(env: *mut JNIEnv, arr: jobject, index: jint, wCode: jint) -> jvalue {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_SetArrayElement(env: *mut JNIEnv, arr: jobject, index: jint, val: jobject) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_SetPrimitiveArrayElement(env: *mut JNIEnv, arr: jobject, index: jint, v: jvalue, vCode: ::std::os::raw::c_uchar) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_NewArray(env: *mut JNIEnv, eltClass: jclass, length: jint) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_NewMultiArray(env: *mut JNIEnv, eltClass: jclass, dim: jintArray) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetCallerClass(env: *mut JNIEnv, depth: ::std::os::raw::c_int) -> jclass {
    /*todo, so this is needed for booting but it is what could best be described as an advanced feature.
    Therefore it only sorta works*/
    let frame = get_frame(env);
    let state = get_state(env);

    load_class_constant_by_name(state, &frame, class_name(&frame.last_call_stack.as_ref().unwrap().class_pointer.classfile).get_referred_name());
    let jclass = frame.pop().unwrap_object();
    to_object(jclass)
}

#[no_mangle]
unsafe extern "system" fn JVM_FindPrimitiveClass(env: *mut JNIEnv, utf: *const ::std::os::raw::c_char) -> jclass {
    // need to perform not equal to 0 check
    if *utf.offset(0) == 'f' as i8 &&
        *utf.offset(1) == 'l' as i8 &&
        *utf.offset(2) == 'o' as i8 &&
        *utf.offset(3) == 'a' as i8 &&
        *utf.offset(4) == 't' as i8 &&
        *utf.offset(5) == 0 {
        let state = get_state(env);
        let frame = get_frame(env);
        let res = get_or_create_class_object(state, &ClassName::new("java/lang/Float"), frame, state.bootstrap_loader.clone());//todo what if not using bootstap loader
        return to_object(res.into());
    }
    if *utf.offset(0) == 'd' as i8 &&
        *utf.offset(1) == 'o' as i8 &&
        *utf.offset(2) == 'u' as i8 &&
        *utf.offset(3) == 'b' as i8 &&
        *utf.offset(4) == 'l' as i8 &&
        *utf.offset(5) == 'e' as i8 &&
        *utf.offset(6) == 0 {
        let state = get_state(env);
        let frame = get_frame(env);
        let res = get_or_create_class_object(state, &ClassName::new("java/lang/Double"), frame, state.bootstrap_loader.clone());//todo what if not using bootstap loader
        return to_object(res.into());
    }
    if *utf.offset(0) == 'i' as i8 &&
        *utf.offset(1) == 'n' as i8 &&
        *utf.offset(2) == 't' as i8 &&
        *utf.offset(3) == 0 as i8 {
        let state = get_state(env);
        let frame = get_frame(env);
        let res = get_or_create_class_object(state, &ClassName::new("java/lang/Integer"), frame, state.bootstrap_loader.clone());//todo what if not using bootstap loader
        return to_object(res.into());
    }
    if *utf.offset(0) == 'b' as i8 &&
        *utf.offset(1) == 'o' as i8 &&
        *utf.offset(2) == 'o' as i8 &&
        *utf.offset(3) == 'l' as i8 &&
        *utf.offset(4) == 'e' as i8 &&
        *utf.offset(5) == 'a' as i8 &&
        *utf.offset(6) == 'n' as i8 &&
        *utf.offset(7) == 0 {
        let state = get_state(env);
        let frame = get_frame(env);
        let res = get_or_create_class_object(state, &ClassName::new("java/lang/Boolean"), frame, state.bootstrap_loader.clone());//todo what if not using bootstap loader
        return to_object(res.into());
    }
    if *utf.offset(0) == 'c' as i8 &&
        *utf.offset(1) == 'h' as i8 &&
        *utf.offset(2) == 'a' as i8 &&
        *utf.offset(3) == 'r' as i8 &&
        *utf.offset(4) == 0 {
        let state = get_state(env);
        let frame = get_frame(env);
        let res = get_or_create_class_object(state, &ClassName::new("java/lang/Character"), frame, state.bootstrap_loader.clone());//todo what if not using bootstap loader
        return to_object(res.into());
    }

    if *utf.offset(0) == 'l' as i8 &&
        *utf.offset(1) == 'o' as i8 &&
        *utf.offset(2) == 'n' as i8 &&
        *utf.offset(3) == 'g' as i8 &&
        *utf.offset(4) == 0 {
        let state = get_state(env);
        let frame = get_frame(env);
        let res = get_or_create_class_object(state, &ClassName::new("java/lang/Long"), frame, state.bootstrap_loader.clone());//todo what if not using bootstap loader
        return to_object(res.into());
    }

    dbg!((*utf) as u8 as char);
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ResolveClass(env: *mut JNIEnv, cls: jclass) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_FindClassFromBootLoader(env: *mut JNIEnv, name: *const ::std::os::raw::c_char) -> jclass {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_FindClassFromClassLoader(env: *mut JNIEnv, name: *const ::std::os::raw::c_char, init: jboolean, loader: jobject, throwError: jboolean) -> jclass {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_FindClassFromClass(env: *mut JNIEnv, name: *const ::std::os::raw::c_char, init: jboolean, from: jclass) -> jclass {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_FindLoadedClass(env: *mut JNIEnv, loader: jobject, name: jstring) -> jclass {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_DefineClass(env: *mut JNIEnv, name: *const ::std::os::raw::c_char, loader: jobject, buf: *const jbyte, len: jsize, pd: jobject) -> jclass {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_DefineClassWithSource(env: *mut JNIEnv, name: *const ::std::os::raw::c_char, loader: jobject, buf: *const jbyte, len: jsize, pd: jobject, source: *const ::std::os::raw::c_char) -> jclass {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassInterfaces(env: *mut JNIEnv, cls: jclass) -> jobjectArray {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassLoader(env: *mut JNIEnv, cls: jclass) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_IsInterface(env: *mut JNIEnv, cls: jclass) -> jboolean {
//    get_frame(env).print_stack_trace();
    (runtime_class_from_object(cls).unwrap().classfile.access_flags & ACC_INTERFACE > 0) as jboolean
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassSigners(env: *mut JNIEnv, cls: jclass) -> jobjectArray {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_SetClassSigners(env: *mut JNIEnv, cls: jclass, signers: jobjectArray) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetProtectionDomain(env: *mut JNIEnv, cls: jclass) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_IsArrayClass(env: *mut JNIEnv, cls: jclass) -> jboolean {
    unimplemented!()
}

#[no_mangle]
/**
    * Determines if the specified {@code Class} object represents a
    * primitive type.
    *
    * <p> There are nine predefined {@code Class} objects to represent
    * the eight primitive types and void.  These are created by the Java
    * Virtual Machine, and have the same names as the primitive types that
    * they represent, namely {@code boolean}, {@code byte},
    * {@code char}, {@code short}, {@code int},
    * {@code long}, {@code float}, and {@code double}.
    *
    * <p> These objects may only be accessed via the following public static
    * final variables, and are the only {@code Class} objects for which
    * this method returns {@code true}.
    *
    * @return true if and only if this class represents a primitive type
    *
    * @see     java.lang.Boolean#TYPE
    * @see     java.lang.Character#TYPE
    * @see     java.lang.Byte#TYPE
    * @see     java.lang.Short#TYPE
    * @see     java.lang.Integer#TYPE
    * @see     java.lang.Long#TYPE
    * @see     java.lang.Float#TYPE
    * @see     java.lang.Double#TYPE
    * @see     java.lang.Void#TYPE
    * @since JDK1.1
    */
unsafe extern "system" fn JVM_IsPrimitiveClass(env: *mut JNIEnv, cls: jclass) -> jboolean {
//    get_frame(env).print_stack_trace();
    let class_object = runtime_class_from_object(cls);
    if class_object.is_none() {
        return false as jboolean;
    }
    let name = class_name(&class_object.unwrap().classfile).get_referred_name();
    dbg!(&name);
    let is_primitive = name == "java/lang/Boolean".to_string() ||
        name == "java/lang/Character".to_string() ||
        name == "java/lang/Byte".to_string() ||
        name == "java/lang/Short".to_string() ||
        name == "java/lang/Integer".to_string() ||
        name == "java/lang/Long".to_string() ||
        name == "java/lang/Float".to_string() ||
        name == "java/lang/Double".to_string() ||
        name == "java/lang/Void".to_string();

    is_primitive as jboolean
}

#[no_mangle]
unsafe extern "system" fn JVM_GetComponentType(env: *mut JNIEnv, cls: jclass) -> jclass {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassModifiers(env: *mut JNIEnv, cls: jclass) -> jint {
    runtime_class_from_object(cls).unwrap().classfile.access_flags as jint
}

#[no_mangle]
unsafe extern "system" fn JVM_GetDeclaredClasses(env: *mut JNIEnv, ofClass: jclass) -> jobjectArray {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetDeclaringClass(env: *mut JNIEnv, ofClass: jclass) -> jclass {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassSignature(env: *mut JNIEnv, cls: jclass) -> jstring {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassAnnotations(env: *mut JNIEnv, cls: jclass) -> jbyteArray {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassTypeAnnotations(env: *mut JNIEnv, cls: jclass) -> jbyteArray {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetFieldTypeAnnotations(env: *mut JNIEnv, field: jobject) -> jbyteArray {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodTypeAnnotations(env: *mut JNIEnv, method: jobject) -> jbyteArray {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassDeclaredMethods(env: *mut JNIEnv, ofClass: jclass, publicOnly: jboolean) -> jobjectArray {
    unimplemented!()
}

fn field_type_to_class(state: &mut InterpreterState, frame: &Rc<StackEntry>, type_: &ParsedType) -> JavaValue {
    match type_ {
        ParsedType::IntType => {
            load_class_constant_by_name(state, frame, "java/lang/Integer".to_string());
        }
        ParsedType::Class(cl) => {
            load_class_constant_by_name(state, frame, cl.class_name.get_referred_name());
        }
        ParsedType::BooleanType => {
            //todo dup.
            load_class_constant_by_name(state, frame, "java/lang/Boolean".to_string());
        }
        ParsedType::LongType => {
            //todo dup.
            load_class_constant_by_name(state, frame, "java/lang/Long".to_string());
        }
        ParsedType::ArrayReferenceType(sub) => {
            frame.push(JavaValue::Object(array_of_type_class(state, frame.clone(), sub.sub_type.deref()).into()));
        }
        ParsedType::CharType => {
            load_class_constant_by_name(state, frame, "java/lang/Character".to_string());
        }
        _ => {
            dbg!(type_);
            frame.print_stack_trace();
            unimplemented!()
        }
    }
    frame.pop()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassDeclaredFields(env: *mut JNIEnv, ofClass: jclass, publicOnly: jboolean) -> jobjectArray {
    let frame = get_frame(env);
    let state = get_state(env);
//    frame.print_stack_trace();
    let class_obj = runtime_class_from_object(ofClass);
//    dbg!(&class_obj.clone().unwrap_normal_object().class_pointer);
//    let runtime_object = state.class_object_pool.borrow().get(&class_obj).unwrap();
    let field_classfile = check_inited_class(state, &ClassName::Str("java/lang/reflect/Field".to_string()), frame.clone().into(), frame.class_pointer.loader.clone());
    let mut object_array = vec![];
    &class_obj.clone().unwrap().classfile.fields.iter().enumerate().for_each(|(i, f)| {
        push_new_object(frame.clone(), &field_classfile);
        let field_object = frame.pop();

        object_array.push(field_object.clone());
        let field_class_name = class_name(&class_obj.clone().as_ref().unwrap().classfile).get_referred_name();
        load_class_constant_by_name(state, &frame, field_class_name);
        let parent_runtime_class = frame.pop();
        let field_name = class_obj.clone().unwrap().classfile.constant_pool[f.name_index as usize].extract_string_from_utf8();
        create_string_on_stack(state, &frame, field_name);
        let field_name_string = frame.pop();

        let field_desc_str = class_obj.clone().unwrap().classfile.constant_pool[f.descriptor_index as usize].extract_string_from_utf8();
        let field_type = parse_field_descriptor(&frame.class_pointer.loader, field_desc_str.as_str()).unwrap().field_type;
        let field_type_class = field_type_to_class(state, &frame, &field_type);

        let modifiers = JavaValue::Int(f.access_flags as i32);
        let slot = JavaValue::Int(i as i32);

        create_string_on_stack(state, &frame, field_desc_str);
        let signature_string = frame.pop();

        //todo impl annotations.
        let annotations = JavaValue::Object(Some(Arc::new(Object::Array(ArrayObject { elems: RefCell::new(vec![]), elem_type: ParsedType::ByteType }))));

        run_constructor(
            state,
            frame.clone(),
            field_classfile.clone(),
            vec![field_object, parent_runtime_class, field_name_string, field_type_class, modifiers, slot, signature_string, annotations],
            "(Ljava/lang/Class;Ljava/lang/String;Ljava/lang/Class;IILjava/lang/String;[B)V".to_string(),
        )
    });

    //first arg: runtime_class
    //second arg: name
    //third arg: type class pointer
    //fourth arg: access_flags
    //fifth: put index here
    //descriptor
    //just put empty byte array??
//    Field(Class<?> var1, String var2, Class<?> var3, int var4, int var5, String var6, byte[] var7) {
//        this.clazz = var1;
//        this.name = var2;
//        this.type = var3;
//        this.modifiers = var4;
//        this.slot = var5;
//        this.signature = var6;
//        this.annotations = var7;
//    }
//    class_obj.unwrap()

    let res = Some(Arc::new(Object::Array(ArrayObject { elems: RefCell::new(object_array), elem_type: ParsedType::Class(ClassWithLoader { class_name: class_name(&field_classfile.classfile), loader: field_classfile.loader.clone() }) })));
    to_object(res)
}


const CONSTRUCTOR_SIGNATURE: &'static str = "(Ljava/lang/Class;[Ljava/lang/Class;[Ljava/lang/Class;IILjava/lang/String;[B[B)V";

#[no_mangle]
unsafe extern "system" fn JVM_GetClassDeclaredConstructors(env: *mut JNIEnv, ofClass: jclass, publicOnly: jboolean) -> jobjectArray {
    let temp = runtime_class_from_object(ofClass).unwrap();
    let target_classfile = &temp.classfile;
    let constructors = target_classfile.lookup_method_name(&"<init>".to_string());
    let state = get_state(env);
    let frame = get_frame(env);
    let class_obj = runtime_class_from_object(ofClass);
    let loader = frame.class_pointer.loader.clone();
    let constructor_class = check_inited_class(state, &ClassName::new("java/lang/reflect/Constructor"), frame.clone().into(), loader.clone());
    let mut object_array = vec![];

    constructors.clone().iter().filter(|(i, m)| {
        if publicOnly > 0 {
            m.access_flags & ACC_PUBLIC > 0
        } else {
            true
        }
    }).for_each(|(i, m)| {
        let class_type = ParsedType::Class(ClassWithLoader { class_name: ClassName::class(), loader: loader.clone() });//todo this should be a global const

        push_new_object(frame.clone(), &constructor_class);
        let constructor_object = frame.pop();

        object_array.push(constructor_object.clone());

        let clazz = {
            let field_class_name = class_name(&class_obj.clone().as_ref().unwrap().classfile).get_referred_name();
            load_class_constant_by_name(state, &frame, field_class_name);
            frame.pop()
        };

        let parameter_types = {
            let mut res = vec![];
            let desc_str = m.descriptor_str(&target_classfile);
            let parsed = parse_method_descriptor(&loader, desc_str.as_str()).unwrap();
            for param_type in parsed.parameter_types {
                res.push(match param_type {
                    ParsedType::Class(c) => {
                        load_class_constant_by_name(state, &frame, c.class_name.get_referred_name());
                        frame.pop()
                    }
                    _ => unimplemented!()
                });
            }

            JavaValue::Object(Some(Arc::new(Object::Array(ArrayObject { elems: RefCell::new(res), elem_type: class_type.clone() }))))
        };


        let exceptionTypes = {
            //todo not currently supported
            assert!(m.code_attribute().unwrap().exception_table.is_empty());
            JavaValue::Object(Some(Arc::new(Object::Array(ArrayObject { elems: RefCell::new(vec![]), elem_type: class_type.clone() }))))
        };

        let modifiers = JavaValue::Int(constructor_class.classfile.access_flags as i32);
        //todo what does slot do?
        let slot = JavaValue::Int(-1);

        let signature = {
            create_string_on_stack(state, &frame, m.descriptor_str(&target_classfile));
            frame.pop()
        };

        //todo impl these
        let empty_byte_array = JavaValue::Object(Some(Arc::new(Object::Array(ArrayObject { elems: RefCell::new(vec![]), elem_type: ParsedType::ByteType }))));

        let full_args = vec![constructor_object, clazz, parameter_types, exceptionTypes, modifiers, slot, signature, empty_byte_array.clone(), empty_byte_array];
        run_constructor(state, frame.clone(), constructor_class.clone(), full_args, CONSTRUCTOR_SIGNATURE.to_string())
    });
    let res = Some(Arc::new(Object::Array(ArrayObject { elems: RefCell::new(object_array), elem_type: ParsedType::Class(ClassWithLoader { class_name: class_name(&constructor_class.classfile), loader: constructor_class.loader.clone() }) })));
    to_object(res)
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassAccessFlags(env: *mut JNIEnv, cls: jclass) -> jint {
    runtime_class_from_object(cls).unwrap().classfile.access_flags as i32
}

#[no_mangle]
unsafe extern "system" fn JVM_InvokeMethod(env: *mut JNIEnv, method: jobject, obj: jobject, args0: jobjectArray) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_NewInstanceFromConstructor(env: *mut JNIEnv, c: jobject, args0: jobjectArray) -> jobject {
//    assert_ne!(args0, std::ptr::null_mut());
    let args = if args0 == std::ptr::null_mut() {
        vec![]
    } else {
        let temp_1 = from_object(args0).unwrap().clone();
        let array_temp = temp_1.unwrap_array().borrow();
        let elems_refcell = array_temp.elems.borrow();
        elems_refcell.clone()
    };
    let constructor_obj = from_object(c).unwrap();
    let constructor_obj_fields = constructor_obj.unwrap_normal_object().fields.borrow();
    let signature_str_obj = constructor_obj_fields.get("signature").unwrap();
    let temp_4 = constructor_obj_fields.get("clazz").unwrap().unwrap_object().unwrap();
    let temp_3 = temp_4.unwrap_normal_object().object_class_object_pointer.borrow();
    let clazz = temp_3.as_ref().unwrap().clone();
    let temp_2 = signature_str_obj.unwrap_object().unwrap().unwrap_normal_object().fields.borrow().get("value").unwrap().unwrap_object().unwrap();
    let sig_chars = &temp_2.unwrap_array().borrow().elems;
    let mut signature = String::new();
    for char_ in sig_chars.borrow().iter() {
        signature.push(char_.unwrap_char())
    }
    let state = get_state(env);
    let frame = get_frame(env);
    push_new_object(frame.clone(), &clazz);
    let obj = frame.pop();
    let mut full_args = vec![obj.clone()];
    full_args.extend(args.iter().cloned());
    run_constructor(state, frame, clazz, full_args, signature);
    to_object(obj.unwrap_object())
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassConstantPool(env: *mut JNIEnv, cls: jclass) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetSize(env: *mut JNIEnv, unused: jobject, jcpool: jobject) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetClassAt(env: *mut JNIEnv, unused: jobject, jcpool: jobject, index: jint) -> jclass {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetClassAtIfLoaded(env: *mut JNIEnv, unused: jobject, jcpool: jobject, index: jint) -> jclass {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetMethodAt(env: *mut JNIEnv, unused: jobject, jcpool: jobject, index: jint) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetMethodAtIfLoaded(env: *mut JNIEnv, unused: jobject, jcpool: jobject, index: jint) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetFieldAt(env: *mut JNIEnv, unused: jobject, jcpool: jobject, index: jint) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetFieldAtIfLoaded(env: *mut JNIEnv, unused: jobject, jcpool: jobject, index: jint) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetMemberRefInfoAt(env: *mut JNIEnv, unused: jobject, jcpool: jobject, index: jint) -> jobjectArray {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetIntAt(env: *mut JNIEnv, unused: jobject, jcpool: jobject, index: jint) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetLongAt(env: *mut JNIEnv, unused: jobject, jcpool: jobject, index: jint) -> jlong {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetFloatAt(env: *mut JNIEnv, unused: jobject, jcpool: jobject, index: jint) -> jfloat {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetDoubleAt(env: *mut JNIEnv, unused: jobject, jcpool: jobject, index: jint) -> jdouble {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetStringAt(env: *mut JNIEnv, unused: jobject, jcpool: jobject, index: jint) -> jstring {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetUTF8At(env: *mut JNIEnv, unused: jobject, jcpool: jobject, index: jint) -> jstring {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodParameters(env: *mut JNIEnv, method: jobject) -> jobjectArray {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_DoPrivileged(env: *mut JNIEnv, cls: jclass, action: jobject, context: jobject, wrapException: jboolean) -> jobject {
//    if wrapException == 0{
//        unimplemented!()
//    }
    let state = get_state(env);
    let frame = get_frame(env);
    let action = from_object(action);
//    dbg!(&class_name(&action.as_ref().unwrap().unwrap_object().class_pointer.classfile));
//    dbg!(&action.as_re/f().unwrap().unwrap_object().fields.borrow().keys());
    let unwrapped_action = action.clone().unwrap();
    let runtime_class = &unwrapped_action.unwrap_normal_object().class_pointer;
    let classfile = &runtime_class.classfile;
    let (run_method_i, run_method) = classfile.lookup_method("run".to_string(), "()Ljava/lang/Object;".to_string()).unwrap();
    let expected_descriptor = MethodDescriptor {
        parameter_types: vec![],
        return_type: ParsedType::Class(ClassWithLoader { class_name: ClassName::object(), loader: runtime_class.loader.clone() }),
    };
    frame.push(JavaValue::Object(action));
//    dbg!(&frame.operand_stack);
//    dbg!(&run_method.code_attribute().unwrap());
    //todo shouldn't this be invoke_virtual
    actually_virtual(state, frame.clone(), expected_descriptor, &runtime_class, run_method);
//    dbg!(&frame.operand_stack);
//    unimplemented!()

    let res = frame.pop().unwrap_object();
//    dbg!(&res);
    to_object(res)
}

#[no_mangle]
unsafe extern "system" fn JVM_GetInheritedAccessControlContext(env: *mut JNIEnv, cls: jclass) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetStackAccessControlContext(env: *mut JNIEnv, cls: jclass) -> jobject {
//    let frame = get_frame(env);
//    frame.print_stack_trace();
    //todo this is obscure java stuff that isn't supported atm.
    to_object(None)
}

#[no_mangle]
unsafe extern "system" fn JVM_RegisterSignal(sig: jint, handler: *mut ::std::os::raw::c_void) -> *mut ::std::os::raw::c_void {
    //todo unimpl for now
    transmute(0xdeaddeadbeafdead as usize)
}

#[no_mangle]
unsafe extern "system" fn JVM_RaiseSignal(sig: jint) -> jboolean {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_FindSignal(name: *const ::std::os::raw::c_char) -> jint {
    if name.offset(0).read() == 'H' as c_char && name.offset(1).read() == 'U' as c_char && name.offset(2).read() == 'P' as c_char {
        1 //todo bindgen signal.h
    } else if name.offset(0).read() == 'I' as c_char && name.offset(1).read() == 'N' as c_char && name.offset(2).read() == 'T' as c_char {
        2 //todo bindgen signal.h
    } else if name.offset(0).read() == 'T' as c_char && name.offset(1).read() == 'E' as c_char && name.offset(2).read() == 'R' as c_char && name.offset(3).read() == 'M' as c_char {
        15 //todo bindgen signal.h
    } else {
        unimplemented!()
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_DesiredAssertionStatus(env: *mut JNIEnv, unused: jclass, cls: jclass) -> jboolean {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_AssertionStatusDirectives(env: *mut JNIEnv, unused: jclass) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_SupportsCX8() -> jboolean {
    false as jboolean//todo this is actually something that might be easy to support.
}


#[no_mangle]
unsafe extern "system" fn JVM_DTraceGetVersion(env: *mut JNIEnv) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_DTraceIsProbeEnabled(env: *mut JNIEnv, method: jmethodID) -> jboolean {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_DTraceDispose(env: *mut JNIEnv, activation_handle: jlong) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_DTraceIsSupported(env: *mut JNIEnv) -> jboolean {
    unimplemented!()
}

#[doc = "PART 2: Support for the Verifier and Class File Format Checker"]
#[no_mangle]
unsafe extern "system" fn JVM_GetClassNameUTF(env: *mut JNIEnv, cb: jclass) -> *const ::std::os::raw::c_char {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassCPTypes(env: *mut JNIEnv, cb: jclass, types: *mut ::std::os::raw::c_uchar) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassCPEntriesCount(env: *mut JNIEnv, cb: jclass) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassFieldsCount(env: *mut JNIEnv, cb: jclass) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassMethodsCount(env: *mut JNIEnv, cb: jclass) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxExceptionsCount(env: *mut JNIEnv, cb: jclass, method_index: jint) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxByteCode(env: *mut JNIEnv, cb: jclass, method_index: jint, code: *mut ::std::os::raw::c_uchar) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxByteCodeLength(env: *mut JNIEnv, cb: jclass, method_index: jint) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxExceptionTableLength(env: *mut JNIEnv, cb: jclass, index: ::std::os::raw::c_int) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetFieldIxModifiers(env: *mut JNIEnv, cb: jclass, index: ::std::os::raw::c_int) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxModifiers(env: *mut JNIEnv, cb: jclass, index: ::std::os::raw::c_int) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxLocalsCount(env: *mut JNIEnv, cb: jclass, index: ::std::os::raw::c_int) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxArgsSize(env: *mut JNIEnv, cb: jclass, index: ::std::os::raw::c_int) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxMaxStack(env: *mut JNIEnv, cb: jclass, index: ::std::os::raw::c_int) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_IsConstructorIx(env: *mut JNIEnv, cb: jclass, index: ::std::os::raw::c_int) -> jboolean {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_IsVMGeneratedMethodIx(env: *mut JNIEnv, cb: jclass, index: ::std::os::raw::c_int) -> jboolean {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxNameUTF(env: *mut JNIEnv, cb: jclass, index: jint) -> *const ::std::os::raw::c_char {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxSignatureUTF(env: *mut JNIEnv, cb: jclass, index: jint) -> *const ::std::os::raw::c_char {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetCPFieldNameUTF(env: *mut JNIEnv, cb: jclass, index: jint) -> *const ::std::os::raw::c_char {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetCPMethodNameUTF(env: *mut JNIEnv, cb: jclass, index: jint) -> *const ::std::os::raw::c_char {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetCPMethodSignatureUTF(env: *mut JNIEnv, cb: jclass, index: jint) -> *const ::std::os::raw::c_char {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetCPFieldSignatureUTF(env: *mut JNIEnv, cb: jclass, index: jint) -> *const ::std::os::raw::c_char {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetCPClassNameUTF(env: *mut JNIEnv, cb: jclass, index: jint) -> *const ::std::os::raw::c_char {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetCPFieldClassNameUTF(env: *mut JNIEnv, cb: jclass, index: jint) -> *const ::std::os::raw::c_char {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetCPMethodClassNameUTF(env: *mut JNIEnv, cb: jclass, index: jint) -> *const ::std::os::raw::c_char {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetCPFieldModifiers(env: *mut JNIEnv, cb: jclass, index: ::std::os::raw::c_int, calledClass: jclass) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetCPMethodModifiers(env: *mut JNIEnv, cb: jclass, index: ::std::os::raw::c_int, calledClass: jclass) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ReleaseUTF(utf: *const ::std::os::raw::c_char) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_IsSameClassPackage(env: *mut JNIEnv, class1: jclass, class2: jclass) -> jboolean {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetLastErrorString(buf: *mut ::std::os::raw::c_char, len: ::std::os::raw::c_int) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_NativePath(arg1: *mut ::std::os::raw::c_char) -> *mut ::std::os::raw::c_char {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_Open(fname: *const ::std::os::raw::c_char, flags: jint, mode: jint) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_Close(fd: jint) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_Read(fd: jint, buf: *mut ::std::os::raw::c_char, nbytes: jint) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_Write(fd: jint, buf: *mut ::std::os::raw::c_char, nbytes: jint) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_Available(fd: jint, pbytes: *mut jlong) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_Lseek(fd: jint, offset: jlong, whence: jint) -> jlong {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_SetLength(fd: jint, length: jlong) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_Sync(fd: jint) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_InitializeSocketLibrary() -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_Socket(domain: jint, type_: jint, protocol: jint) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_SocketClose(fd: jint) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_SocketShutdown(fd: jint, howto: jint) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_Recv(fd: jint, buf: *mut ::std::os::raw::c_char, nBytes: jint, flags: jint) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_Send(fd: jint, buf: *mut ::std::os::raw::c_char, nBytes: jint, flags: jint) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_Timeout(fd: ::std::os::raw::c_int, timeout: ::std::os::raw::c_long) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_Listen(fd: jint, count: jint) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_Connect(fd: jint, him: *mut sockaddr, len: jint) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_Bind(fd: jint, him: *mut sockaddr, len: jint) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_Accept(fd: jint, him: *mut sockaddr, len: *mut jint) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_SocketAvailable(fd: jint, result: *mut jint) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetSockName(fd: jint, him: *mut sockaddr, len: *mut ::std::os::raw::c_int) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetHostName(name: *mut ::std::os::raw::c_char, namelen: ::std::os::raw::c_int) -> ::std::os::raw::c_int {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_RawMonitorCreate() -> *mut ::std::os::raw::c_void {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_RawMonitorDestroy(mon: *mut ::std::os::raw::c_void) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_RawMonitorEnter(mon: *mut ::std::os::raw::c_void) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_RawMonitorExit(mon: *mut ::std::os::raw::c_void) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetManagement(version: jint) -> *mut ::std::os::raw::c_void {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_InitAgentProperties(env: *mut JNIEnv, agent_props: jobject) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetEnclosingMethodInfo(env: *mut JNIEnv, ofClass: jclass) -> jobjectArray {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetThreadStateValues(env: *mut JNIEnv, javaThreadState: jint) -> jintArray {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetThreadStateNames(env: *mut JNIEnv, javaThreadState: jint, values: jintArray) -> jobjectArray {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetVersionInfo(env: *mut JNIEnv, info: *mut jvm_version_info, info_size: usize) {
    (*info).jvm_version = 8;//todo what should I put here?
}

#[no_mangle]
unsafe extern "system" fn JVM_GetTemporaryDirectory(env: *mut JNIEnv) -> jstring {
    unimplemented!()
}


#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxExceptionTableEntry(
    env: *mut JNIEnv,
    cb: jclass,
    method_index: jint,
    entry_index: jint,
    entry: *mut JVM_ExceptionTableEntryType,
) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetMethodIxExceptionIndexes(
    env: *mut JNIEnv,
    cb: jclass,
    method_index: jint,
    exceptions: *mut ::std::os::raw::c_ushort,
) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn jio_vsnprintf(
    str: *mut ::std::os::raw::c_char,
    count: usize,
    fmt: *const ::std::os::raw::c_char,
    args: *mut __va_list_tag,
) -> ::std::os::raw::c_int {
    trace!("JIO Output:");
    vsnprintf(str, count as u64, fmt, args)
}

#[no_mangle]
unsafe extern "system" fn JVM_CopySwapMemory(
    env: *mut JNIEnv,
    srcObj: jobject,
    srcOffset: jlong,
    dstObj: jobject,
    dstOffset: jlong,
    size: jlong,
    elemSize: jlong,
) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_FindClassFromCaller(
    env: *mut JNIEnv,
    c_name: *const ::std::os::raw::c_char,
    init: jboolean,
    loader: jobject,
    caller: jclass,
) -> jclass {
    let state = get_state(env);
    let frame = get_frame(env);

    let name = CStr::from_ptr(&*c_name).to_str().unwrap().to_string();
    to_object(Some(get_or_create_class_object(state, &ClassName::Str(name), frame.clone(), frame.class_pointer.loader.clone())))
}


#[no_mangle]
unsafe extern "system" fn JVM_KnownToNotExist(
    env: *mut JNIEnv,
    loader: jobject,
    classname: *const ::std::os::raw::c_char,
) -> jboolean {
    unimplemented!()
}


#[no_mangle]
unsafe extern "system" fn JVM_GetResourceLookupCacheURLs(env: *mut JNIEnv, loader: jobject) -> jobjectArray {
    unimplemented!()
}


#[no_mangle]
unsafe extern "system" fn JVM_GetResourceLookupCache(
    env: *mut JNIEnv,
    loader: jobject,
    resource_name: *const ::std::os::raw::c_char,
) -> jintArray {
    unimplemented!()
}


#[no_mangle]
unsafe extern "C" fn jio_snprintf(
    str: *mut ::std::os::raw::c_char,
    count: usize,
    fmt: *const ::std::os::raw::c_char,
//    ...
) -> ::std::os::raw::c_int {
    unimplemented!()
}


#[no_mangle]
unsafe extern "C" fn jio_fprintf(
    arg1: *mut FILE,
    fmt: *const ::std::os::raw::c_char,
//    ...
) -> ::std::os::raw::c_int {
    unimplemented!()
}


#[no_mangle]
unsafe extern "system" fn jio_vfprintf(
    arg1: *mut FILE,
    fmt: *const ::std::os::raw::c_char,
    args: *mut __va_list_tag,
) -> ::std::os::raw::c_int {
    unimplemented!()
}


//this ends required symbols
//The following symbols are not needed for linking

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_registerNatives(
    env: *mut JNIEnv,
    cb: jclass) -> () {
    //todo for no register nothing, register later as needed.
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_arrayBaseOffset(env: *mut JNIEnv,
                                                               obj: jobject,
                                                               cb: jclass) -> jint {
    -1//unimplemented but can't return nothing.
}


#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_arrayIndexScale(env: *mut JNIEnv,
                                                               obj: jobject,
                                                               cb: jclass) -> jint {
    -1//unimplemented but can't return nothing.
}


#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_addressSize(env: *mut JNIEnv,
                                                           obj: jobject) -> jint {
    64//officially speaking unimplemented but can't return nothing, and should maybe return something reasonable todo
}

#[no_mangle]
unsafe extern "system" fn Java_sun_reflect_Reflection_getCallerClass(env: *mut JNIEnv,
                                                                     cb: jclass) -> jclass
{
    return JVM_GetCallerClass(env, JVM_CALLER_DEPTH);
}
