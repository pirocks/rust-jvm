use std::ptr::null_mut;

use classfile_view::view::ClassView;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use descriptor_parser::{MethodDescriptor, parse_method_descriptor};
use jvmti_jni_bindings::{jboolean, jclass, JNIEnv, jobject};
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::ptype::{PType, ReferenceType};
use slow_interpreter::instructions::invoke::virtual_::invoke_virtual_method_i;
use slow_interpreter::java_values::JavaValue;
use slow_interpreter::rust_jni::interface::local_frame::new_local_ref_public;
use slow_interpreter::rust_jni::native_util::{from_object, get_interpreter_state, get_state, to_object};

#[no_mangle]
unsafe extern "C" fn JVM_DoPrivileged(env: *mut JNIEnv, cls: jclass, action: jobject, context: jobject, wrapException: jboolean) -> jobject {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let action = from_object(action);
    let unwrapped_action = action.clone().unwrap();//todo handle npe
    let runtime_class = &unwrapped_action.unwrap_normal_object().class_pointer;
    let class_view = &runtime_class.view();
    let run_method = class_view.lookup_method(&"run".to_string(), &parse_method_descriptor("()Ljava/lang/Object;").unwrap()).unwrap();//todo figure out if these unwraps are okay
    let expected_descriptor = MethodDescriptor {
        parameter_types: vec![],
        return_type: PType::Ref(ReferenceType::Class(ClassName::object())),
    };
    int_state.push_current_operand_stack(JavaValue::Object(action));
    //todo shouldn't this be invoke_virtual
    invoke_virtual_method_i(jvm, int_state, expected_descriptor, runtime_class.clone(), &run_method);
    if int_state.throw().is_some() {
        return null_mut();
    }
    let res = int_state.pop_current_operand_stack().unwrap_object();
    new_local_ref_public(res, int_state)
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
    new_local_ref_public(JavaValue::Object(jvm.thread_state.get_current_thread().thread_object().object().into()).cast_thread().get_inherited_access_control_context().object().into(), int_state)
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
    //todo this is obscure java stuff that isn't supported atm.
    // let int_state = get_interpreter_state(env);
    // new_local_ref_public(None, int_state)
    null_mut()
}
