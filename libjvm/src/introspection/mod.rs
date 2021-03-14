use std::borrow::Borrow;
use std::cell::{RefCell, UnsafeCell};
use std::ffi::CStr;
use std::ops::Deref;
use std::ptr::null_mut;
use std::sync::Arc;

use by_address::ByAddress;
use num_cpus::get;

use classfile_view::loading::{ClassLoadingError, LoaderName};
use classfile_view::view::{ClassView, HasAccessFlags};
use classfile_view::view::attribute_view::InnerClassesView;
use classfile_view::view::method_view::MethodView;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use jvmti_jni_bindings::{jboolean, jbyteArray, jclass, jint, jio_vfprintf, JNIEnv, jobject, jobjectArray, jstring, JVM_ExceptionTableEntryType, jvmtiCapabilities};
use rust_jvm_common::classfile::{ACC_ABSTRACT, ACC_PUBLIC};
use rust_jvm_common::classnames::{class_name, ClassName};
use rust_jvm_common::ptype::{PType, ReferenceType};
use slow_interpreter::class_loading::check_initing_or_inited_class;
use slow_interpreter::class_objects::{get_or_create_class_object, get_or_create_class_object_force_loader};
use slow_interpreter::instructions::ldc::{create_string_on_stack, load_class_constant_by_type};
use slow_interpreter::interpreter::WasException;
use slow_interpreter::interpreter_util::{push_new_object, run_constructor};
use slow_interpreter::java::lang::class::JClass;
use slow_interpreter::java::lang::class_not_found_exception::ClassNotFoundException;
use slow_interpreter::java::lang::string::JString;
use slow_interpreter::java_values::{ArrayObject, JavaValue, Object};
use slow_interpreter::java_values::Object::Array;
use slow_interpreter::runtime_class::RuntimeClass;
use slow_interpreter::rust_jni::interface::local_frame::new_local_ref_public;
use slow_interpreter::rust_jni::interface::string::new_string_with_string;
use slow_interpreter::rust_jni::interface::util::class_object_to_runtime_class;
use slow_interpreter::rust_jni::native_util::{from_jclass, from_object, get_interpreter_state, get_state, to_object};
use slow_interpreter::sun::reflect::reflection::Reflection;
use slow_interpreter::threading::JavaThread;
use slow_interpreter::threading::monitors::Monitor;

pub mod constant_pool;
pub mod is_x;
pub mod index;
pub mod method_annotations;

#[no_mangle]
unsafe extern "system" fn JVM_GetClassInterfaces(env: *mut JNIEnv, cls: jclass) -> jobjectArray {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let interface_vec = match from_jclass(cls).as_runtime_class(jvm).view().interfaces().map(|interface| {
        let class_obj = get_or_create_class_object(jvm, interface.interface_name().into(), int_state)?;
        Ok(JavaValue::Object(Some(class_obj)))
    }).collect::<Result<Vec<_>, WasException>>() {
        Ok(interface_vec) => interface_vec,
        Err(WasException {}) => {
            return null_mut();
        }
    };
    let res = Some(Arc::new(Array(ArrayObject::new_array(jvm, int_state, interface_vec, ClassName::class().into(), jvm.thread_state.new_monitor("".to_string())))));
    new_local_ref_public(res, int_state)
}


#[no_mangle]
unsafe extern "system" fn JVM_GetClassSigners(env: *mut JNIEnv, cls: jclass) -> jobjectArray {
    null_mut()// not supporting class signing atm.
}

#[no_mangle]
unsafe extern "system" fn JVM_GetProtectionDomain(env: *mut JNIEnv, cls: jclass) -> jobject {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let class = from_jclass(cls).as_runtime_class(jvm);
    match jvm.protection_domains.read().unwrap().get_by_left(&ByAddress(class)) {
        None => null_mut(),
        Some(pd_obj) => {
            new_local_ref_public(pd_obj.clone().0.into(), int_state)
        }
    }
}


