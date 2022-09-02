use std::cell::RefCell;
use std::ptr::null_mut;

use libc::time;

use another_jit_vm_ir::WasException;
use classfile_view::view::{ClassView, HasAccessFlags};
use classfile_view::view::field_view::FieldView;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use jvmti_jni_bindings::{jboolean, jclass, jint, jio_vfprintf, JNIEnv, jobjectArray};
use rust_jvm_common::classnames::{class_name, ClassName};
use rust_jvm_common::compressed_classfile::{CPDType, CPRefType};
use rust_jvm_common::compressed_classfile::names::CClassName;
use rust_jvm_common::ptype::{PType, ReferenceType};
use slow_interpreter::better_java_stack::opaque_frame::OpaqueFrame;
use slow_interpreter::class_loading::check_initing_or_inited_class;
use slow_interpreter::instructions::ldc::load_class_constant_by_type;
use slow_interpreter::interpreter_state::InterpreterStateGuard;
use slow_interpreter::interpreter_util::{new_object, run_constructor};
use slow_interpreter::java::lang::class::JClass;
use slow_interpreter::java::lang::reflect::field::Field;
use slow_interpreter::java::lang::string::JString;
use slow_interpreter::java_values::{ArrayObject, JavaValue, Object};
use slow_interpreter::jvm_state::JVMState;
use slow_interpreter::new_java_values::java_value_common::JavaValueCommon;
use slow_interpreter::new_java_values::unallocated_objects::{UnAllocatedObject, UnAllocatedObjectArray};
use slow_interpreter::rust_jni::interface::{field_object_from_view, get_interpreter_state, get_state};
use slow_interpreter::rust_jni::interface::local_frame::{new_local_ref_public, new_local_ref_public_new};
use slow_interpreter::rust_jni::native_util::{from_jclass, to_object};

#[no_mangle]
unsafe extern "system" fn JVM_GetClassFieldsCount(env: *mut JNIEnv, cb: jclass) -> jint {
    let jvm = get_state(env);
    from_jclass(jvm, cb).as_runtime_class(jvm).view().num_fields() as i32
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassDeclaredFields<'gc>(env: *mut JNIEnv, ofClass: jclass, publicOnly: jboolean) -> jobjectArray {
    let jvm = get_state(env);
    let class_obj = from_jclass(jvm, ofClass).as_runtime_class(jvm);
    let int_state = get_interpreter_state(env);
    let mut object_array = vec![];
    for f in class_obj.clone().view().fields() {
        let field_object = match field_object_from_view(jvm, int_state, class_obj.clone(), f) {
            Ok(field_object) => field_object,
            Err(WasException {}) => {
                return null_mut();
            }
        };

        object_array.push(field_object)
    }
    let array_rc = check_initing_or_inited_class(jvm, int_state, CPDType::array(CClassName::field().into())).unwrap();
    let res = jvm.allocate_object(UnAllocatedObject::Array(UnAllocatedObjectArray {
        whole_array_runtime_class: array_rc,
        elems: object_array.iter().map(|handle| handle.as_njv()).collect(),
    }));
    new_local_ref_public_new(Some(res.as_allocated_obj()), todo!()/*int_state*/)
}