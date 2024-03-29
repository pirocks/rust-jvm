use std::ffi::CString;
use std::mem::{size_of, transmute};
use std::sync::RwLockReadGuard;

use classfile_view::view::ptype_view::PTypeView;
use jvmti_jni_bindings::{jclass, jint, jmethodID, jobject, JVMTI_CLASS_STATUS_ARRAY, JVMTI_CLASS_STATUS_INITIALIZED, JVMTI_CLASS_STATUS_PREPARED, JVMTI_CLASS_STATUS_PRIMITIVE, JVMTI_CLASS_STATUS_VERIFIED, jvmtiEnv, jvmtiError, jvmtiError_JVMTI_ERROR_ABSENT_INFORMATION, jvmtiError_JVMTI_ERROR_INVALID_CLASS, jvmtiError_JVMTI_ERROR_NONE};
use rust_jvm_common::compressed_classfile::compressed_types::CPDType;


use slow_interpreter::class_loading::assert_loaded_class;
use slow_interpreter::class_objects::get_or_create_class_object;
use slow_interpreter::exceptions::WasException;
use slow_interpreter::java_values::JavaValue;
use slow_interpreter::jvm_state::{Classes, JVMState};
use slow_interpreter::rust_jni::jni_utils::new_local_ref_public;
use slow_interpreter::rust_jni::native_util::{from_jclass, from_object, try_from_jclass};
use slow_interpreter::stdlib::java::NewAsObjectOrJavaValue;
use slow_interpreter::utils::pushable_frame_todo;
use crate::universal_error;
use slow_interpreter::rust_jni::jvmti::{get_interpreter_state, get_state};

pub unsafe extern "C" fn get_source_file_name(env: *mut jvmtiEnv, klass: jclass, source_name_ptr: *mut *mut ::std::os::raw::c_char) -> jvmtiError {
    let jvm = get_state(env);
    let tracing_guard = jvm.config.tracing.trace_jdwp_function_enter(jvm, "GetSourceFileName");
    let class_obj = from_jclass(jvm, klass);
    let runtime_class = class_obj.as_runtime_class(jvm);
    let class_view = runtime_class.view();
    let sourcefile = class_view.sourcefile_attr();
    if let Some(file) = sourcefile {
        let wtf8buf = file.file();
        source_name_ptr.write(todo!());
        jvm.config.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
    } else {
        //todo this should validate if info actualy missing in accordance with doc comment
        jvm.config.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_ABSENT_INFORMATION)
    }
}

pub unsafe extern "C" fn get_implemented_interfaces(env: *mut jvmtiEnv, klass: jclass, interface_count_ptr: *mut jint, interfaces_ptr: *mut *mut jclass) -> jvmtiError {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let tracing_guard = jvm.config.tracing.trace_jdwp_function_enter(jvm, "GetImplementedInterfaces");
    let class_obj = from_jclass(jvm, klass);
    let runtime_class = class_obj.as_runtime_class(jvm);
    let class_view = runtime_class.view();
    let num_interfaces = class_view.num_interfaces();
    interface_count_ptr.write(num_interfaces as i32);
    interfaces_ptr.write(libc::calloc(num_interfaces, size_of::<*mut jclass>()) as *mut jclass);
    for (i, interface) in class_view.interfaces().enumerate() {
        let interface_obj = get_or_create_class_object(jvm, interface.interface_name().into(), pushable_frame_todo()/*int_state*/);
        let interface_class = new_local_ref_public(interface_obj.unwrap().to_gc_managed().into(), todo!()/*int_state*/);
        interfaces_ptr.read().add(i).write(interface_class)
    }
    jvm.config.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}

