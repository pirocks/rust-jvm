use std::ffi::CString;
use std::intrinsics::transmute;
use std::ptr::null_mut;
use std::sync::Arc;

use classfile_parser::code::InstructionTypeNum::l2d;
use classfile_view::view::{ClassView, HasAccessFlags};
use classfile_view::view::field_view::FieldView;
use jvmti_jni_bindings::{jboolean, jclass, jfieldID, jint, jvmtiEnv, jvmtiError, jvmtiError_JVMTI_ERROR_NONE};

use crate::field_table::FieldId;
use crate::JVMState;
use crate::jvmti::get_state;
use crate::runtime_class::RuntimeClass;
use crate::rust_jni::native_util::from_jclass;

pub unsafe extern "C" fn is_field_synthetic(env: *mut jvmtiEnv, klass: jclass, field: jfieldID, is_synthetic_ptr: *mut jboolean) -> jvmtiError {
    let jvm = get_state(env);
    let (classfile_view, i) = get_field(klass, field, jvm);
    let field_view = classfile_view.field(i as usize);
    let is_synthetic = field_view.is_synthetic();
    is_synthetic_ptr.write(is_synthetic as jboolean);
    jvmtiError_JVMTI_ERROR_NONE
}

fn get_field(klass: jclass, field: jfieldID, jvm: &JVMState) -> (Arc<ClassView>, u16) {
    let field_id: FieldId = unsafe { transmute(field) };
    let (runtime_class, i) = jvm.field_table.read().unwrap().lookup(field_id);
    unsafe { Arc::ptr_eq(&from_jclass(klass).as_runtime_class(), &runtime_class); }
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
    let name = field_view.field_name();
    let field_desc = field_view.field_desc();
    generic_ptr.write(null_mut());
    name_ptr.write(jvm.native_interface_allocations.allocate_cstring(CString::new(name).unwrap()));
    signature_ptr.write(jvm.native_interface_allocations.allocate_cstring(CString::new(field_desc).unwrap()));
    jvmtiError_JVMTI_ERROR_NONE
}
