#![feature(c_variadic)]
#![feature(box_syntax)]
#![allow(unused_variables)]
#![allow(unreachable_code)]

use std::ffi::{c_char, CStr};
use std::fs::File;
use std::io::{Cursor, Write};
use std::mem::transmute;
use std::ptr::null_mut;
use std::sync::{Arc};
use libc::rand;
use wtf8::Wtf8Buf;
use classfile_view::view::{ClassBackedView, ClassView};
use classfile_view::view::ptype_view::PTypeView;
use jvmti_jni_bindings::{jboolean, jbyte, jchar, jclass, jfieldID, jint, jmethodID, JNI_ERR, JNI_OK, JNIEnv, jobject, jsize, jstring, jvalue};


use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPDType};
use rust_jvm_common::compressed_classfile::field_names::FieldName;
use rust_jvm_common::compressed_classfile::method_names::MethodName;


use rust_jvm_common::FieldId;
use rust_jvm_common::loading::{LoaderName};

use slow_interpreter::better_java_stack::opaque_frame::OpaqueFrame;
use slow_interpreter::exceptions::WasException;
use slow_interpreter::interpreter_util::new_object;
use slow_interpreter::java_values::{ByAddressAllocatedObject, JavaValue};
use slow_interpreter::new_java_values::NewJavaValueHandle;
use slow_interpreter::better_java_stack::frames::PushableFrame;
use slow_interpreter::jvm_state::JVMState;
use slow_interpreter::rust_jni::jni_utils::{get_interpreter_state, get_state, get_throw};
use slow_interpreter::rust_jni::native_util::{from_jclass, from_object, from_object_new, to_object, to_object_new};
use slow_interpreter::stdlib::java::lang::reflect::method::Method;
use slow_interpreter::stdlib::java::lang::string::JString;
use slow_interpreter::stdlib::java::NewAsObjectOrJavaValue;
use slow_interpreter::utils::{field_object_from_view, pushable_frame_todo, throw_npe};
use crate::call::VarargProvider;
use itertools::Itertools;
use classfile_parser::parse_class_file;
use slow_interpreter::define_class_safe::define_class_safe;
use slow_interpreter::new_java_values::owned_casts::OwnedCastAble;


pub mod jni;
pub mod array;
pub mod call;
pub mod exception;
pub mod get_field;
pub mod global_ref;
pub mod instance_of;
pub mod local_frame;
pub mod method;
pub mod misc;
pub mod new_object;
pub mod set_field;
pub mod string;
pub mod util;
pub mod nio;

///MonitorEnter
//
// jint MonitorEnter(JNIEnv *env, jobject obj);
//
// Enters the monitor associated with the underlying Java object referred to by obj.
// Enters the monitor associated with the object referred to by obj. The obj reference must not be NULL.
//
// Each Java object has a monitor associated with it. If the current thread already owns the monitor associated with obj, it increments a counter in the monitor indicating the number of times this thread has entered the monitor. If the monitor associated with obj is not owned by any thread, the current thread becomes the owner of the monitor, setting the entry count of this monitor to 1. If another thread already owns the monitor associated with obj, the current thread waits until the monitor is released, then tries again to gain ownership.
//
// A monitor entered through a MonitorEnter JNI function call cannot be exited using the monitorexit Java virtual machine instruction or a synchronized method return. A MonitorEnter JNI function call and a monitorenter Java virtual machine instruction may race to enter the monitor associated with the same object.
//
// To avoid deadlocks, a monitor entered through a MonitorEnter JNI function call must be exited using the MonitorExit JNI call, unless the DetachCurrentThread call is used to implicitly release JNI monitors.
// LINKAGE:
// Index 217 in the JNIEnv jni_interface function table.
// PARAMETERS:
//
// env: the JNI jni_interface pointer.
//
// obj: a normal Java object or class object.
// RETURNS:
//
// Returns “0” on success; returns a negative value on failure.
pub unsafe extern "C" fn monitor_enter(env: *mut JNIEnv, obj: jobject) -> jint {
    let jvm = get_state(env);
    let interpreter_state = get_interpreter_state(env);
    match from_object(jvm, obj) {
        Some(x) => x,
        None => return JNI_ERR,
    }
        .monitor_lock(jvm, pushable_frame_todo()/*interpreter_state*/);
    JNI_OK as i32
}

