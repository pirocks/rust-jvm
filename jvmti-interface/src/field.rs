use std::ffi::CString;
use std::mem::size_of;
use std::ptr::null_mut;
use std::sync::Arc;

use classfile_view::view::{ClassView, HasAccessFlags};
use jvmti_jni_bindings::{jboolean, jclass, jfieldID, jint, jobject, jvmtiEnv, jvmtiError, jvmtiError_JVMTI_ERROR_NONE};
use rust_jvm_common::FieldId;

use slow_interpreter::jvm_state::JVMState;
use slow_interpreter::rust_jni::native_util::from_jclass;
use slow_interpreter::rust_jni::jvmti::{get_state};
use slow_interpreter::utils::new_field_id;

pub unsafe extern "C" fn is_field_synthetic(env: *mut jvmtiEnv, klass: jclass, field: jfieldID, is_synthetic_ptr: *mut jboolean) -> jvmtiError {
    let jvm = get_state(env);
    let (classfile_view, i) = get_field(klass, field, jvm);
    let field_view = classfile_view.field(i as usize);
    let is_synthetic = field_view.is_synthetic();
    is_synthetic_ptr.write(is_synthetic as jboolean);
    jvmtiError_JVMTI_ERROR_NONE
}

fn get_field<'gc>(klass: jclass, field: jfieldID, jvm: &'gc JVMState<'gc>) -> (Arc<dyn ClassView>, u16) {
    let field_id: FieldId = field as usize;
    let (runtime_class, i) = jvm.field_table.read().unwrap().lookup(field_id);
    unsafe {
        assert!(Arc::ptr_eq(&from_jclass(jvm, klass as jobject).as_runtime_class(jvm), &runtime_class));
    }
    let view = runtime_class.view();
    (view.clone(), i)
}

pub unsafe extern "C" fn get_field_modifiers(env: *mut jvmtiEnv, klass: jclass, field: jfieldID, modifiers_ptr: *mut jint) -> jvmtiError {
    let jvm = get_state(env);
    let (classfile_view, i) = get_field(klass, field, jvm);
    let field_view = classfile_view.field(i as usize);
    modifiers_ptr.write(field_view.access_flags() as jint);
    jvmtiError_JVMTI_ERROR_NONE
}

pub unsafe extern "C" fn get_field_name(env: *mut jvmtiEnv, klass: jclass, field: jfieldID, name_ptr: *mut *mut ::std::os::raw::c_char, signature_ptr: *mut *mut ::std::os::raw::c_char, generic_ptr: *mut *mut ::std::os::raw::c_char) -> jvmtiError {
    let jvm = get_state(env);
    let (classfile_view, i) = get_field(klass, field, jvm);
    let field_view = classfile_view.field(i as usize);
    let name = field_view.field_name().0.to_str(&jvm.string_pool);
    let field_desc = field_view.field_desc();
    generic_ptr.write(null_mut());
    name_ptr.write(jvm.native.native_interface_allocations.allocate_cstring(CString::new(name).unwrap()));
    signature_ptr.write(jvm.native.native_interface_allocations.allocate_cstring(CString::new(field_desc).unwrap()));
    jvmtiError_JVMTI_ERROR_NONE
}

pub unsafe extern "C" fn get_class_fields(env: *mut jvmtiEnv, klass: jclass, field_count_ptr: *mut jint, fields_ptr: *mut *mut jfieldID) -> jvmtiError {
    let jvm = get_state(env);
    let tracing_guard = jvm.config.tracing.trace_jdwp_function_enter(jvm, "GetClassFields");
    let class_obj = from_jclass(jvm, klass as jobject);
    let runtime_class = class_obj.as_runtime_class(jvm);
    let class_view = runtime_class.view();
    let num_fields = class_view.num_fields();
    field_count_ptr.write(num_fields as jint);
    fields_ptr.write(libc::calloc(num_fields, size_of::<*mut jfieldID>()) as *mut *mut jvmti_jni_bindings::_jfieldID);
    for i in 0..num_fields {
        fields_ptr.read().add(i).write(new_field_id(jvm, runtime_class.clone(), i))
    }
    jvm.config.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}