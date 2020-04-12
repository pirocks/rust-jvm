use jni_bindings::{JNIEnv, jclass, jint, jboolean, jobjectArray};
use slow_interpreter::rust_jni::native_util::{get_frame, get_state, to_object};
use slow_interpreter::rust_jni::interface::util::runtime_class_from_object;
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
use slow_interpreter::monitor::Monitor;


#[no_mangle]
unsafe extern "system" fn JVM_GetClassFieldsCount(env: *mut JNIEnv, cb: jclass) -> jint {
    unimplemented!()
}


#[no_mangle]
unsafe extern "system" fn JVM_GetClassDeclaredFields(env: *mut JNIEnv, ofClass: jclass, publicOnly: jboolean) -> jobjectArray {
    let frame = get_frame(env);
    let state = get_state(env);
    let class_obj = runtime_class_from_object(ofClass, get_state(env),&get_frame(env));
    let mut object_array = vec![];
    // dbg!(unsafe {&STRING_INTERNMENT_CAMP});
    &class_obj.clone().unwrap().classfile.fields.iter().enumerate().for_each(|(i, f)| {
        //todo so this is big and messy put I don't really see a way to simplify
        let field_class_name_ = class_obj.clone().as_ref().unwrap().class_view.name();
        load_class_constant_by_type(state, &frame, &PTypeView::Ref(ReferenceTypeView::Class(field_class_name_)));
        let parent_runtime_class = frame.pop();

        let field_name = class_obj.clone().unwrap().classfile.constant_pool[f.name_index as usize].extract_string_from_utf8();

        let field_desc_str = class_obj.clone().unwrap().classfile.constant_pool[f.descriptor_index as usize].extract_string_from_utf8();
        let field_type = parse_field_descriptor(field_desc_str.as_str()).unwrap().field_type;
        let field_type_class = ptype_to_class_object(state, &frame, &field_type);

        let modifiers = f.access_flags as i32;
        let slot = i as i32;
        let clazz = parent_runtime_class.cast_class();
        let name = JString::from(state,&frame,field_name);
        let type_ = JavaValue::Object(field_type_class).cast_class();
        let signature = JString::from(state,&frame,field_desc_str);
        let annotations_ = vec![];//todo impl annotations.

        object_array.push(Field::init(
            state,
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
    // dbg!(unsafe {&STRING_INTERNMENT_CAMP});
    let res = Some(Arc::new(
        Object::Array(ArrayObject {
            elem_type: PTypeView::Ref(ReferenceTypeView::Class(ClassName::field())),
            elems: RefCell::new(object_array),
            monitor: Monitor::new()
        })));
    to_object(res)
}


