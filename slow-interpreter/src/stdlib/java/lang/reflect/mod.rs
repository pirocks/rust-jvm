use itertools::Itertools;

use classfile_view::view::HasAccessFlags;
use classfile_view::view::method_view::MethodView;
use jvmti_jni_bindings::jint;
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::{CPDType, CPRefType};


use crate::{check_initing_or_inited_class, JavaValueCommon, PushableFrame, UnAllocatedObject, WasException};
use crate::jvm_state::JVMState;
use crate::new_java_values::NewJavaValueHandle;
use crate::new_java_values::unallocated_objects::UnAllocatedObjectArray;
use crate::stdlib::java::lang::class::JClass;
use crate::stdlib::java::lang::string::JString;
use crate::stdlib::java::NewAsObjectOrJavaValue;

/*
// unofficial modifier flags, used by HotSpot:
    static final int BRIDGE    = 0x00000040;
    static final int VARARGS   = 0x00000080;
    static final int SYNTHETIC = 0x00001000;
    static final int ANNOTATION= 0x00002000;
    static final int ENUM      = 0x00004000;

    static final int
                MN_IS_METHOD           = 0x00010000, // method (not constructor)
                MN_IS_CONSTRUCTOR      = 0x00020000, // constructor
                MN_IS_FIELD            = 0x00040000, // field
                MN_IS_TYPE             = 0x00080000, // nested type
                MN_CALLER_SENSITIVE    = 0x00100000, // @CallerSensitive annotation detected
                MN_REFERENCE_KIND_SHIFT = 24, // refKind
                MN_REFERENCE_KIND_MASK = 0x0F000000 >> MN_REFERENCE_KIND_SHIFT,
                // The SEARCH_* bits are not for MN.flags but for the matchFlags argument of MHN.getMembers:
                MN_SEARCH_SUPERCLASSES = 0x00100000,
                MN_SEARCH_INTERFACES   = 0x00200000;

         /**
         * Access modifier flags.
         */
        static final char
            ACC_PUBLIC                 = 0x0001,
            ACC_PRIVATE                = 0x0002,
            ACC_PROTECTED              = 0x0004,
            ACC_STATIC                 = 0x0008,
            ACC_FINAL                  = 0x0010,
            ACC_SYNCHRONIZED           = 0x0020,
            ACC_VOLATILE               = 0x0040,
            ACC_TRANSIENT              = 0x0080,
            ACC_NATIVE                 = 0x0100,
            ACC_INTERFACE              = 0x0200,
            ACC_ABSTRACT               = 0x0400,
            ACC_STRICT                 = 0x0800,
            ACC_SYNTHETIC              = 0x1000,
            ACC_ANNOTATION             = 0x2000,
            ACC_ENUM                   = 0x4000,
            // aliases:
            ACC_SUPER                  = ACC_SYNCHRONIZED,
            ACC_BRIDGE                 = ACC_VOLATILE,
            ACC_VARARGS                = ACC_TRANSIENT;

            todo do these need to be added on top of access flags?
*/

fn get_modifiers(method_view: &MethodView) -> jint {
    method_view.access_flags() as i32
}

fn get_signature<'gc, 'l>(
    jvm: &'gc JVMState<'gc>,
    int_state: &mut impl PushableFrame<'gc>,
    method_view: &MethodView,
) -> Result<Option<JString<'gc>>, WasException<'gc>> {
    match method_view.generic_signature() {
        None => Ok(None),
        Some(sig) => Ok(Some(JString::from_rust(jvm, int_state, sig)?.intern(jvm, int_state)?))
    }
}

fn exception_types_table<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, method_view: &MethodView) -> Result<NewJavaValueHandle<'gc>, WasException<'gc>> {
    let class_type: CPDType = CClassName::class().into();
    let empty_vec = vec![];
    let types_iter = method_view
        .code_attribute()
        .map(|x| &x.exception_table)
        .unwrap_or(&empty_vec)
        .iter()
        .map(|x| x.catch_type)
        .map(|x| match x {
            None => CPRefType::Class(CClassName::throwable()),
            Some(x) => CPRefType::Class(x),
        })
        .map(|x| x.to_cpdtype());

    let mut exception_table = vec![]; //types_iter
    for ptype in types_iter {
        exception_table.push(JClass::from_type(jvm, int_state, ptype)?.new_java_value_handle())
    }
    Ok(NewJavaValueHandle::Object(jvm.allocate_object(UnAllocatedObject::Array(UnAllocatedObjectArray {
        whole_array_runtime_class: check_initing_or_inited_class(jvm, int_state, CPDType::array(class_type)).unwrap(),
        elems: exception_table.iter().map(|handle| handle.as_njv()).collect_vec(),
    }))))
}

fn parameters_type_objects<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, method_view: &MethodView) -> Result<NewJavaValueHandle<'gc>, WasException<'gc>> {
    let class_type: CPDType = CClassName::class().into();
    let mut res = vec![];
    let parsed = method_view.desc();
    for param_type in &parsed.arg_types {
        res.push(JClass::from_type(jvm, int_state, param_type.clone())?.new_java_value_handle());
    }
    let not_owned_elems = res.iter().map(|handle| handle.as_njv()).collect_vec();
    let whole_array_runtime_class = check_initing_or_inited_class(jvm, int_state, CPDType::array(class_type)).unwrap();

    let allocated_obj = jvm.allocate_object(UnAllocatedObject::Array(UnAllocatedObjectArray { whole_array_runtime_class, elems: not_owned_elems }));
    Ok(NewJavaValueHandle::Object(allocated_obj))
}

pub mod method;
pub mod constructor;
pub mod field;

