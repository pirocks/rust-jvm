use slow_interpreter::rust_jni::native_util::{to_object, from_object, get_frame, get_state};
use jni_bindings::{jobject, jclass, JNIEnv, jboolean};
use rust_jvm_common::ptype::{PType, ReferenceType};
use rust_jvm_common::classnames::ClassName;


use slow_interpreter::instructions::invoke::virtual_::invoke_virtual_method_i;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use slow_interpreter::java_values::JavaValue;
use descriptor_parser::{MethodDescriptor, parse_method_descriptor};

#[no_mangle]
unsafe extern "system" fn JVM_DoPrivileged(env: *mut JNIEnv, cls: jclass, action: jobject, context: jobject, wrapException: jboolean) -> jobject {
    let state = get_state(env);
    let frame = get_frame(env);
    let action = from_object(action);
    let unwrapped_action = action.clone().unwrap();
    let runtime_class = &unwrapped_action.unwrap_normal_object().class_pointer;
    let class_view = &runtime_class.class_view;
    let run_method = class_view.method_index().lookup(&"run".to_string(), &parse_method_descriptor("()Ljava/lang/Object;").unwrap()).unwrap();
    let expected_descriptor = MethodDescriptor {
        parameter_types: vec![],
        return_type: PType::Ref(ReferenceType::Class(ClassName::object())),
    };
    frame.push(JavaValue::Object(action));
    //todo shouldn't this be invoke_virtual
    invoke_virtual_method_i(state, expected_descriptor, runtime_class.clone(), run_method.method_i(),&run_method, false);
    let res = frame.pop().unwrap_object();
    to_object(res)
}

#[no_mangle]
unsafe extern "system" fn JVM_GetInheritedAccessControlContext(env: *mut JNIEnv, cls: jclass) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetStackAccessControlContext(env: *mut JNIEnv, cls: jclass) -> jobject {
    //todo this is obscure java stuff that isn't supported atm.
    to_object(None)
}
