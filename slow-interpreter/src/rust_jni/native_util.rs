use std::os::raw::c_void;
use std::ptr::{NonNull, null_mut};

use gc_memory_layout_common::memory_regions::MemoryRegions;
use jvmti_jni_bindings::{_jobject, jclass, JNIEnv, jobject};

use crate::{AllocatedHandle, JVMState};
use crate::class_objects::get_or_create_class_object;
use crate::java_values::GcManagedObject;
use crate::new_java_values::allocated_objects::AllocatedObject;
use crate::new_java_values::NewJavaValueHandle;
use crate::rust_jni::jni_interface::jni::{get_interpreter_state, get_state};
use crate::rust_jni::jni_interface::local_frame::new_local_ref_public_new;
use crate::stdlib::java::lang::class::JClass;

pub unsafe extern "C" fn get_object_class(env: *mut JNIEnv, obj: jobject) -> jclass {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let unwrapped = from_object_new(jvm, obj).unwrap(); //todo handle npe
    let object_region_header = MemoryRegions::find_object_region_header(NonNull::new(obj as *mut c_void).unwrap());
    if object_region_header.class_pointer_cache != null_mut() {
        return object_region_header.class_pointer_cache as jclass;
    }
    let rc = unwrapped.runtime_class(jvm);
    let class_object = get_or_create_class_object(jvm, rc.cpdtype(), int_state);
    let res_class = new_local_ref_public_new(class_object.unwrap().as_allocated_obj().into(), int_state) as jclass;
    object_region_header.class_pointer_cache = res_class;
    res_class
}


pub unsafe fn to_object<'gc>(obj: Option<GcManagedObject<'gc>>) -> jobject {
    match obj {
        None => std::ptr::null_mut(),
        Some(o) => {
            // o.self_check();
            let res = o.raw_ptr_usize() as *mut _jobject;
            res
        }
    }
}

pub unsafe fn to_object_new<'gc>(obj: Option<AllocatedObject<'gc, '_>>) -> jobject {
    match obj {
        None => std::ptr::null_mut(),
        Some(o) => {
            let res = o.raw_ptr_usize() as *mut _jobject;
            res
        }
    }
}

pub unsafe fn from_object<'gc>(jvm: &'gc JVMState<'gc>, obj: jobject) -> Option<GcManagedObject<'gc>> {
    let option = NonNull::new(obj as *mut c_void)?;
    // if !jvm.gc.all_allocated_object.read().unwrap().contains(&option) {
    //     dbg!(option.as_ptr());
    //     dbg!(jvm.gc.all_allocated_object.read().unwrap());
    //     panic!()
    // }
    todo!()
    // Some(GcManagedObject::from_native(option, jvm))
}

pub unsafe fn from_object_new<'gc>(jvm: &'gc JVMState<'gc>, obj: jobject) -> Option<AllocatedHandle<'gc>> {
    let ptr = NonNull::new(obj as *mut c_void)?;
    let handle = jvm.gc.register_root_reentrant(jvm, ptr);
    Some(handle)
}

pub unsafe fn from_jclass<'gc>(jvm: &'gc JVMState<'gc>, obj: jclass) -> JClass<'gc> {//all jclasses have life of 'gc
    try_from_jclass(jvm, obj).unwrap()
    //todo handle npe
}

pub unsafe fn try_from_jclass<'gc>(jvm: &'gc JVMState<'gc>, obj: jclass) -> Option<JClass<'gc>> { //all jclasses have life of 'gc
    let possibly_null = from_object_new(jvm, obj);
    let not_null = possibly_null?;
    NewJavaValueHandle::Object(not_null).cast_class()
}