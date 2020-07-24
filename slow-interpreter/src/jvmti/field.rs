use std::intrinsics::transmute;
use std::sync::Arc;

use classfile_parser::code::InstructionTypeNum::l2d;
use classfile_view::view::field_view::FieldView;
use classfile_view::view::HasAccessFlags;
use jvmti_jni_bindings::{jboolean, jclass, jfieldID, jint, jvmtiEnv, jvmtiError, jvmtiError_JVMTI_ERROR_NONE};

use crate::field_table::FieldId;
use crate::JVMState;
use crate::jvmti::get_state;
use crate::rust_jni::native_util::from_jclass;

pub unsafe extern "C" fn is_field_synthetic(env: *mut jvmtiEnv, klass: jclass, field: jfieldID, is_synthetic_ptr: *mut jboolean) -> jvmtiError {
    let jvm = get_state(env);
    let field_view = get_field(klass, field, jvm);
    let is_synthetic = field_view.is_synthetic();
    is_synthetic_ptr.write(is_synthetic as jboolean);
    jvmtiError_JVMTI_ERROR_NONE
}

fn get_field(klass: jclass, field: jfieldID, jvm: &JVMState) -> FieldView {
    let field_id: FieldId = unsafe { transmute(field) };
    let (runtime_class, i) = jvm.field_table.read().unwrap().lookup(field_id);
    unsafe { Arc::ptr_eq(&from_jclass(klass).as_runtime_class(), &runtime_class); }
    let field_view = runtime_class.view().field(i as usize);
    field_view
}

pub unsafe extern "C" fn get_field_modifiers(env: *mut jvmtiEnv, klass: jclass, field: jfieldID, modifiers_ptr: *mut jint) -> jvmtiError {
    let jvm = get_state(env);
    let field_view = get_field(klass, field, jvm);
    modifiers_ptr.write(field_view.access_flags() as jint);
    jvmtiError_JVMTI_ERROR_NONE
}
