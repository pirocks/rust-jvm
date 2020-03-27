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


#[no_mangle]
unsafe extern "system" fn JVM_GetClassFieldsCount(env: *mut JNIEnv, cb: jclass) -> jint {
    unimplemented!()
}


#[no_mangle]
unsafe extern "system" fn JVM_GetClassDeclaredFields(env: *mut JNIEnv, ofClass: jclass, publicOnly: jboolean) -> jobjectArray {
    let frame = get_frame(env);
    let state = get_state(env);
    let class_obj = runtime_class_from_object(ofClass, get_state(env),&get_frame(env));
    let field_classfile = check_inited_class(state, &ClassName::Str("java/lang/reflect/Field".to_string()), frame.clone().into(), frame.class_pointer.loader.clone());
    let mut object_array = vec![];
    &class_obj.clone().unwrap().classfile.fields.iter().enumerate().for_each(|(i, f)| {
        push_new_object(state,frame.clone(), &field_classfile);
        let field_object = frame.pop();
        //todo so this is big and messy put I don't really see a way to simplify
        object_array.push(field_object.clone());
        let field_class_name_ = class_obj.clone().as_ref().unwrap().class_view.name();
        load_class_constant_by_type(state, &frame, &PTypeView::Ref(ReferenceTypeView::Class(field_class_name_)));
        let parent_runtime_class = frame.pop();
        let field_name = class_obj.clone().unwrap().classfile.constant_pool[f.name_index as usize].extract_string_from_utf8();
        create_string_on_stack(state, &frame, field_name);
        let field_name_string = frame.pop();

        let field_desc_str = class_obj.clone().unwrap().classfile.constant_pool[f.descriptor_index as usize].extract_string_from_utf8();
        let field_type = parse_field_descriptor(field_desc_str.as_str()).unwrap().field_type;
        let field_type_class = ptype_to_class_object(state, &frame, &field_type);

        let modifiers = JavaValue::Int(f.access_flags as i32);
        let slot = JavaValue::Int(i as i32);

        create_string_on_stack(state, &frame, field_desc_str);
        let signature_string = frame.pop();

        //todo impl annotations.
        let annotations = JavaValue::Object(Some(Arc::new(Object::Array(ArrayObject { elems: RefCell::new(vec![]), elem_type: PTypeView::ByteType }))));

        run_constructor(
            state,
            frame.clone(),
            field_classfile.clone(),
            vec![field_object, parent_runtime_class, field_name_string, JavaValue::Object(field_type_class), modifiers, slot, signature_string, annotations],
            "(Ljava/lang/Class;Ljava/lang/String;Ljava/lang/Class;IILjava/lang/String;[B)V".to_string(),
        )
    });

    let res = Some(Arc::new(
        Object::Array(ArrayObject {
            elem_type: PTypeView::Ref(ReferenceTypeView::Class(class_name(&field_classfile.classfile))),
            elems: RefCell::new(object_array),
        })));
    to_object(res)
}


