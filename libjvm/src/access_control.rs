use std::ptr::null_mut;

use by_address::ByAddress;
use itertools::Itertools;

use classfile_view::view::ClassView;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use jvmti_jni_bindings::{jboolean, jclass, JNIEnv, jobject};
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType, CPRefType};
use rust_jvm_common::compressed_classfile::names::{CClassName, MethodName};
use rust_jvm_common::descriptor_parser::MethodDescriptor;
use rust_jvm_common::ptype::{PType, ReferenceType};
use slow_interpreter::instructions::invoke::virtual_::{invoke_virtual, invoke_virtual_method_i};
use slow_interpreter::interpreter::WasException;
use slow_interpreter::java::NewAsObjectOrJavaValue;
use slow_interpreter::java::security::access_control_context::AccessControlContext;
use slow_interpreter::java_values::{JavaValue, Object};
use slow_interpreter::new_java_values::{NewJavaValue, NewJavaValueHandle};
use slow_interpreter::rust_jni::interface::local_frame::{new_local_ref_public, new_local_ref_public_new};
use slow_interpreter::rust_jni::native_util::{from_object, from_object_new, get_interpreter_state, get_state, to_object};
use slow_interpreter::utils::throw_npe;

#[no_mangle]
unsafe extern "C" fn JVM_DoPrivileged(env: *mut JNIEnv, cls: jclass, action: jobject, context: jobject, wrapException: jboolean) -> jobject {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let action = from_object_new(jvm, action);
    let unwrapped_action = match action {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state);
        }
    };
    let expected_descriptor = CMethodDescriptor { arg_types: vec![], return_type: CPDType::Ref(CPRefType::Class(CClassName::object())) };
    let mut args = vec![];
    args.push(NewJavaValue::AllocObject(unwrapped_action.as_allocated_obj()));
    let res = invoke_virtual(jvm, int_state, MethodName::method_run(), &expected_descriptor,args).unwrap().unwrap().unwrap_object();
    if int_state.throw().is_some() {
        return null_mut();
    }
    new_local_ref_public_new(res.as_ref().map(|handle|handle.as_allocated_obj()), int_state)
}

///Java_java_security_AccessController_getInheritedAccessControlContext
////**
//      * Returns the "inherited" AccessControl context. This is the context
//      * that existed when the thread was created. Package private so
//      * AccessControlContext can use it.
//      */
/// aka this is the inheritedAccessControlContext field on thread object
#[no_mangle]
unsafe extern "system" fn JVM_GetInheritedAccessControlContext(env: *mut JNIEnv, cls: jclass) -> jobject {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    new_local_ref_public(JavaValue::Object(todo!() /*jvm.thread_state.get_current_thread().thread_object().object().into()*/).cast_thread().get_inherited_access_control_context(jvm).object().as_allocated_obj().to_gc_managed().into(), int_state)
}

///  /**
//      * Returns the AccessControl context. i.e., it gets
//      * the protection domains of all the callers on the stack,
//      * starting at the first class with a non-null
//      * ProtectionDomain.
//      *
//      * @return the access control context based on the current stack or
//      *         null if there was only privileged system code.
//      */
#[no_mangle]
unsafe extern "system" fn JVM_GetStackAccessControlContext(env: *mut JNIEnv, cls: jclass) -> jobject {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let stack = int_state.frame_iter().collect_vec();
    let classes_guard = jvm.classes.read().unwrap();
    let protection_domains = &classes_guard.protection_domains;
    let protection_domains = stack
        .iter()
        .rev()
        .flat_map(|entry| {
            match protection_domains.get_by_left(&ByAddress(entry.try_class_pointer(jvm)?.clone())) {
                None => None,
                Some(domain) => {
                    NewJavaValueHandle::Object(domain.owned_inner_ref().handle.duplicate_discouraged()).cast_protection_domain().into()
                }
            }
        })
        .collect::<Vec<_>>();
    if protection_domains.is_empty() {
        return null_mut();
    } else {
        match AccessControlContext::new(jvm, int_state, protection_domains) {
            Ok(access_control_ctx) => new_local_ref_public(todo!()/*access_control_ctx.object().to_gc_managed().into()*/, int_state),
            Err(WasException {}) => return null_mut(),
        }
    }
}