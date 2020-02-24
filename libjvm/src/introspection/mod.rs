use jni_bindings::{jobjectArray, jclass, JNIEnv, jobject, jint, jstring, jbyteArray, jboolean, JVM_ExceptionTableEntryType};
use slow_interpreter::rust_jni::native_util::{to_object, get_state, get_frame, from_object};
use std::sync::Arc;
use runtime_common::java_values::{Object, ArrayObject, JavaValue};
use std::cell::RefCell;
use rust_jvm_common::unified_types::{PType, ReferenceType};
use rust_jvm_common::classnames::{class_name, ClassName};
use slow_interpreter::interpreter_util::{run_constructor, push_new_object, check_inited_class};
use slow_interpreter::instructions::ldc::{create_string_on_stack, load_class_constant_by_name};
use runtime_common::{StackEntry, InterpreterState};
use std::rc::Rc;
use slow_interpreter::{array_of_type_class, get_or_create_class_object};
use rust_jvm_common::classfile::ACC_PUBLIC;
use std::ops::Deref;
use std::ffi::CStr;
use slow_interpreter::rust_jni::interface::util::runtime_class_from_object;
use slow_interpreter::rust_jni::interface::string::new_string_with_string;
use descriptor_parser::{parse_method_descriptor, parse_field_descriptor};
use rust_jvm_common::view::ptype_view::{PTypeView, ReferenceTypeView};
use libjvm_utils::ptype_to_class_object;

pub mod constant_pool;
pub mod is_x;
pub mod index;
pub mod method_annotations;

#[no_mangle]
unsafe extern "system" fn JVM_GetClassInterfaces(env: *mut JNIEnv, cls: jclass) -> jobjectArray {
    unimplemented!()
}