pub unsafe extern "C" fn get_class_status(env: *mut jvmtiEnv, klass: jclass, status_ptr: *mut jint) -> jvmtiError {
    let jvm = get_state(env);
    let tracing_guard = jvm.config.tracing.trace_jdwp_function_enter(jvm, "GetClassStatus");
    let class = from_object(jvm, klass as jobject).unwrap(); //todo handle null
    let res = {
        let type_ = &JavaValue::Object(class.into()).to_new().cast_class().unwrap().as_type(jvm);
        let mut status = 0;
        status |= JVMTI_CLASS_STATUS_PREPARED as i32;
        status |= JVMTI_CLASS_STATUS_VERIFIED as i32;
        status |= JVMTI_CLASS_STATUS_INITIALIZED as i32; //todo so technically this isn't correct, b/c we don't check static intializer completeness
        match type_ {
            CPDType::Class(_) => {}
            CPDType::Array { .. } => {
                status |= JVMTI_CLASS_STATUS_ARRAY as i32;
            }
            _ => {
                status |= JVMTI_CLASS_STATUS_PRIMITIVE as i32;
            }
        };
        status
    };
    status_ptr.write(res);

    //    JVMTI_CLASS_STATUS_VERIFIED	1	Class bytecodes have been verified
    //     JVMTI_CLASS_STATUS_PREPARED	2	Class preparation is complete
    //     JVMTI_CLASS_STATUS_INITIALIZED	4	Class initialization is complete. Static initializer has been run.
    //     JVMTI_CLASS_STATUS_ERROR	8	Error during initialization makes class unusable
    //     JVMTI_CLASS_STATUS_ARRAY	16	Class is an array. If set, all other bits are zero.
    //     JVMTI_CLASS_STATUS_PRIMITIVE	32	Class is a primitive class (for example, java.lang.Integer.TYPE). If set, all other bits are zero.
    //todo actually implement this
    //todo handle primitive classes
    jvm.config.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}

///Get Loaded Classes
///
///     jvmtiError
///     GetLoadedClasses(jvmtiEnv* env,
///                 jint* class_count_ptr,
///                 jclass** classes_ptr)
///
/// Return an array of all classes loaded in the virtual machine.
/// The number of classes in the array is returned via class_count_ptr, and the array itself via classes_ptr.
///
/// Array classes of all types (including arrays of primitive types) are included in the returned list.
/// Primitive classes (for example, java.lang.Integer.TYPE) are not included in this list.
///
/// Phase	Callback Safe	Position	Since
/// may only be called during the live phase 	No 	78	1.0
///
/// Capabilities
/// Required Functionality
///
/// Parameters
/// Name 	Type 	Description
/// class_count_ptr	jint*	On return, points to the number of classes.
///
/// Agent passes a pointer to a jint. On return, the jint has been set.
/// classes_ptr	jclass**	On return, points to an array of references, one for each class.
///
/// Agent passes a pointer to a jclass*. On return, the jclass* points to a newly allocated array of size *class_count_ptr.
/// The array should be freed with Deallocate. The objects returned by classes_ptr are JNI local references and must be managed.
///
/// Errors
/// This function returns either a universal error or one of the following errors
/// Error 	Description
/// JVMTI_ERROR_NULL_POINTER	class_count_ptr is NULL.
/// JVMTI_ERROR_NULL_POINTER	classes_ptr is NULL.
pub unsafe extern "C" fn get_loaded_classes<'gc, 'l>(env: *mut jvmtiEnv, class_count_ptr: *mut jint, classes_ptr: *mut *mut jclass) -> jvmtiError {
    let jvm: &'gc JVMState<'gc> = get_state(env);
    let int_state = get_interpreter_state(env);
    let tracing_guard = jvm.config.tracing.trace_jdwp_function_enter(jvm, "GetLoadedClasses");
    let mut res_vec = vec![];

    let classes: RwLockReadGuard<'_, Classes<'gc>> = jvm.classes.read().unwrap();
    let collected = classes.get_loaded_classes();
    for (_loader, ptype) in collected {
        let class_object = get_or_create_class_object(jvm, ptype.clone(), pushable_frame_todo()/*int_state*/);
        res_vec.push(new_local_ref_public(class_object.unwrap().to_gc_managed().into(), todo!()/*int_state*/))
    }
    class_count_ptr.write(res_vec.len() as i32);
    classes_ptr.write(transmute(Vec::leak(res_vec).as_mut_ptr())); //todo leaking
    jvm.config.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}

pub unsafe extern "C" fn get_class_signature(env: *mut jvmtiEnv, klass: jclass, signature_ptr: *mut *mut ::std::os::raw::c_char, generic_ptr: *mut *mut ::std::os::raw::c_char) -> jvmtiError {
    let jvm = get_state(env);
    let tracing_guard = jvm.config.tracing.trace_jdwp_function_enter(jvm, "GetClassSignature");
    let notnull_class = from_object(jvm, klass as jobject).unwrap(); //todo handle npe
    let class_object_ptype = JavaValue::Object(todo!() /*notnull_class.into()*/).to_new().cast_class().unwrap().as_type(jvm);
    let type_ = class_object_ptype;
    if !signature_ptr.is_null() {
        let jvm_repr = CString::new(PTypeView::from_compressed(type_, &jvm.string_pool).jvm_representation()).unwrap();
        let jvm_repr_ptr = jvm_repr.into_raw();
        let allocated_jvm_repr = libc::malloc(libc::strlen(jvm_repr_ptr) + 1) as *mut ::std::os::raw::c_char;
        signature_ptr.write(allocated_jvm_repr);
        libc::strcpy(allocated_jvm_repr, jvm_repr_ptr);
    }
    if !generic_ptr.is_null() {
        todo!();
        let java_repr = CString::new(PTypeView::from_compressed(type_, &jvm.string_pool).java_source_representation()).unwrap();
        let java_repr_ptr = dbg!(java_repr).into_raw();
        let allocated_java_repr = libc::malloc(libc::strlen(java_repr_ptr) + 1) as *mut ::std::os::raw::c_char;
        generic_ptr.write(allocated_java_repr);
        libc::strcpy(allocated_java_repr, java_repr_ptr);
    }
    jvm.config.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}