#[no_mangle]
unsafe extern "system" fn JVM_GetComponentType(env: *mut JNIEnv, cls: jclass) -> jclass {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let object = from_object(cls);
    let temp = JavaValue::Object(object).cast_class().as_type(jvm);
    let object_class = temp.unwrap_ref_type();
    new_local_ref_public(match JClass::from_type(jvm, int_state, object_class.unwrap_array()) {
        Ok(jclass) => jclass,
        Err(WasException {}) => return null_mut()
    }.java_value().unwrap_object(), int_state)
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassModifiers(env: *mut JNIEnv, cls: jclass) -> jint {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let jclass = from_jclass(cls);
    jclass.as_runtime_class(jvm).view().access_flags() as jint
}

#[no_mangle]
unsafe extern "system" fn JVM_GetDeclaredClasses(env: *mut JNIEnv, ofClass: jclass) -> jobjectArray {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let class = from_jclass(ofClass).as_runtime_class(jvm);
    let res_array = match class.view().inner_classes_view() {
        None => vec![],
        Some(inner_classes) => {
            inner_classes.classes().flat_map(|inner_class| Some(PTypeView::Ref(inner_class.inner_name()?))).collect::<Vec<_>>()
        }
    }.into_iter().map(|ptype| Ok(JavaValue::Object(get_or_create_class_object(jvm, ptype, int_state)?.into()))).collect::<Result<Vec<_>, _>>();
    let res_jv = JavaValue::new_vec_from_vec(jvm, match res_array {
        Ok(obj_array) => obj_array,
        Err(WasException {}) => return null_mut(),
    }, ClassName::class().into());
    new_local_ref_public(res_jv.unwrap_object(), int_state)
}

#[no_mangle]
unsafe extern "system" fn JVM_GetDeclaringClass(env: *mut JNIEnv, ofClass: jclass) -> jclass {
    //todo unimplemented for now
    std::ptr::null_mut()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassSignature(env: *mut JNIEnv, cls: jclass) -> jstring {
    unimplemented!()
}

pub mod get_methods;

#[no_mangle]
unsafe extern "system" fn JVM_GetClassAccessFlags(env: *mut JNIEnv, cls: jclass) -> jint {
    let jvm = get_state(env);
    let res = from_jclass(cls).as_runtime_class(jvm).view().access_flags() as i32;
    res
}


#[no_mangle]
unsafe extern "system" fn JVM_ClassDepth(env: *mut JNIEnv, name: jstring) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassContext(env: *mut JNIEnv) -> jobjectArray {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassNameUTF(env: *mut JNIEnv, cb: jclass) -> *const ::std::os::raw::c_char {
    unimplemented!()
}

pub mod fields;
pub mod methods;


#[no_mangle]
pub unsafe extern "system" fn JVM_GetCallerClass(env: *mut JNIEnv, depth: ::std::os::raw::c_int) -> jclass {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let mut stack = int_state.cloned_stack_snapshot().into_iter().rev();
    stack.next();
    stack.next();
    let possibly_class_pointer = stack.find_map(|entry| {
        let class_pointer = entry.try_class_pointer()?;
        let view = class_pointer.view();
        if entry.is_native() || entry.is_opaque_frame() {
            return None;
        }
        if let Some(name) = view.name().try_unwrap_name() {
            if name == ClassName::method() && view.method_view_i(entry.method_i() as usize).name() == "invoke" {
                return None;
            }
        }
        Some(class_pointer.clone())
    });
    let type_ = if let Some(class_pointer) = possibly_class_pointer {
        class_pointer.ptypeview()
    } else {
        return null_mut();
    };
    load_class_constant_by_type(jvm, int_state, type_);
    let jclass = int_state.pop_current_operand_stack().unwrap_object();
    new_local_ref_public(jclass, int_state)
}


#[no_mangle]
unsafe extern "system" fn JVM_IsSameClassPackage(env: *mut JNIEnv, class1: jclass, class2: jclass) -> jboolean {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    match Reflection::is_same_class_package(jvm, int_state, from_jclass(class1), from_jclass(class2)) {
        Ok(res) => res,
        Err(WasException {}) => return jboolean::MAX
    }
}


#[no_mangle]
unsafe extern "system" fn JVM_FindClassFromCaller(
    env: *mut JNIEnv,
    c_name: *const ::std::os::raw::c_char,
    init: jboolean,
    loader: jobject,
    caller: jclass,
) -> jclass {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let name = CStr::from_ptr(&*c_name).to_str().unwrap().to_string();
    let p_type = PTypeView::Ref(ReferenceTypeView::Class(ClassName::Str(name.clone())));

    let loader_name = from_object(loader)
        .map(|loader_obj| JavaValue::Object(loader_obj.into()).cast_class_loader().to_jvm_loader(jvm)).unwrap_or(LoaderName::BootstrapLoader);

    let class_lookup_result = get_or_create_class_object_force_loader(
        jvm,
        p_type.clone(),
        int_state,
        loader_name,
    );
    match class_lookup_result {
        Ok(class_object) => {
            if init != 0 {
                if let Err(WasException {}) = check_initing_or_inited_class(jvm, int_state, p_type) {
                    return null_mut();
                };
            }
            new_local_ref_public(Some(class_object), int_state)
        }
        Err(WasException {}) => {
            null_mut()
        }
    }
}


#[no_mangle]
unsafe extern "system" fn JVM_GetClassName(env: *mut JNIEnv, cls: jclass) -> jstring {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let obj = from_jclass(cls).as_runtime_class(jvm);
    let full_name = &obj.ptypeview().class_name_representation();
    new_string_with_string(env, full_name.to_string())
}

