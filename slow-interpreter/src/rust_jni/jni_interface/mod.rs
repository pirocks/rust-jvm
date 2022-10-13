use std::collections::HashMap;
use std::ffi::CStr;
use std::fs::File;
use std::io::{Cursor, Write};
use std::mem::transmute;
use std::ops::Deref;
use std::ptr::null_mut;
use std::sync::{Arc, Mutex, RwLock};

use by_address::ByAddress;
use itertools::Itertools;
use libc::rand;
use wtf8::Wtf8Buf;

use classfile_parser::parse_class_file;
use classfile_view::view::{ClassBackedView, ClassView, HasAccessFlags};
use classfile_view::view::field_view::FieldView;
use classfile_view::view::ptype_view::PTypeView;
use java5_verifier::type_infer;
use jvmti_jni_bindings::{jboolean, jbyte, jchar, jclass, jfieldID, jint, jmethodID, JmmInterface, jmmInterface_1_, JNI_ERR, JNI_OK, JNIEnv, JNIInvokeInterface_, JNINativeInterface_, jobject, jsize, jstring, jvalue, jvmtiInterface_1_};
use runtime_class_stuff::{ClassStatus, RuntimeClass, RuntimeClassClass};
use rust_jvm_common::classfile::Classfile;
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPDType};
use rust_jvm_common::compressed_classfile::field_names::FieldName;
use rust_jvm_common::compressed_classfile::method_names::MethodName;


use rust_jvm_common::descriptor_parser::parse_field_descriptor;
use rust_jvm_common::FieldId;
use rust_jvm_common::loading::{ClassLoadingError, ClassWithLoader, LoaderName};
use stage0::compiler_common::frame_data::SunkVerifierFrames;
use verification::{VerifierContext, verify};
use verification::verifier::TypeSafetyError;

use crate::{JVMState, NewAsObjectOrJavaValue, PushableFrame};
use crate::better_java_stack::opaque_frame::OpaqueFrame;
use crate::class_loading::{check_initing_or_inited_class, ClassIntrinsicsData, create_class_object, get_static_var_types};
use crate::class_objects::get_or_create_class_object_force_loader;
use crate::exceptions::WasException;
use crate::interpreter::common::ldc::load_class_constant_by_type;
use crate::interpreter_util::new_object;
use crate::java_values::{ByAddressAllocatedObject, JavaValue};
use crate::new_java_values::NewJavaValueHandle;
use crate::runtime_class::{initialize_class, prepare_class, static_vars};
use crate::rust_jni::invoke_interface::get_env;
use crate::rust_jni::jni_interface::call::VarargProvider;
use crate::rust_jni::jni_interface::jmm::initial_jmm;
use crate::rust_jni::jni_interface::jni::{get_interpreter_state, get_state, initial_jni_interface};
use crate::rust_jni::jvmti_interface::initial_jvmti;
use crate::rust_jni::native_util::{from_jclass, from_object, from_object_new, to_object, to_object_new};
use crate::stdlib::java::lang::class::JClass;
use crate::stdlib::java::lang::class_not_found_exception::ClassNotFoundException;
use crate::stdlib::java::lang::reflect::field::Field;
use crate::stdlib::java::lang::reflect::method::Method;
use crate::stdlib::java::lang::string::JString;
use crate::utils::{pushable_frame_todo, throw_npe};

pub struct PerStackInterfaces {
    jni: JNINativeInterface_,
    jmm: jmmInterface_1_,
    jvmti: jvmtiInterface_1_,
    invoke_interface: JNIInvokeInterface_,
}

impl PerStackInterfaces {
    pub fn new() -> Self {
        Self {
            jni: initial_jni_interface(),
            jmm: initial_jmm(),
            jvmti: initial_jvmti(),
            invoke_interface: initial_invoke_interface(),
        }
    }

    pub fn jni_inner_mut(&mut self) -> &mut JNINativeInterface_ {
        &mut self.jni
    }

    pub fn jmm_inner_mut(&mut self) -> &mut JmmInterface {
        &mut self.jmm
    }

    pub fn jvmti_inner_mut(&mut self) -> &mut jvmtiInterface_1_ {
        &mut self.jvmti
    }

    pub fn invoke_interface_mut(&mut self) -> &mut JNIInvokeInterface_ {
        &mut self.invoke_interface
    }
}

