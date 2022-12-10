use std::collections::HashSet;

use jvmti_jni_bindings::jobject;
use rust_jvm_common::loading::LoaderName;
use rust_jvm_common::StackNativeJavaValue;

use crate::java_values::{JavaValue};

#[derive(Debug, Clone)]
pub struct OpaqueFrameInfo<'gc> {
    pub native_local_refs: Vec<HashSet<jobject>>,
    pub operand_stack: Vec<JavaValue<'gc>>,
}

#[derive(Clone)]
pub struct NativeFrameInfo<'gc> {
    pub method_id: usize,
    pub loader: LoaderName,
    pub native_local_refs: Vec<HashSet<jobject>>,
    // pub local_vars: Vec<NativeJavaValue<'gc>>,
    pub operand_stack: Vec<StackNativeJavaValue<'gc>>,
}