///MonitorExit
//
// jint MonitorExit(JNIEnv *env, jobject obj);
//
// The current thread must be the owner of the monitor associated with the underlying Java object referred to by obj. The thread decrements the counter indicating the number of times it has entered this monitor. If the value of the counter becomes zero, the current thread releases the monitor.
//
// Native code must not use MonitorExit to exit a monitor entered through a synchronized method or a monitorenter Java virtual machine instruction.
// LINKAGE:
// Index 218 in the JNIEnv jni_interface function table.
// PARAMETERS:
//
// env: the JNI jni_interface pointer.
//
// obj: a normal Java object or class object.
// RETURNS:
//
// Returns “0” on success; returns a negative value on failure.
// EXCEPTIONS:
//
// IllegalMonitorStateException: if the current thread does not own the monitor.
pub unsafe extern "C" fn monitor_exit(env: *mut JNIEnv, obj: jobject) -> jint {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    match from_object(jvm, obj) {
        Some(x) => x,
        None => return JNI_ERR,
    }
        .monitor_unlock(jvm, pushable_frame_todo()/*int_state*/);
    JNI_OK as i32
}

///GetStringChars
//
// const jchar * GetStringChars(JNIEnv *env, jstring string,
// jboolean *isCopy);
//
// Returns a pointer to the array of Unicode characters of the string. This pointer is valid until ReleaseStringChars() is called.
//
// If isCopy is not NULL, then *isCopy is set to JNI_TRUE if a copy is made; or it is set to JNI_FALSE if no copy is made.
// LINKAGE:
// Index 165 in the JNIEnv jni_interface function table.
// PARAMETERS:
//
// env: the JNI jni_interface pointer.
//
// string: a Java string object.
//
// isCopy: a pointer to a boolean.
// RETURNS:
//
// Returns a pointer to a Unicode string, or NULL if the operation fails.
pub unsafe extern "C" fn get_string_chars(env: *mut JNIEnv, str: jstring, is_copy: *mut jboolean) -> *const jchar {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    *is_copy = u8::from(true);
    let string: JString = match JavaValue::Object(todo!() /*from_jclass(jvm,str)*/).cast_string() {
        None => return throw_npe(jvm, /*int_state*/pushable_frame_todo(),todo!()),
        Some(string) => string,
    };
    let char_vec = string.value(jvm);
    let mut res = null_mut();
    jvm.native.native_interface_allocations.allocate_and_write_vec(char_vec, null_mut(), &mut res as *mut *mut jchar);
    res
}

///AllocObject
//
// jobject AllocObject(JNIEnv *env, jclass clazz);
//
// Allocates a new Java object without invoking any of the constructors for the object. Returns a reference to the object.
//
// The clazz argument must not refer to an array class.
// LINKAGE:
//
// Index 27 in the JNIEnv jni_interface function table.
// PARAMETERS:
//
// env: the JNI jni_interface pointer.
//
// clazz: a Java class object.
// RETURNS:
//
// Returns a Java object, or NULL if the object cannot be constructed.
// THROWS:
//
// InstantiationException: if the class is an jni_interface or an abstract class.
//
// OutOfMemoryError: if the system runs out of memory.
unsafe extern "C" fn alloc_object<'gc, 'l>(env: *mut JNIEnv, clazz: jclass) -> jobject {
    let jvm: &'gc JVMState<'gc> = get_state(env);
    let int_state = get_interpreter_state(env);
    let mut temp: OpaqueFrame<'gc, '_> = todo!();
    let res_object = new_object(jvm, int_state, &from_jclass(jvm, clazz).as_runtime_class(jvm), false).to_jv().unwrap_object();
    to_object(res_object)
}