fn initial_invoke_interface() -> JNIInvokeInterface_ {
    JNIInvokeInterface_ {
        reserved0: null_mut(),
        reserved1: null_mut(),
        reserved2: null_mut(),
        DestroyJavaVM: None,
        AttachCurrentThread: None,
        DetachCurrentThread: None,
        GetEnv: Some(get_env),
        AttachCurrentThreadAsDaemon: None,
    }
}

pub mod jmm;
pub mod jvmti;
pub mod jni;

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
        None => return throw_npe(jvm, /*int_state*/pushable_frame_todo()),
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
unsafe extern "C" fn throw_new(env: *mut JNIEnv, clazz: jclass, msg: *const ::std::os::raw::c_char) -> jint {
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
        let rust_string = match CStr::from_ptr(msg).to_str() {
            Ok(string) => string,
            Err(_) => return -2,
        }
            .to_string();
        let java_string = match JString::from_rust(jvm, pushable_frame_todo()/*int_state*/, Wtf8Buf::from_string(rust_string)) {
            Ok(java_string) => java_string,
            Err(WasException { exception_obj }) => {
                todo!();
                return -4;
            }
        };
        (constructor_method_id, to_object(todo!()/*java_string.object().to_gc_managed().into()*/))
    };
    let new_object = (**env).NewObjectA.as_ref().unwrap();
    let jvalue_ = jvalue { l: java_string_object };
    let obj = new_object(env, clazz, transmute(constructor_method_id), &jvalue_ as *const jvalue);
    let int_state = get_interpreter_state(env);
    todo!();/*int_state.set_throw(
        Some(match from_object_new(jvm, obj) {
            None => return -3,
            Some(res) => res,
        }
            .into()),
    );*/
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

