use std::borrow::Borrow;
use std::cell::{RefCell, UnsafeCell};
use std::ffi::{c_void, CStr};
use std::ops::Deref;
use std::os::raw::c_char;
use std::ptr::null_mut;

use by_address::ByAddress;
use itertools::Itertools;
use num_cpus::get;
use wtf8::Wtf8Buf;

use another_jit_vm_ir::WasException;
use classfile_view::view::{ClassView, HasAccessFlags};
use classfile_view::view::attribute_view::InnerClassesView;
use classfile_view::view::method_view::MethodView;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use jvmti_jni_bindings::{jboolean, jbyteArray, jclass, jint, jio_vfprintf, JNIEnv, jobject, jobjectArray, jstring, JVM_ExceptionTableEntryType, jvmtiCapabilities};
use rust_jvm_common::classfile::{ACC_ABSTRACT, ACC_PUBLIC};
use rust_jvm_common::classnames::{class_name, ClassName};
use rust_jvm_common::compressed_classfile::{CompressedParsedRefType, CPDType, CPRefType};
use rust_jvm_common::compressed_classfile::names::{CClassName, CompressedClassName, MethodName};
use rust_jvm_common::loading::{ClassLoadingError, LoaderName};
use rust_jvm_common::ptype::{PType, ReferenceType};
use sketch_jvm_version_of_utf8::JVMString;
use slow_interpreter::class_loading::{assert_inited_or_initing_class, check_initing_or_inited_class};
use slow_interpreter::class_objects::{get_or_create_class_object, get_or_create_class_object_force_loader};
use slow_interpreter::instructions::ldc::{create_string_on_stack, load_class_constant_by_type};
use slow_interpreter::interpreter_util::{new_object, run_constructor};
use slow_interpreter::java::lang::class::JClass;
use slow_interpreter::java::lang::class_not_found_exception::ClassNotFoundException;
use slow_interpreter::java::lang::string::JString;
use slow_interpreter::java::NewAsObjectOrJavaValue;
use slow_interpreter::java_values::{ArrayObject, JavaValue, Object};
use slow_interpreter::java_values::Object::Array;
use slow_interpreter::new_java_values::{NewJavaValue, NewJavaValueHandle};
use slow_interpreter::new_java_values::java_value_common::JavaValueCommon;
use slow_interpreter::new_java_values::unallocated_objects::{UnAllocatedObject, UnAllocatedObjectArray};
use slow_interpreter::rust_jni::interface::local_frame::{new_local_ref_public, new_local_ref_public_new};
use slow_interpreter::rust_jni::interface::string::new_string_with_string;
use slow_interpreter::rust_jni::interface::util::class_object_to_runtime_class;
use slow_interpreter::rust_jni::native_util::{from_jclass, from_object, from_object_new, get_interpreter_state, get_state, to_object, to_object_new};
use slow_interpreter::rust_jni::value_conversion::native_to_runtime_class;
use slow_interpreter::sun::reflect::reflection::Reflection;
use slow_interpreter::threading::JavaThread;
use slow_interpreter::threading::monitors::Monitor;
use slow_interpreter::utils::throw_npe;

pub mod constant_pool;
pub mod is_x;
pub mod get_methods;