///ToReflectedMethod
//
// jobject ToReflectedMethod(JNIEnv *env, jclass cls,
//    jmethodID methodID, jboolean isStatic);
//
// Converts a method ID derived from cls to a java.lang.reflect.Method or java.lang.reflect.Constructor object. isStatic must be set to JNI_TRUE if the method ID refers to a static field, and JNI_FALSE otherwise.
//
// Throws OutOfMemoryError and returns 0 if fails.
// LINKAGE:
//
// Index 9 in the JNIEnv jni_interface function table.
// SINCE:
//
// JDK/JRE 1.2
unsafe extern "C" fn to_reflected_method(env: *mut JNIEnv, _cls: jclass, method_id: jmethodID, _is_static: jboolean) -> jobject {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let method_id: usize = transmute(method_id);
    let (runtime_class, index) = match jvm.method_table.read().unwrap().try_lookup(method_id) {
        Some(x) => x,
        None => return null_mut(),
    };
    let runtime_class_view = runtime_class.view();
    let method_view = runtime_class_view.method_view_i(index);
    let method_obj = match Method::method_object_from_method_view(jvm, pushable_frame_todo()/*int_state*/, &method_view) {
        Ok(method_obj) => method_obj,
        Err(_) => todo!(),
    };
    to_object(todo!()/*method_obj.object().to_gc_managed().into()*/)
}

///ExceptionDescribe
//
// void ExceptionDescribe(JNIEnv *env);
//
// Prints an exception and a backtrace of the stack to a system error-reporting channel, such as stderr. This is a convenience routine provided for debugging.
// LINKAGE:
//
// Index 16 in the JNIEnv jni_interface function table.
// PARAMETERS:
//
// env: the JNI jni_interface pointer.
// ExceptionClear
//
// void ExceptionClear(JNIEnv *env);
//
// Clears any exception that is currently being thrown. If no exception is currently being thrown, this routine has no effect.
// LINKAGE:
//
// Index 17 in the JNIEnv jni_interface function table.
// PARAMETERS:
//
// env: the JNI jni_interface pointer.
unsafe extern "C" fn exception_describe(env: *mut JNIEnv) {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    todo!();
    // if let Some(throwing) = int_state.throw() {
    //     int_state.set_throw(None);
    //     todo!()
    //     /*match JavaValue::Object(todo!() /*throwing.into()*/).cast_throwable().print_stack_trace(jvm, int_state) {
    //         Ok(_) => {}
    //         Err(WasException {}) => {}
    //     };*/
    // }
}

///FatalError
//
// void FatalError(JNIEnv *env, const char *msg);
//
// Raises a fatal error and does not expect the VM to recover. This function does not return.
// LINKAGE:
//
// Index 18 in the JNIEnv jni_interface function table.
// PARAMETERS:
//
// env: the JNI jni_interface pointer.
//
// msg: an error message. The string is encoded in modified UTF-8.
// ExceptionCheck
// We introduce a convenience function to check for pending exceptions without creating a local reference to the exception object.
//
// jboolean ExceptionCheck(JNIEnv *env);
//
// Returns JNI_TRUE when there is a pending exception; otherwise, returns JNI_FALSE.
// LINKAGE:
// Index 228 in the JNIEnv jni_interface function table.
// SINCE:
//
// JDK/JRE 1.2
unsafe extern "C" fn fatal_error(_env: *mut JNIEnv, msg: *const ::std::os::raw::c_char) {
    panic!("JNI raised a fatal error.\n JNI MSG: {}", CStr::from_ptr(msg).to_string_lossy())
}

///ThrowNew
//
// jint ThrowNew(JNIEnv *env, jclass clazz,
// const char *message);
//
// Constructs an exception object from the specified class with the message specified by message and causes that exception to be thrown.
// LINKAGE:
//
// Index 14 in the JNIEnv jni_interface function table.
// PARAMETERS:
//
// env: the JNI jni_interface pointer.
//
// clazz: a subclass of java.lang.Throwable.
//
// message: the message used to construct the java.lang.Throwable object. The string is encoded in modified UTF-8.
// RETURNS:
//
// Returns 0 on success; a negative value on failure.
// THROWS:
//
// the newly constructed java.lang.Throwable object.
unsafe extern "C" fn throw_new(env: *mut JNIEnv, clazz: jclass, msg: *const c_char) -> jint {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let (constructor_method_id, java_string_object) = {
        let runtime_class = from_jclass(jvm, clazz).as_runtime_class(jvm);
        let class_view = runtime_class.view();
        let desc = CMethodDescriptor {
            arg_types: vec![CClassName::string().into()],
            return_type: CPDType::VoidType,
        };
        let constructor_method_id = match class_view.lookup_method(MethodName::constructor_init(), &desc) {
            None => return -1,
            Some(constructor) => jvm.method_table.write().unwrap().get_method_id(runtime_class.clone(), constructor.method_i() as u16),
        };
        let rust_string = if msg != null_mut() {
            match CStr::from_ptr(msg).to_str() {
                Ok(string) => string,
                Err(_) => return -2,
            }
                .to_string()
        }else {
            "".to_string()
        };
        let java_string = match JString::from_rust(jvm, int_state, Wtf8Buf::from_string(rust_string)) {
            Ok(java_string) => java_string,
            Err(WasException { exception_obj }) => {
                todo!();
                return -4;
            }
        };
        (constructor_method_id, to_object_new(java_string.full_object_ref().into()))
    };
    let new_object = (**env).NewObjectA.as_ref().unwrap();
    let jvalue_ = jvalue { l: java_string_object };
    let obj = new_object(env, clazz, transmute(constructor_method_id), &jvalue_ as *const jvalue);
    let int_state = get_interpreter_state(env);
    *get_throw(env) = Some(match from_object_new(jvm, obj) {
        None => return -3,
        Some(res) => WasException { exception_obj: res.cast_throwable() },
    }.into());
    JNI_OK as i32
}

