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
use slow_interpreter::get_or_create_class_object;
use std::rc::Rc;
use std::intrinsics::transmute;
use slow_interpreter::rust_jni::native_util::{get_state, get_frame, to_object, from_object};
use jni_bindings::{JNIEnv, jclass, jstring, jobject, jlong, jint, jboolean, jobjectArray, jvalue, jbyte, jsize, jbyteArray, jfloat, jdouble, jmethodID, sockaddr, jintArray, jvm_version_info, getc, __va_list_tag, FILE, JVM_ExceptionTableEntryType, vsnprintf, JVM_CALLER_DEPTH};
use log::trace;
use slow_interpreter::interpreter_util::{check_inited_class, push_new_object, run_function};
use slow_interpreter::instructions::ldc::load_class_constant_by_name;
use slow_interpreter::instructions::invoke::{invoke_virtual_method_i, invoke_special};
use classfile_parser::types::MethodDescriptor;
use rust_jvm_common::unified_types::{ParsedType, ClassWithLoader};
use runtime_common::java_values::{JavaValue, Object};
use slow_interpreter::rust_jni::value_conversion::native_to_runtime_class;
use std::sync::Arc;
use std::cell::RefCell;
use runtime_common::runtime_class::RuntimeClass;
use std::thread::Thread;
use slow_interpreter::rust_jni::string::new_string_with_string;
use std::ffi::CStr;
//so in theory I need something like this:
//    asm!(".symver JVM_GetEnclosingMethodInfo JVM_GetEnclosingMethodInfo@@SUNWprivate_1.1");
//but in reality I don't?

#[no_mangle]
unsafe extern "system" fn JVM_GetClassName(env: *mut JNIEnv, cls: jclass) -> jstring {
    let obj = native_to_runtime_class(cls);
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
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_Clone(env: *mut JNIEnv, obj: jobject) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_InternString(env: *mut JNIEnv, str: jstring) -> jstring {
    unimplemented!()
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
unsafe extern "system" fn JVM_InitProperties(env: *mut JNIEnv, p: jobject) -> jobject {
    //todo so in theory I should do stuff here, but not needed for hello world so....
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

#[no_mangle]
unsafe extern "system" fn JVM_FindLibraryEntry(handle: *mut ::std::os::raw::c_void, name: *const ::std::os::raw::c_char) -> *mut ::std::os::raw::c_void {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_IsSupportedJNIVersion(version: jint) -> jboolean {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_IsNaN(d: jdouble) -> jboolean {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_FillInStackTrace(env: *mut JNIEnv, throwable: jobject) {
    unimplemented!()
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
            let runtime_thread_class = native_to_runtime_class(threadClass);
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
    if state.terminate || state.throw {
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
    dbg!(&frame.operand_stack);
    if state.terminate || state.throw {
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
    load_class_constant_by_name(state, &frame, "java/lang/Object".to_string());
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
    unimplemented!()
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
unsafe extern "system" fn JVM_IsPrimitiveClass(env: *mut JNIEnv, cls: jclass) -> jboolean {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetComponentType(env: *mut JNIEnv, cls: jclass) -> jclass {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassModifiers(env: *mut JNIEnv, cls: jclass) -> jint {
    unimplemented!()
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

#[no_mangle]
unsafe extern "system" fn JVM_GetClassDeclaredFields(env: *mut JNIEnv, ofClass: jclass, publicOnly: jboolean) -> jobjectArray {
    let frame = get_frame(env);
    let state = get_state(env);
    frame.print_stack_trace();
    let class_obj = native_to_runtime_class(ofClass);
//    dbg!(&class_obj.clone().unwrap_normal_object().class_pointer);
    let runtime_object = state.class_object_pool.borrow().get(&class_obj).unwrap();
    let field_classfile = check_inited_class(state, &ClassName::Str("java/lang/reflect/Field".to_string()), frame.clone().into(), frame.class_pointer.loader.clone());
    let mut object_array = vec![];
    for f in class_obj.classfile.fields {
        push_new_object(frame.clone(), &field_classfile);
        let field_object = frame.operand_stack.pop();

        object_array.push(field_object)
    }

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
    unimplemented!()
}

fn run_constructor(state: &mut InterpreterState, frame: Rc<StackEntry>, target_classfile: Arc<RuntimeClass>, full_args: Vec<JavaValue>, descriptor: String) {
    target_classfile.classfile.lookup_method("<init>", descriptor)
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassDeclaredConstructors(env: *mut JNIEnv, ofClass: jclass, publicOnly: jboolean) -> jobjectArray {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassAccessFlags(env: *mut JNIEnv, cls: jclass) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_InvokeMethod(env: *mut JNIEnv, method: jobject, obj: jobject, args0: jobjectArray) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_NewInstanceFromConstructor(env: *mut JNIEnv, c: jobject, args0: jobjectArray) -> jobject {
    unimplemented!()
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
    invoke_virtual_method_i(state, frame.clone(), expected_descriptor, runtime_class.clone(), run_method_i, run_method);
//    dbg!(&frame.operand_stack);
//    unimplemented!()
    to_object(frame.pop().unwrap_object())
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
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_RaiseSignal(sig: jint) -> jboolean {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_FindSignal(name: *const ::std::os::raw::c_char) -> jint {
    unimplemented!()
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
    unimplemented!()
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