//shouldn't take class as arg and should be an impl method on Field
pub fn field_object_from_view<'gc, 'l>(
    jvm: &'gc JVMState<'gc>,
    int_state: &mut impl PushableFrame<'gc>,
    class_obj: Arc<RuntimeClass<'gc>>,
    f: FieldView,
) -> Result<NewJavaValueHandle<'gc>, WasException<'gc>> {
    let field_class_name_ = class_obj.clone().cpdtype();
    let parent_runtime_class = load_class_constant_by_type(jvm, int_state, field_class_name_)?;

    let field_name = f.field_name();

    let field_desc_str = f.field_desc();
    let field_type = parse_field_descriptor(field_desc_str.as_str()).unwrap().field_type;

    let signature = f.signature_attribute();

    let modifiers = f.access_flags() as i32;
    let slot = f.field_i() as i32;
    let clazz = parent_runtime_class.cast_class().expect("todo");
    let field_name_str = field_name.0.to_str(&jvm.string_pool);
    let name = JString::from_rust(jvm, int_state, Wtf8Buf::from_string(field_name_str))?.intern(jvm, int_state)?;
    let type_ = JClass::from_type(jvm, int_state, CPDType::from_ptype(&field_type, &jvm.string_pool))?;
    let signature = match signature {
        None => None,
        Some(signature) => Some(JString::from_rust(jvm, int_state, signature)?),
    };

    let annotations_ = vec![]; //todo impl annotations.

    Ok(Field::init(jvm, int_state, clazz, name, type_, modifiers, slot, signature, annotations_)?.new_java_value_handle())
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

pub fn define_class_safe<'gc, 'l>(
    jvm: &'gc JVMState<'gc>,
    int_state: &mut impl PushableFrame<'gc>,
    parsed: Arc<Classfile>,
    current_loader: LoaderName,
    class_view: ClassBackedView,
) -> Result<NewJavaValueHandle<'gc>, WasException<'gc>> {
    let class_name = class_view.name().unwrap_name();
    let class_view = Arc::new(class_view);
    let super_class = class_view.super_name().map(|name| check_initing_or_inited_class(jvm, int_state, name.into()).unwrap());
    let interfaces = class_view.interfaces().map(|interface| check_initing_or_inited_class(jvm, int_state, interface.interface_name().into()).unwrap()).collect_vec();
    let static_var_types = get_static_var_types(class_view.deref());
    let runtime_class = Arc::new(RuntimeClass::Object(
        RuntimeClassClass::new_new(&jvm.inheritance_tree, &mut jvm.bit_vec_paths.write().unwrap(), class_view.clone(), super_class, interfaces, RwLock::new(ClassStatus::UNPREPARED), &jvm.string_pool, &jvm.class_ids)
    ));
    jvm.classpath.class_cache.write().unwrap().insert(class_view.name().unwrap_name(), parsed.clone());
    let mut class_view_cache = HashMap::new();
    class_view_cache.insert(ClassWithLoader { class_name, loader: current_loader }, class_view.clone() as Arc<dyn ClassView>);
    let mut vf = VerifierContext {
        live_pool_getter: jvm.get_live_object_pool_getter(),
        classfile_getter: jvm.get_class_getter(int_state.current_loader(jvm)),
        string_pool: &jvm.string_pool,
        current_class: class_name,
        class_view_cache: Mutex::new(class_view_cache),
        current_loader: LoaderName::BootstrapLoader, //todo
        verification_types: Default::default(),
        debug: false,
        perf_metrics: &jvm.perf_metrics,
        permissive_types_workaround: false,
    };
    match verify(&mut vf, class_name, LoaderName::BootstrapLoader /*todo*/) {
        Ok(_) => {
            jvm.sink_function_verification_date(&vf.verification_types, runtime_class.clone());
        }
        Err(TypeSafetyError::ClassNotFound(ClassLoadingError::ClassNotFoundException(class_name))) => {
            dbg!(&class_name);
            let class = JString::from_rust(jvm, pushable_frame_todo(), Wtf8Buf::from_str(class_name.get_referred_name()))?;
            let to_throw = ClassNotFoundException::new(jvm, int_state, class)?.object().new_java_handle().unwrap_object().unwrap();
            todo!();// int_state.set_throw(Some(to_throw));
            return Err(WasException { exception_obj: todo!() });
        }
        Err(TypeSafetyError::NotSafe(msg)) => {
            dbg!(&msg);
            panic!()
        }
        Err(TypeSafetyError::Java5Maybe) => {
            //todo check for privileged here
            for method_view in class_view.methods() {
                let method_id = jvm.method_table.write().unwrap().get_method_id(runtime_class.clone(), method_view.method_i());
                let code = method_view.code_attribute().unwrap();
                let instructs = code.instructions.iter().sorted_by_key(|(offset, instruct)| *offset).map(|(_, instruct)| instruct.clone()).collect_vec();
                let res = type_infer(&method_view);
                let frames_tops = res.inferred_frames().iter().map(|(offset, frame)| {
                    (*offset, SunkVerifierFrames::PartialInferredFrame(frame.clone()))
                }).collect::<HashMap<_, _>>();
                let frames_no_tops = res.inferred_frames().iter().map(|(offset, frame)| {
                    (*offset, SunkVerifierFrames::PartialInferredFrame(frame.no_tops()))
                }).collect::<HashMap<_, _>>();
                jvm.function_frame_type_data.write().unwrap().no_tops.insert(method_id, frames_no_tops);
                jvm.function_frame_type_data.write().unwrap().tops.insert(method_id, frames_tops);
            }
        }
        Err(TypeSafetyError::ClassNotFound(ClassLoadingError::ClassFileInvalid(_))) => panic!(),
        Err(TypeSafetyError::ClassNotFound(ClassLoadingError::ClassVerificationError)) => panic!(),
    };
    let class_object = create_class_object(jvm, int_state, None, current_loader,ClassIntrinsicsData{
        is_array: false,
        is_primitive: false,
        component_type: None,
        this_cpdtype: class_name.into()
    })?;
    let mut classes = jvm.classes.write().unwrap();
    classes.anon_classes.push(runtime_class.clone());
    classes.initiating_loaders.insert(class_name.clone().into(), (current_loader, runtime_class.clone()));
    classes.loaded_classes_by_type.entry(current_loader).or_insert(HashMap::new()).insert(class_name.clone().into(), runtime_class.clone());
    classes.class_object_pool.insert(ByAddressAllocatedObject::Owned(class_object.duplicate_discouraged()), ByAddress(runtime_class.clone()));
    drop(classes);
    assert_eq!(class_object.runtime_class(jvm).cpdtype(), CClassName::class().into());
    prepare_class(jvm, int_state, Arc::new(ClassBackedView::from(parsed.clone(), &jvm.string_pool)), &mut static_vars(runtime_class.deref(), jvm));
    runtime_class.set_status(ClassStatus::PREPARED);
    runtime_class.set_status(ClassStatus::INITIALIZING);
    initialize_class(runtime_class.clone(), jvm, int_state)?;
    runtime_class.set_status(ClassStatus::INITIALIZED);
    Ok(get_or_create_class_object_force_loader(jvm, class_name.into(), int_state, current_loader).unwrap().new_java_handle())
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
pub(crate) unsafe fn push_type_to_operand_stack_new<'gc, 'l>(
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


pub(crate) unsafe fn push_type_to_operand_stack<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, type_: &CPDType, l: &mut VarargProvider) {
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