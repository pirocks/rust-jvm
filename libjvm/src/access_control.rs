use slow_interpreter::rust_jni::native_util::{to_object, from_object, get_frame, get_state};
use jni_bindings::{jobject, jclass, JNIEnv, jboolean};
use runtime_common::java_values::JavaValue;
use rust_jvm_common::ptype::{PType, ReferenceType};
use rust_jvm_common::classnames::ClassName;


use slow_interpreter::instructions::invoke::virtual_::invoke_virtual_method_i;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use classfile_view::view::descriptor_parser::MethodDescriptor;

#[no_mangle]
unsafe extern "system" fn JVM_DoPrivileged(env: *mut JNIEnv, cls: jclass, action: jobject, context: jobject, wrapException: jboolean) -> jobject {
    let state = get_state(env);
    let frame = get_frame(env);
    let action = from_object(action);
    let unwrapped_action = action.clone().unwrap();
    let runtime_class = &unwrapped_action.unwrap_normal_object().class_pointer;
    let classfile = &runtime_class.classfile;
    let (run_method_i, run_method) = classfile.lookup_method("run".to_string(), "()Ljava/lang/Object;".to_string()).unwrap();
    let expected_descriptor = MethodDescriptor {
        parameter_types: vec![],
        return_type: PTypeView::Ref(ReferenceTypeView::Class(ClassName::object())),
    };
    frame.push(JavaValue::Object(action));
    //todo shouldn't this be invoke_virtual
    invoke_virtual_method_i(state, frame.clone(), expected_descriptor, runtime_class.clone(), run_method_i,run_method);
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
