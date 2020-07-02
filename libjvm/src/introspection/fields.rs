use jvmti_jni_bindings::{JNIEnv, jclass, jint, jboolean, jobjectArray};
use slow_interpreter::rust_jni::native_util::{get_frame, get_state, to_object, from_jclass};
use slow_interpreter::interpreter_util::{push_new_object, check_inited_class, run_constructor};
use rust_jvm_common::classnames::{ClassName, class_name};
use slow_interpreter::instructions::ldc::{load_class_constant_by_type, create_string_on_stack};

use std::sync::Arc;
use std::cell::RefCell;
use rust_jvm_common::ptype::{PType, ReferenceType};
use libjvm_utils::ptype_to_class_object;
use classfile_view::view::ptype_view::{ReferenceTypeView, PTypeView};
use slow_interpreter::java_values::{JavaValue, Object, ArrayObject};
use descriptor_parser::parse_field_descriptor;
use slow_interpreter::java::lang::reflect::field::Field;
use slow_interpreter::java::lang::string::JString;
use slow_interpreter::rust_jni::interface::string::STRING_INTERNMENT_CAMP;
use classfile_view::view::HasAccessFlags;


#[no_mangle]
unsafe extern "system" fn JVM_GetClassFieldsCount(env: *mut JNIEnv, cb: jclass) -> jint {
    unimplemented!()
}


#[no_mangle]
unsafe extern "system" fn JVM_GetClassDeclaredFields(env: *mut JNIEnv, ofClass: jclass, publicOnly: jboolean) -> jobjectArray {
    let frame = get_frame(&mut get_frames(env));
    let jvm = get_state(env);
    let class_obj = from_jclass(ofClass).as_runtime_class();
    let mut object_array = vec![];
    // dbg!(unsafe {&STRING_INTERNMENT_CAMP});
    &class_obj.view().fields().enumerate().for_each(|(i, f)| {
        //todo so this is big and messy put I don't really see a way to simplify
        let field_class_name_ = class_obj.clone().view().name();
        load_class_constant_by_type(jvm, &frame, &PTypeView::Ref(ReferenceTypeView::Class(field_class_name_)));
        let parent_runtime_class = frame.pop();

        let field_name = f.field_name();

        let field_desc_str = f.field_desc();
        let field_type = parse_field_descriptor(field_desc_str.as_str()).unwrap().field_type;
        let field_type_class = ptype_to_class_object(jvm, &frame, &field_type);

        let modifiers = f.access_flags() as i32;
        let slot = i as i32;
        let clazz = parent_runtime_class.cast_class();
        let name = JString::from(jvm, &frame, field_name);
        let type_ = JavaValue::Object(field_type_class).cast_class();
        let signature = JString::from(jvm, &frame, field_desc_str);
        let annotations_ = vec![];//todo impl annotations.

        object_array.push(Field::init(
            jvm,
            &frame,
            clazz,
            name,
            type_,
            modifiers,
            slot,
            signature,
            annotations_
        ).java_value())
    });
    let res = Some(Arc::new(
        Object::Array(ArrayObject {
            elem_type: PTypeView::Ref(ReferenceTypeView::Class(ClassName::field())),
            elems: RefCell::new(object_array),
            monitor: jvm.thread_state.new_monitor("".to_string())
        })));
    to_object(res)
}


