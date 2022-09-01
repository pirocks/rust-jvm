use std::ffi::c_void;
use std::intrinsics::{offset, transmute, volatile_load};
use std::mem::size_of;
use std::ops::Deref;
use std::ptr::{null, null_mut};

use nix::convert_ioctl_res;

use classfile_view::view::{ClassView, HasAccessFlags};
use classfile_view::view::ptype_view::PTypeView;
use jvmti_jni_bindings::{jboolean, jbyte, jclass, jint, jlong, JNIEnv, jobject, JVM_CALLER_DEPTH};
use rust_jvm_common::compressed_classfile::CPDType;
use rust_jvm_common::compressed_classfile::names::FieldName;
use rust_jvm_common::FieldId;
use slow_interpreter::class_loading::check_initing_or_inited_class;
use slow_interpreter::interpreter_util::new_object;
use slow_interpreter::java_values::{JavaValue, Object};
use slow_interpreter::jvm_state::JVMState;
use slow_interpreter::new_java_values::{NewJavaValue, NewJavaValueHandle};
use slow_interpreter::new_java_values::allocated_objects::AllocatedHandle;
use slow_interpreter::new_java_values::java_value_common::JavaValueCommon;
use slow_interpreter::new_java_values::owned_casts::OwnedCastAble;
use slow_interpreter::runtime_class::static_vars;
use slow_interpreter::rust_jni::interface::get_field::new_field_id;
use slow_interpreter::rust_jni::native_util::{from_jclass, from_object, from_object_new, to_object, to_object_new};
use slow_interpreter::utils::throw_npe;

use crate::introspection::JVM_GetCallerClass;

pub mod compare_and_swap;
pub mod defineAnonymousClass;
pub mod object_access;
pub mod reflection;
pub mod raw_pointer;
pub mod defineClass;
pub mod park;