#[no_mangle]
unsafe extern "system" fn JVM_GetClassInterfaces(env: *mut JNIEnv, cls: jclass) -> jobjectArray {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let j_class = from_jclass(jvm, cls);
    let interface_vec = match j_class
        .as_runtime_class(jvm)
        .view()
        .interfaces()
        .map(|interface| {
            let class_obj = get_or_create_class_object(jvm, interface.interface_name().into(), int_state)?;
            Ok(class_obj.duplicate_discouraged())
        })
        .collect::<Result<Vec<_>, WasException>>()
    {
        Ok(interface_vec) => interface_vec,
        Err(WasException {}) => {
            return null_mut();
        }
    };
    let whole_array_runtime_class = assert_inited_or_initing_class(jvm, CPDType::array(CClassName::class().into()));
    let elems = interface_vec.iter().map(|handle| NewJavaValue::AllocObject(handle.as_allocated_obj())).collect_vec();
    let res = jvm.allocate_object(UnAllocatedObject::Array(UnAllocatedObjectArray { whole_array_runtime_class, elems }));
    new_local_ref_public_new(Some(res.as_allocated_obj()), int_state)
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassSigners(env: *mut JNIEnv, cls: jclass) -> jobjectArray {
    null_mut()
    // not supporting class signing atm.
}

#[no_mangle]
unsafe extern "system" fn JVM_GetProtectionDomain(env: *mut JNIEnv, cls: jclass) -> jobject {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let class = from_jclass(jvm, cls).as_runtime_class(jvm);
    match jvm.classes.read().unwrap().protection_domains.get_by_left(&ByAddress(class)) {
        None => null_mut(),
        Some(pd_obj) => new_local_ref_public(pd_obj.clone().owned_inner().to_gc_managed().into(), int_state),
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_GetComponentType(env: *mut JNIEnv, cls: jclass) -> jclass {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let object = from_object_new(jvm, cls);
    let temp = NewJavaValueHandle::from_optional_object(object).cast_class().unwrap().as_type(jvm);
    let object_class = temp.unwrap_ref_type();
    new_local_ref_public_new(
        match JClass::from_type(jvm, int_state, object_class.unwrap_array_type().clone()) {
            Ok(jclass) => jclass,
            Err(WasException {}) => return null_mut(),
        }.full_object_ref().into(),
        int_state,
    )
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassModifiers(env: *mut JNIEnv, cls: jclass) -> jint {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let jclass = from_jclass(jvm, cls);
    jclass.as_runtime_class(jvm).view().access_flags() as jint
}

#[no_mangle]
unsafe extern "system" fn JVM_GetDeclaredClasses(env: *mut JNIEnv, ofClass: jclass) -> jobjectArray {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let class = from_jclass(jvm, ofClass).as_runtime_class(jvm);
    let res_array = match class.view().inner_classes_view() {
        None => vec![],
        Some(inner_classes) => inner_classes
            .classes()
            .filter(|inner_class| inner_class.outer_name(&jvm.string_pool) == class.unwrap_class_class().class_view.name().unwrap_name())
            .flat_map(|inner_class| Some(CPDType::Class(inner_class.complete_name(&jvm.string_pool)?)))
            .collect::<Vec<_>>(),
    }
        .into_iter()
        .map(|ptype| {
            Ok(get_or_create_class_object(jvm, ptype, int_state)?.new_java_handle())
        })
        .collect::<Result<Vec<_>, _>>();
    let obj_array = match res_array {
        Ok(obj_array) => obj_array,
        Err(WasException {}) => return null_mut(),
    };
    let res_jv = JavaValue::new_vec_from_vec(
        jvm,
        obj_array.iter().map(|njvh| njvh.as_njv()).collect_vec(),
        CClassName::class().into(),
    );
    new_local_ref_public_new(Some(res_jv.as_allocated_obj()), int_state)
}

#[no_mangle]
unsafe extern "system" fn JVM_GetDeclaringClass(env: *mut JNIEnv, ofClass: jclass) -> jclass {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let rc = from_jclass(jvm, ofClass).as_runtime_class(jvm);
    if rc.cpdtype().is_primitive(){
        return null_mut()
    }
    let class_name = rc.unwrap_class_class().class_view.name().unwrap_name();
    let view = rc.view();
    let inner_classes = match view.inner_classes_view() {
        Some(x) => x,
        None => return null_mut(),
    };
    for inner_class in inner_classes.classes() {
        // dbg!(inner_class.complete_name(&jvm.string_pool).unwrap().0.to_str(&jvm.string_pool));
        // dbg!(class_name.0.to_str(&jvm.string_pool));
        if inner_class.complete_name(&jvm.string_pool) == Some(class_name){
            let target_class_name = inner_class.outer_name(&jvm.string_pool);
            // dbg!(target_class_name.0.to_str(&jvm.string_pool));
            let class = get_or_create_class_object(jvm,target_class_name.into(),int_state).unwrap();
            return to_object_new(Some(class.as_allocated_obj()));
        }
    }
    return null_mut()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassSignature(env: *mut JNIEnv, cls: jclass) -> jstring {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);

    let rc = from_jclass(jvm, cls).as_runtime_class(jvm);

    let signature = match rc.view().signature_attr() {
        Some(x) => x,
        None => todo!(),
    };

    match JString::from_rust(jvm, int_state, signature) {
        Ok(jstring) => new_local_ref_public_new(jstring.full_object_ref().into(), int_state),
        Err(WasException) => null_mut(),
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassAccessFlags(env: *mut JNIEnv, cls: jclass) -> jint {
    let jvm = get_state(env);
    let res = from_jclass(jvm, cls).as_runtime_class(jvm).view().access_flags() as i32;
    res
}

#[no_mangle]
unsafe extern "system" fn JVM_ClassDepth(env: *mut JNIEnv, name: jstring) -> jint {
    unreachable!("As far as I can tell this is never actually used. But I guess if you see this I was wrong. ")
}

////**
//      * Returns the current execution stack as an array of classes.
//      * <p>
//      * The length of the array is the number of methods on the execution
//      * stack. The element at index <code>0</code> is the class of the
//      * currently executing method, the element at index <code>1</code> is
//      * the class of that method's caller, and so on.
//      *
//      * @return  the execution stack.
//      */
#[no_mangle]
unsafe extern "system" fn JVM_GetClassContext(env: *mut JNIEnv) -> jobjectArray {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let jclasses = match int_state.cloned_stack_snapshot(jvm).into_iter().rev().flat_map(|entry| Some(entry.try_class_pointer(jvm)?.cpdtype()))
        .map(|ptype| get_or_create_class_object(jvm, ptype, int_state)
            .map(|elem| elem.new_java_handle())
        )
        .collect::<Result<Vec<_>, WasException>>() {
        Ok(jclasses) => jclasses,
        Err(WasException {}) => return null_mut(),
    };
    new_local_ref_public_new(JavaValue::new_vec_from_vec(jvm, jclasses.iter().map(|handle| handle.as_njv()).collect(), CClassName::class().into()).new_java_value().unwrap_object_alloc(), int_state)
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassNameUTF(env: *mut JNIEnv, cb: jclass) -> *const c_char {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let jstring = match JavaValue::Object(todo!() /*from_jclass(jvm,JVM_GetClassName(env, cb))*/).cast_string() {
        None => return throw_npe(jvm, int_state),
        Some(jstring) => jstring,
    };
    let rust_string = jstring.to_rust_string(jvm);
    let sketch_string = JVMString::from_regular_string(rust_string.as_str());
    let mut len = 0;
    let mut data_ptr: *mut u8 = null_mut();
    jvm.native.native_interface_allocations.allocate_and_write_vec(sketch_string.buf.clone(), &mut len as *mut jint, &mut data_ptr as *mut *mut u8);
    data_ptr as *const c_char
}

pub mod fields;
pub mod methods;

#[no_mangle]
pub unsafe extern "system" fn JVM_GetCallerClass(env: *mut JNIEnv, depth: ::std::os::raw::c_int) -> jclass {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let mut stack = int_state.frame_iter().collect::<Vec<_>>().into_iter();
    let this_native_fn_frame = stack.next().unwrap();
    assert!(this_native_fn_frame.is_native_method() || this_native_fn_frame.is_opaque());
    let mut parent_frame = stack.next().unwrap();
    if parent_frame.is_native_method() || parent_frame.is_opaque() {
        parent_frame = stack.next().unwrap();
    }
    if parent_frame.is_native_method() || parent_frame.is_opaque() {
        parent_frame = stack.next().unwrap();
    }
    assert!(!parent_frame.is_native_method() && !parent_frame.is_opaque());
    let possibly_class_pointer = stack.find_map(|entry| {
        let class_pointer = entry.try_class_pointer(jvm)?;
        let view = class_pointer.view();
        let method_view = view.method_view_i(entry.method_i(jvm));
        if method_view.is_native() || entry.is_opaque() {
            return None;
        }
        if let Some(name) = view.name().try_unwrap_name() {
            if name == CClassName::method() && view.method_view_i(entry.method_i(jvm)).name() == MethodName::method_invoke() {
                return None;
            }
        }
        Some(class_pointer.clone())
    });
    let type_ = if let Some(class_pointer) = possibly_class_pointer {
        class_pointer.cpdtype()
    } else {
        return null_mut();
    };
    let jclass = load_class_constant_by_type(jvm, int_state, type_).unwrap();
    new_local_ref_public_new(jclass.try_unwrap_object_alloc().unwrap().as_ref().map(|handle| handle.as_allocated_obj()), int_state)
}

#[no_mangle]
unsafe extern "system" fn JVM_IsSameClassPackage(env: *mut JNIEnv, class1: jclass, class2: jclass) -> jboolean {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    match Reflection::is_same_class_package(jvm, int_state, from_jclass(jvm, class1), from_jclass(jvm, class2)) {
        Ok(res) => res,
        Err(WasException {}) => return jboolean::MAX,
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_FindClassFromCaller(env: *mut JNIEnv, c_name: *const ::std::os::raw::c_char, init: jboolean, loader: jobject, caller: jclass) -> jclass {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let name = CStr::from_ptr(&*c_name).to_str().unwrap().to_string();
    let p_type = CompressedClassName(jvm.string_pool.add_name(name, true)).into();

    let loader_name = from_object_new(jvm, loader)
        .map(|loader_obj| NewJavaValueHandle::Object(loader_obj.into()).cast_class_loader().to_jvm_loader(jvm))
        .unwrap_or(LoaderName::BootstrapLoader);

    let class_lookup_result = get_or_create_class_object_force_loader(jvm, p_type, int_state, loader_name);
    match class_lookup_result {
        Ok(class_object) => {
            if init != 0 {
                if let Err(WasException {}) = check_initing_or_inited_class(jvm, int_state, p_type) {
                    return null_mut();
                };
            }
            new_local_ref_public_new(Some(class_object.as_allocated_obj()), int_state)
        }
        Err(WasException {}) => null_mut(),
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassName(env: *mut JNIEnv, cls: jclass) -> jstring {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let obj = from_jclass(jvm, cls).as_runtime_class(jvm);
    let full_name = PTypeView::from_compressed(obj.cpdtype(), &jvm.string_pool).class_name_representation();
    new_string_with_string(env, Wtf8Buf::from_string(full_name))
}