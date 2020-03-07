use std::cell::RefCell;
use std::sync::Arc;
use runtime_common::java_values::{Object, ArrayObject, JavaValue};
use classfile_view::view::ptype_view::{ReferenceTypeView, PTypeView};
use rust_jvm_common::classnames::{class_name, ClassName};
use slow_interpreter::rust_jni::native_util::{to_object, get_frame, get_state};
use slow_interpreter::interpreter_util::{run_constructor, push_new_object, check_inited_class};
use slow_interpreter::instructions::ldc::{create_string_on_stack, load_class_constant_by_type};
use rust_jvm_common::classfile::ACC_PUBLIC;
use jni_bindings::{JNIEnv, jclass, jboolean, jobjectArray};
use slow_interpreter::rust_jni::interface::util::runtime_class_from_object;
use slow_interpreter::rust_jni::get_all_methods;
use libjvm_utils::ptype_to_class_object;
use classfile_view::view::HasAccessFlags;

const METHOD_SIGNATURE: &'static str = " (Ljava/lang/Class;Ljava/lang/String;[Ljava/lang/Class;Ljava/lang/Class;[Ljava/lang/Class;IILjava/lang/String;[B[B[B)V";

#[no_mangle]
unsafe extern "system" fn JVM_GetClassDeclaredMethods(env: *mut JNIEnv, ofClass: jclass, publicOnly: jboolean) -> jobjectArray {
    let state = get_state(env);
    let frame = get_frame(env);
    let temp = runtime_class_from_object(ofClass, state, &frame).unwrap();
    let target_classfile = &temp.classfile;
    let methods = get_all_methods(state, frame, temp);
    // let mut object_array = vec![];
    methods.iter().filter(|(c, i)| {
        if publicOnly > 0 {
            c.class_view.method_view_i(*i).is_public()
        } else {
            true
        }
    }).for_each(|(c, i)| {
        //constructor goes:
        // this.clazz = var1;
        // this.name = var2;
        // this.parameterTypes = var3;
        // this.returnType = var4;
        // this.exceptionTypes = var5;
        // this.modifiers = var6;
        // this.slot = var7;
        // this.signature = var8;
        // this.annotations = var9;
        // this.parameterAnnotations = var10;
        // this.annotationDefault = var11;
        let clazz = {
            unimplemented!()
        };
        let name = {};
        let parameterTypes = {};
        let returnType = {};
        unimplemented!()
    });
    unimplemented!()
}


const CONSTRUCTOR_SIGNATURE: &'static str = "(Ljava/lang/Class;[Ljava/lang/Class;[Ljava/lang/Class;IILjava/lang/String;[B[B)V";

#[no_mangle]
unsafe extern "system" fn JVM_GetClassDeclaredConstructors(env: *mut JNIEnv, ofClass: jclass, publicOnly: jboolean) -> jobjectArray {
    let state = get_state(env);
    let frame = get_frame(env);
    let temp = runtime_class_from_object(ofClass, state, &frame).unwrap();
    let target_classfile = &temp.classfile;
    let constructors = target_classfile.lookup_method_name(&"<init>".to_string());
    let class_obj = runtime_class_from_object(ofClass, state, &frame);
    let loader = frame.class_pointer.loader.clone();
    let constructor_class = check_inited_class(state, &ClassName::new("java/lang/reflect/Constructor"), frame.clone().into(), loader.clone());
    let mut object_array = vec![];

    constructors.clone().iter().filter(|(i, m)| {
        if publicOnly > 0 {
            m.access_flags & ACC_PUBLIC > 0
        } else {
            true
        }
    }).for_each(|(i, _)| {
        let class_type = PTypeView::Ref(ReferenceTypeView::Class(ClassName::class()));//todo this should be a global const
        push_new_object(frame.clone(), &constructor_class);
        let constructor_object = frame.pop();
        object_array.push(constructor_object.clone());

        let method_view = temp.class_view.method_view_i(*i);

        let clazz = {
            let field_class_name = class_obj.as_ref().unwrap().class_view.name();
            //todo this doesn't cover the full generality of this, b/c we could be calling on int.class or array classes
            load_class_constant_by_type(state, &frame, &PTypeView::Ref(ReferenceTypeView::Class(field_class_name.clone())));
            frame.pop()
        };

        let parameter_types = {
            let mut res = vec![];
            let parsed = method_view.desc();
            for param_type in parsed.parameter_types {
                res.push(JavaValue::Object(ptype_to_class_object(state, &frame, &param_type.to_ptype())));
            }

            JavaValue::Object(Some(Arc::new(Object::Array(ArrayObject { elems: RefCell::new(res), elem_type: class_type.clone() }))))
        };


        let exceptionTypes = {
            //todo not currently supported
            assert!(method_view.code_attribute().unwrap().exception_table.is_empty());
            JavaValue::Object(Some(Arc::new(Object::Array(ArrayObject { elems: RefCell::new(vec![]), elem_type: class_type.clone() }))))
        };

        let modifiers = JavaValue::Int(constructor_class.classfile.access_flags as i32);
        //todo what does slot do?
        let slot = JavaValue::Int(-1);

        let signature = {
            create_string_on_stack(state, &frame, method_view.desc_str());
            frame.pop()
        };

        //todo impl these
        let empty_byte_array = JavaValue::Object(Some(Arc::new(Object::Array(ArrayObject { elems: RefCell::new(vec![]), elem_type: PTypeView::ByteType }))));

        let full_args = vec![constructor_object, clazz, parameter_types, exceptionTypes, modifiers, slot, signature, empty_byte_array.clone(), empty_byte_array];
        run_constructor(state, frame.clone(), constructor_class.clone(), full_args, CONSTRUCTOR_SIGNATURE.to_string())
    });
    let res = Some(Arc::new(Object::Array(ArrayObject {
        elems: RefCell::new(object_array),
        elem_type: PTypeView::Ref(ReferenceTypeView::Class(class_name(&constructor_class.classfile))),
    })));
    to_object(res)
}