unsafe extern "C" fn to_reflected_field(env: *mut JNIEnv, _cls: jclass, field_id: jfieldID, _is_static: jboolean) -> jobject {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);

    let field_id: FieldId = transmute(field_id);
    let (rc, i) = jvm.field_table.write().unwrap().lookup(field_id);
    to_object_new(
        match field_object_from_view(jvm, pushable_frame_todo()/*int_state*/, rc.clone(), rc.view().field(i as usize)) {
            Ok(res) => res,
            Err(_) => todo!(),
        }
            .unwrap_object().as_ref().map(|handle| handle.as_allocated_obj()),
    )
}


unsafe extern "C" fn from_reflected_method(env: *mut JNIEnv, method: jobject) -> jmethodID {
    let jvm = get_state(env);
    let method_obj = JavaValue::Object(todo!() /*from_jclass(jvm,method)*/).cast_method();
    let runtime_class = method_obj.get_clazz(jvm).as_runtime_class(jvm);
    let param_types = method_obj.parameter_types(jvm).iter().map(|param| param.as_runtime_class(jvm).cpdtype()).collect_vec();
    let name_str = method_obj.get_name(jvm).to_rust_string(jvm);
    let name = MethodName(jvm.string_pool.add_name(name_str, false));
    runtime_class
        .clone()
        .view()
        .lookup_method_name(name)
        .iter()
        .find(|candiate_method| candiate_method.desc().arg_types == param_types.iter().map(|from| from.clone()).collect_vec())
        .map(|method| jvm.method_table.write().unwrap().get_method_id(runtime_class.clone(), method.method_i() as u16) as jmethodID)
        .unwrap_or(transmute(-1isize))
}

unsafe extern "C" fn from_reflected_field(env: *mut JNIEnv, method: jobject) -> jfieldID {
    let jvm = get_state(env);
    let field_obj = JavaValue::Object(from_object(jvm, method)).cast_field();
    let runtime_class = field_obj.clazz(jvm).gc_lifeify().as_runtime_class(jvm);
    let field_name = FieldName(jvm.string_pool.add_name(field_obj.name(jvm).to_rust_string(jvm), false));
    runtime_class.view().fields().find(|candidate_field| candidate_field.field_name() == field_name).map(|field| field.field_i()).map(|field_i| jvm.field_table.write().unwrap().get_field_id(runtime_class, field_i as u16) as jfieldID).unwrap_or(transmute(-1isize))
}

unsafe extern "C" fn get_version(_env: *mut JNIEnv) -> jint {
    return 0x00010008;
}