#[no_mangle]
unsafe extern "system" fn JVM_GetClassSigners(env: *mut JNIEnv, cls: jclass) -> jobjectArray {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetProtectionDomain(env: *mut JNIEnv, cls: jclass) -> jobject {
    unimplemented!()
}


#[no_mangle]
unsafe extern "system" fn JVM_GetComponentType(env: *mut JNIEnv, cls: jclass) -> jclass {
    let state = get_state(env);
    let frame = get_frame(env);
    let object_non_null = from_object(cls).unwrap().clone();
    let object_class = object_non_null.unwrap_normal_object().array_class_object_pointer.borrow();
    to_object(ptype_to_class_object(state,&frame,object_class.as_ref().unwrap()))
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassModifiers(env: *mut JNIEnv, cls: jclass) -> jint {
    runtime_class_from_object(cls).unwrap().classfile.access_flags as jint
}

#[no_mangle]
unsafe extern "system" fn JVM_GetDeclaredClasses(env: *mut JNIEnv, ofClass: jclass) -> jobjectArray {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetDeclaringClass(env: *mut JNIEnv, ofClass: jclass) -> jclass {
    //todo unimplemented for now
    std::ptr::null_mut()
    //unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassSignature(env: *mut JNIEnv, cls: jclass) -> jstring {
    unimplemented!()
}


#[no_mangle]
unsafe extern "system" fn JVM_GetClassDeclaredMethods(env: *mut JNIEnv, ofClass: jclass, publicOnly: jboolean) -> jobjectArray {
    unimplemented!()
}


const CONSTRUCTOR_SIGNATURE: &'static str = "(Ljava/lang/Class;[Ljava/lang/Class;[Ljava/lang/Class;IILjava/lang/String;[B[B)V";

#[no_mangle]
unsafe extern "system" fn JVM_GetClassDeclaredConstructors(env: *mut JNIEnv, ofClass: jclass, publicOnly: jboolean) -> jobjectArray {
    let temp = runtime_class_from_object(ofClass).unwrap();
    let target_classfile = &temp.classfile;
    let constructors = target_classfile.lookup_method_name(&"<init>".to_string());
    let state = get_state(env);
    let frame = get_frame(env);
    let class_obj = runtime_class_from_object(ofClass);
    let loader = frame.class_pointer.loader.clone();
    let constructor_class = check_inited_class(state, &ClassName::new("java/lang/reflect/Constructor"), frame.clone().into(), loader.clone());
    let mut object_array = vec![];

    constructors.clone().iter().filter(|(i, m)| {
        if publicOnly > 0 {
            m.access_flags & ACC_PUBLIC > 0
        } else {
            true
        }
    }).for_each(|(i, m)| {
        let class_type = PTypeView::Ref(ReferenceTypeView::Class(ClassName::class()));//todo this should be a global const

        push_new_object(frame.clone(), &constructor_class);
        let constructor_object = frame.pop();

        object_array.push(constructor_object.clone());

        let clazz = {
            let field_class_name_ = class_name(&class_obj.clone().as_ref().unwrap().classfile);
            let field_class_name = field_class_name_.get_referred_name();
            load_class_constant_by_name(state, &frame, &ReferenceTypeView::Class(ClassName::Str(field_class_name.clone())));
            frame.pop()
        };

        let parameter_types = {
            let mut res = vec![];
            let desc_str = m.descriptor_str(&target_classfile);
            let parsed = parse_method_descriptor(desc_str.as_str()).unwrap();
            for param_type in parsed.parameter_types {
                res.push(JavaValue::Object(ptype_to_class_object(state,&frame,&param_type.to_ptype())));
            }

            JavaValue::Object(Some(Arc::new(Object::Array(ArrayObject { elems: RefCell::new(res), elem_type: class_type.clone() }))))
        };


        let exceptionTypes = {
            //todo not currently supported
            assert!(m.code_attribute().unwrap().exception_table.is_empty());
            JavaValue::Object(Some(Arc::new(Object::Array(ArrayObject { elems: RefCell::new(vec![]), elem_type: class_type.clone() }))))
        };

        let modifiers = JavaValue::Int(constructor_class.classfile.access_flags as i32);
        //todo what does slot do?
        let slot = JavaValue::Int(-1);

        let signature = {
            create_string_on_stack(state, &frame, m.descriptor_str(&target_classfile));
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

#[no_mangle]
unsafe extern "system" fn JVM_GetClassAccessFlags(env: *mut JNIEnv, cls: jclass) -> jint {
    runtime_class_from_object(cls).unwrap().classfile.access_flags as i32
}


#[no_mangle]
unsafe extern "system" fn JVM_ClassDepth(env: *mut JNIEnv, name: jstring) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassContext(env: *mut JNIEnv) -> jobjectArray {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassNameUTF(env: *mut JNIEnv, cb: jclass) -> *const ::std::os::raw::c_char {
    unimplemented!()
}

pub mod fields;
pub mod methods;


#[no_mangle]
unsafe extern "system" fn JVM_GetClassMethodsCount(env: *mut JNIEnv, cb: jclass) -> jint {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_GetCallerClass(env: *mut JNIEnv, depth: ::std::os::raw::c_int) -> jclass {
    /*todo, so this is needed for booting but it is what could best be described as an advanced feature.
    Therefore it only sorta works*/
    let frame = get_frame(env);
    let state = get_state(env);

    load_class_constant_by_name(state, &frame, &ReferenceTypeView::Class(frame.last_call_stack.as_ref().unwrap().class_pointer.class_view.name()));
    let jclass = frame.pop().unwrap_object();
    to_object(jclass)
}


#[no_mangle]
unsafe extern "system" fn JVM_IsSameClassPackage(env: *mut JNIEnv, class1: jclass, class2: jclass) -> jboolean {
    unimplemented!()
}



#[no_mangle]
unsafe extern "system" fn JVM_FindClassFromCaller(
    env: *mut JNIEnv,
    c_name: *const ::std::os::raw::c_char,
    init: jboolean,
    loader: jobject,
    caller: jclass,
) -> jclass {
    let state = get_state(env);
    let frame = get_frame(env);

    let name = CStr::from_ptr(&*c_name).to_str().unwrap().to_string();
    to_object(Some(get_or_create_class_object(state, &ReferenceTypeView::Class(ClassName::Str(name)), frame.clone(), frame.class_pointer.loader.clone())))
}


#[no_mangle]
unsafe extern "system" fn JVM_GetClassName(env: *mut JNIEnv, cls: jclass) -> jstring {
    let obj = runtime_class_from_object(cls).unwrap();
    let full_name = class_name(&obj.classfile).get_referred_name().replace("/", ".");
    new_string_with_string(env, full_name)
}
