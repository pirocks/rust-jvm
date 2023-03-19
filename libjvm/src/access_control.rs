use std::ptr::null_mut;

use by_address::ByAddress;
use itertools::Itertools;

use jvmti_jni_bindings::{jboolean, jclass, JNIEnv, jobject};
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::CMethodDescriptor;
use rust_jvm_common::compressed_classfile::method_names::MethodName;


use slow_interpreter::better_java_stack::frames::HasFrame;
use slow_interpreter::exceptions::WasException;
use slow_interpreter::interpreter::common::invoke::virtual_::{invoke_virtual};
use slow_interpreter::java_values::{ExceptionReturn, JavaValue};
use slow_interpreter::jvm_state::JVMState;
use slow_interpreter::new_java_values::{NewJavaValue};
use slow_interpreter::new_java_values::owned_casts::OwnedCastAble;
use slow_interpreter::rust_jni::jni_utils::{get_interpreter_state, get_state, get_throw, new_local_ref_public, new_local_ref_public_new};
use slow_interpreter::rust_jni::native_util::{from_object_new};
use slow_interpreter::stdlib::java::NewAsObjectOrJavaValue;
use slow_interpreter::stdlib::java::security::access_control_context::AccessControlContext;
use slow_interpreter::stdlib::java::security::protection_domain::ProtectionDomain;
use slow_interpreter::throw_utils::throw_npe;

#[no_mangle]
unsafe extern "C" fn JVM_DoPrivileged(env: *mut JNIEnv, _cls: jclass, action: jobject, _context: jobject, _wrapException: jboolean) -> jobject {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let action = from_object_new(jvm, action);
    let unwrapped_action = match action {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state,get_throw(env));
        }
    };
    let expected_descriptor = CMethodDescriptor { arg_types: vec![], return_type: CClassName::object().into() };
    let mut args = vec![];
    args.push(NewJavaValue::AllocObject(unwrapped_action.as_allocated_obj()));
    let res = match invoke_virtual(jvm, int_state, MethodName::method_run(), &expected_descriptor, args) {
        Ok(x) => x,
        Err(WasException { exception_obj }) => {
            exception_obj.print_stack_trace(jvm,int_state).unwrap();
            *get_throw(env) = Some(WasException { exception_obj });
            return null_mut();
        }
    }.unwrap().unwrap_object();
    new_local_ref_public_new(res.as_ref().map(|handle| handle.as_allocated_obj()), int_state)
}

///Java_java_security_AccessController_getInheritedAccessControlContext
////**
//      * Returns the "inherited" AccessControl context. This is the context
//      * that existed when the thread was created. Package private so
//      * AccessControlContext can use it.
//      */
/// aka this is the inheritedAccessControlContext field on thread object
#[no_mangle]
unsafe extern "system" fn JVM_GetInheritedAccessControlContext(env: *mut JNIEnv, _cls: jclass) -> jobject {
    let _jvm = get_state(env);
    let _int_state = get_interpreter_state(env);
    new_local_ref_public(JavaValue::Object(todo!() /*jvm.thread_state.get_current_thread().thread_object().object().into()*/).cast_thread().get_inherited_access_control_context(_jvm).object().to_gc_managed().into(), _int_state)
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
unsafe extern "system" fn JVM_GetStackAccessControlContext<'vm>(env: *mut JNIEnv, _cls: jclass) -> jobject {
    let jvm: &'vm JVMState<'vm> = get_state(env);
    let int_state = get_interpreter_state(env);
    let stack = int_state.frame_iter().collect_vec();
    let classes_guard = jvm.classes.read().unwrap();
    let protection_domains = &classes_guard.protection_domains;
    let protection_domains: Vec<ProtectionDomain<'vm>> = stack
        .iter()
        .rev()
        .flat_map(|entry| {
            match protection_domains.get_by_left(&ByAddress(entry.try_class_pointer(jvm).as_ref().ok()?.clone())) {
                None => None,
                Some(domain) => {
                    domain.owned_inner_ref().duplicate_discouraged().cast_protection_domain().into()
                }
            }
        })
        .collect::<Vec<_>>();
    if protection_domains.is_empty() {
        return null_mut();
    } else {
        match AccessControlContext::new(jvm, int_state, protection_domains) {
            Ok(access_control_ctx) => new_local_ref_public_new(access_control_ctx.full_object_ref().into(), int_state),
            Err(WasException { exception_obj }) => {
                *get_throw(env) = Some(WasException{ exception_obj });
                return jobject::invalid_default();
            }
        }
    }
}
