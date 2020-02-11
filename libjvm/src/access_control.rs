use slow_interpreter::rust_jni::native_util::{to_object, from_object, get_frame, get_state};
use jni_bindings::{jobject, jclass, JNIEnv, jboolean};
use slow_interpreter::instructions::invoke::actually_virtual;
use runtime_common::java_values::JavaValue;
use rust_jvm_common::unified_types::{ParsedType, ClassWithLoader};
use classfile_parser::types::MethodDescriptor;
use rust_jvm_common::classnames::ClassName;

#[no_mangle]
unsafe extern "system" fn JVM_DoPrivileged(env: *mut JNIEnv, cls: jclass, action: jobject, context: jobject, wrapException: jboolean) -> jobject {
//    if wrapException == 0{
//        unimplemented!()
//    }
    let state = get_state(env);
    let frame = get_frame(env);
    let action = from_object(action);
//    dbg!(&class_name(&action.as_ref().unwrap().unwrap_object().class_pointer.classfile));
//    dbg!(&action.as_re/f().unwrap().unwrap_object().fields.borrow().keys());
    let unwrapped_action = action.clone().unwrap();
    let runtime_class = &unwrapped_action.unwrap_normal_object().class_pointer;
    let classfile = &runtime_class.classfile;
    let (run_method_i, run_method) = classfile.lookup_method("run".to_string(), "()Ljava/lang/Object;".to_string()).unwrap();
    let expected_descriptor = MethodDescriptor {
        parameter_types: vec![],
        return_type: ParsedType::Class(ClassWithLoader { class_name: ClassName::object(), loader: runtime_class.loader.clone() }),
    };
    frame.push(JavaValue::Object(action));
//    dbg!(&frame.operand_stack);
//    dbg!(&run_method.code_attribute().unwrap());
    //todo shouldn't this be invoke_virtual
    actually_virtual(state, frame.clone(), expected_descriptor, &runtime_class, run_method);
//    dbg!(&frame.operand_stack);
//    unimplemented!()

    let res = frame.pop().unwrap_object();
//    dbg!(&res);
    to_object(res)
}

#[no_mangle]
unsafe extern "system" fn JVM_GetInheritedAccessControlContext(env: *mut JNIEnv, cls: jclass) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetStackAccessControlContext(env: *mut JNIEnv, cls: jclass) -> jobject {
//    let frame = get_frame(env);
//    frame.print_stack_trace();
    //todo this is obscure java stuff that isn't supported atm.
    to_object(None)
}
