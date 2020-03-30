use jni_bindings::{jdouble, jboolean, JNIEnv, jclass};
use rust_jvm_common::classfile::ACC_INTERFACE;
use rust_jvm_common::classnames::class_name;
use slow_interpreter::rust_jni::interface::util::runtime_class_from_object;
use slow_interpreter::rust_jni::native_util::{from_object, get_state, get_frame};
use classfile_view::view::HasAccessFlags;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};

#[no_mangle]
unsafe extern "system" fn JVM_IsNaN(d: jdouble) -> jboolean {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_IsInterface(env: *mut JNIEnv, cls: jclass) -> jboolean {
    let state = get_state(env);
    let frame = get_frame(env);
    frame.print_stack_trace();
    let obj = from_object(cls).unwrap().clone();
    let normal_obj = obj.unwrap_normal_object();
    let temp = normal_obj.class_object_ptype.borrow();
    let type_view = temp.as_ref().unwrap();
    (match type_view {
        PTypeView::ByteType => false,
        PTypeView::CharType => false,
        PTypeView::DoubleType => false,
        PTypeView::FloatType => false,
        PTypeView::IntType => false,
        PTypeView::LongType => false,
        PTypeView::Ref(r) => {
            match r {
                ReferenceTypeView::Class(c) => {
                    state.class_object_pool.borrow().get(type_view).unwrap().unwrap_normal_object().class_pointer.class_view.is_interface()
                },
                ReferenceTypeView::Array(a) => {
                    false
                },
            }
        },
        PTypeView::ShortType => false,
        PTypeView::BooleanType => false,
        PTypeView::VoidType => false,
        PTypeView::TopType => panic!(),
        PTypeView::NullType => panic!(),
        PTypeView::Uninitialized(_) => panic!(),
        PTypeView::UninitializedThis => panic!(),
        PTypeView::UninitializedThisOrClass(_) => panic!(),
    }) as jboolean

}

#[no_mangle]
unsafe extern "system" fn JVM_IsArrayClass(env: *mut JNIEnv, cls: jclass) -> jboolean {
    let object_non_null = from_object(cls).unwrap().clone();
    let ptype = object_non_null.unwrap_normal_object().class_object_ptype.borrow();
    let is_array = ptype.as_ref().unwrap().is_array();
    is_array as jboolean
}

#[no_mangle]
unsafe extern "system" fn JVM_IsPrimitiveClass(env: *mut JNIEnv, cls: jclass) -> jboolean {
    let class_object = runtime_class_from_object(cls,get_state(env),&get_frame(env));
    if class_object.is_none() {
        dbg!(&class_object);
        return false as jboolean;
    }
    let name_ = class_name(&class_object.unwrap().classfile);
    let name = name_.get_referred_name();
    // dbg!(name);
    // dbg!(name == &"java/lang/Integer".to_string());
    let is_primitive = name == &"java/lang/Boolean".to_string() ||
        name == &"java/lang/Character".to_string() ||
        name == &"java/lang/Byte".to_string() ||
        name == &"java/lang/Short".to_string() ||
        name == &"java/lang/Integer".to_string() ||
        name == &"java/lang/Long".to_string() ||
        name == &"java/lang/Float".to_string() ||
        name == &"java/lang/Double".to_string() ||
        name == &"java/lang/Void".to_string();

    dbg!(is_primitive);

    dbg!(is_primitive as jboolean);
    is_primitive as jboolean
}


#[no_mangle]
unsafe extern "system" fn JVM_IsConstructorIx(env: *mut JNIEnv, cb: jclass, index: ::std::os::raw::c_int) -> jboolean {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_IsVMGeneratedMethodIx(env: *mut JNIEnv, cb: jclass, index: ::std::os::raw::c_int) -> jboolean {
    unimplemented!()
}