pub unsafe extern "C" fn define_class(env: *mut JNIEnv, name: *const ::std::os::raw::c_char, loader: jobject, buf: *const jbyte, len: jsize) -> jclass {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let _name_string = CStr::from_ptr(name).to_str().unwrap(); //todo unused?
    let loader_name = match from_object_new(jvm, loader) {
        Some(loader_obj) => NewJavaValueHandle::Object(loader_obj).cast_class_loader().to_jvm_loader(jvm),
        None => LoaderName::BootstrapLoader,
    };
    let slice = std::slice::from_raw_parts(buf as *const u8, len as usize);
    let parsed = Arc::new(parse_class_file(&mut Cursor::new(slice)).expect("todo handle invalid"));
    let view = Arc::new(ClassBackedView::from(parsed.clone(), &jvm.string_pool));
    if jvm.config.store_generated_classes {
        File::create(format!("{}{:?}.class", PTypeView::from_compressed(view.name().to_cpdtype(), &jvm.string_pool).class_name_representation(), rand())).unwrap().write_all(slice).unwrap();
    }
    //todo dupe with JVM_DefineClass and JVM_DefineClassWithSource
    to_object_new(
        match define_class_safe(jvm, int_state, parsed.clone(), loader_name, ClassBackedView::from(parsed, &jvm.string_pool)) {
            Ok(class_) => class_,
            Err(_) => todo!(),
        }
            .unwrap_object().unwrap().as_allocated_obj().into(),
    )
}

#[must_use]
pub unsafe fn push_type_to_operand_stack_new<'gc, 'l>(
    jvm: &'gc JVMState<'gc>,
    int_state: &mut impl PushableFrame<'gc>,
    type_: &CPDType,
    l: &mut VarargProvider,
) -> NewJavaValueHandle<'gc> {
    match type_ {
        CPDType::ByteType => {
            let byte_ = l.arg_byte();
            NewJavaValueHandle::Byte(byte_)
        }
        CPDType::CharType => {
            let char_ = l.arg_char();
            NewJavaValueHandle::Char(char_)
        }
        CPDType::DoubleType => {
            let double_ = l.arg_double();
            NewJavaValueHandle::Double(double_)
        }
        CPDType::FloatType => {
            let float_ = l.arg_float();
            NewJavaValueHandle::Float(float_)
        }
        CPDType::IntType => {
            let int: i32 = l.arg_int();
            NewJavaValueHandle::Int(int)
        }
        CPDType::LongType => {
            let long: i64 = l.arg_long();
            NewJavaValueHandle::Long(long)
        }
        CPDType::Class(_) | CPDType::Array { .. } => {
            let native_object: jobject = l.arg_ptr();
            let o = from_object_new(jvm, native_object);
            NewJavaValueHandle::from_optional_object(o)
        }
        CPDType::ShortType => {
            let short = l.arg_short();
            NewJavaValueHandle::Short(short)
        }
        CPDType::BooleanType => {
            let boolean_ = l.arg_bool();
            NewJavaValueHandle::Boolean(boolean_)
        }
        _ => panic!(),
    }
}


pub unsafe fn push_type_to_operand_stack<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, type_: &CPDType, l: &mut VarargProvider) {
    match type_ {
        CPDType::ByteType => {
            let byte_ = l.arg_byte();
            todo!();// int_state.push_current_operand_stack(JavaValue::Byte(byte_))
        }
        CPDType::CharType => {
            let char_ = l.arg_char();
            todo!();// int_state.push_current_operand_stack(JavaValue::Char(char_))
        }
        CPDType::DoubleType => {
            let double_ = l.arg_double();
            todo!();// int_state.push_current_operand_stack(JavaValue::Double(double_))
        }
        CPDType::FloatType => {
            let float_ = l.arg_float();
            todo!();// int_state.push_current_operand_stack(JavaValue::Float(float_))
        }
        CPDType::IntType => {
            let int: i32 = l.arg_int();
            todo!();// int_state.push_current_operand_stack(JavaValue::Int(int))
        }
        CPDType::LongType => {
            let long: i64 = l.arg_long();
            todo!();// int_state.push_current_operand_stack(JavaValue::Long(long))
        }
        CPDType::Class(_) | CPDType::Array { .. } => {
            let native_object: jobject = l.arg_ptr();
            let o = from_object(jvm, native_object);
            todo!();// int_state.push_current_operand_stack(JavaValue::Object(o));
        }
        CPDType::ShortType => {
            let short = l.arg_short();
            todo!();// int_state.push_current_operand_stack(JavaValue::Short(short))
        }
        CPDType::BooleanType => {
            let boolean_ = l.arg_bool();
            todo!();// int_state.push_current_operand_stack(JavaValue::Boolean(boolean_))
        }
        _ => panic!(),
    }
}