///Get Class Methods
///
///     jvmtiError
///     GetClassMethods(jvmtiEnv* env,
///                 jclass klass,
///                 jint* method_count_ptr,
///                 jmethodID** methods_ptr)
///
/// For the class indicated by klass, return a count of methods via method_count_ptr and a list of method IDs via methods_ptr. The method list contains constructors and static initializers as well as true methods. Only directly declared methods are returned (not inherited methods). An empty method list is returned for array classes and primitive classes (for example, java.lang.Integer.TYPE).
///
/// Phase	Callback Safe	Position	Since
/// may only be called during the start or the live phase 	No 	52	1.0
///
/// Capabilities
/// Required Functionality
/// Optional Features
/// Capability 	Effect
/// can_maintain_original_method_order	Can return methods in the order they occur in the class file
///
/// Parameters
/// Name 	Type 	Description
/// klass	jclass	The class to query.
/// method_count_ptr	jint*	On return, points to the number of methods declared in this class.
///
/// Agent passes a pointer to a jint. On return, the jint has been set.
/// methods_ptr	jmethodID**	On return, points to the method ID array.
///
/// Agent passes a pointer to a jmethodID*. On return, the jmethodID* points to a newly allocated array of size *method_count_ptr. The array should be freed with Deallocate.
///
/// Errors
/// This function returns either a universal error or one of the following errors
/// Error 	Description
/// JVMTI_ERROR_CLASS_NOT_PREPARED	klass is not prepared. //todo handle this instead of loading the class
/// JVMTI_ERROR_INVALID_CLASS	klass is not a class object or the class has been unloaded.
/// JVMTI_ERROR_NULL_POINTER	method_count_ptr is NULL.
/// JVMTI_ERROR_NULL_POINTER	methods_ptr is NULL.
pub unsafe extern "C" fn get_class_methods(env: *mut jvmtiEnv, klass: jclass, method_count_ptr: *mut jint, methods_ptr: *mut *mut jmethodID) -> jvmtiError {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    assert!(jvm.vm_live());
    //todo capabilities
    let tracing_guard = jvm.config.tracing.trace_jdwp_function_enter(jvm, "GetClassMethods");
    let class = match try_from_jclass(jvm, klass as jobject) {
        None => {
            return jvm.config.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_INVALID_CLASS);
        }
        Some(class) => class,
    };
    null_check!(method_count_ptr);
    null_check!(methods_ptr);
    let class_type = class.as_type(jvm);
    let loaded_class = assert_loaded_class(jvm, class_type);
    let res = loaded_class
        .view()
        .methods()
        .map(|mv| {
            let method_id = jvm.method_table.write().unwrap().get_method_id(loaded_class.clone(), mv.method_i() as u16);
            method_id as jmethodID
        })
        .collect::<Vec<_>>();
    jvm.native.native_interface_allocations.allocate_and_write_vec(res, method_count_ptr, methods_ptr);
    jvm.config.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}

pub unsafe extern "C" fn get_class_loader(env: *mut jvmtiEnv, klass: jclass, classloader_ptr: *mut jobject) -> jvmtiError {
    // assert_eq!(classloader_ptr, std::ptr::null_mut());//only implement bootstrap loader case
    let jvm = get_state(env);
    let class = from_jclass(jvm, klass as jobject);
    let int_state = get_interpreter_state(env);
    let class_loader = match class.get_class_loader(jvm, pushable_frame_todo()/*int_state*/) {
        Ok(class_loader) => class_loader,
        Err(WasException { exception_obj }) => {
            todo!();
            return universal_error();
        }
    };
    let jobject_ = new_local_ref_public(class_loader.map(|cl| cl.object().to_gc_managed()), todo!()/*int_state*/);
    classloader_ptr.write(jobject_);
    jvmtiError_JVMTI_ERROR_NONE
}