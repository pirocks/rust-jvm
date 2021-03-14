use std::cell::RefCell;
use std::ptr::null_mut;
use std::sync::Arc;

use classfile_view::view::{ClassView, HasAccessFlags};
use classfile_view::view::field_view::FieldView;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use jvmti_jni_bindings::{jboolean, jclass, jint, jio_vfprintf, JNIEnv, jobjectArray};
use rust_jvm_common::classnames::{class_name, ClassName};
use rust_jvm_common::ptype::{PType, ReferenceType};
use slow_interpreter::instructions::ldc::load_class_constant_by_type;
use slow_interpreter::interpreter::WasException;
use slow_interpreter::interpreter_state::InterpreterStateGuard;
use slow_interpreter::interpreter_util::{push_new_object, run_constructor};
use slow_interpreter::java::lang::class::JClass;
use slow_interpreter::java::lang::reflect::field::Field;
use slow_interpreter::java::lang::string::JString;
use slow_interpreter::java_values::{ArrayObject, JavaValue, Object};
use slow_interpreter::jvm_state::JVMState;
use slow_interpreter::runtime_class::RuntimeClass;
use slow_interpreter::rust_jni::interface::field_object_from_view;
use slow_interpreter::rust_jni::interface::local_frame::new_local_ref_public;
use slow_interpreter::rust_jni::native_util::{from_jclass, get_interpreter_state, get_state, to_object};

#[no_mangle]
unsafe extern "system" fn JVM_GetClassFieldsCount(env: *mut JNIEnv, cb: jclass) -> jint {
    let jvm = get_state(env);
    from_jclass(cb).as_runtime_class(jvm).view().num_fields() as i32
}


#[no_mangle]
unsafe extern "system" fn JVM_GetClassDeclaredFields(env: *mut JNIEnv, ofClass: jclass, publicOnly: jboolean) -> jobjectArray {
    let jvm = get_state(env);
    let class_obj = from_jclass(ofClass).as_runtime_class(jvm);
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let mut object_array = vec![];
    for f in class_obj.clone().view().fields() {
        let field_object = match field_object_from_view(jvm, int_state, class_obj.clone(), f) {
            Ok(field_object) => field_object,
            Err(WasException {}) => {
                return null_mut()
            }
        };

        object_array.push(field_object)
    }
    let res = Some(Arc::new(
        Object::Array(ArrayObject::new_array(
            jvm,
            int_state,
            object_array,
            PTypeView::Ref(ReferenceTypeView::Class(ClassName::field())),
            jvm.thread_state.new_monitor("".to_string())
        ))));
    new_local_ref_public(res, int_state)
}

