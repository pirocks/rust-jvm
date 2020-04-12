use std::cell::RefCell;
use std::sync::Arc;
use classfile_view::view::ptype_view::{ReferenceTypeView, PTypeView};
use rust_jvm_common::classnames::{class_name, ClassName};
use slow_interpreter::rust_jni::native_util::{to_object, get_frame, get_state, from_object};
use slow_interpreter::interpreter_util::{run_constructor, push_new_object, check_inited_class};
use slow_interpreter::instructions::ldc::{create_string_on_stack, load_class_constant_by_type};
use rust_jvm_common::classfile::ACC_PUBLIC;
use jni_bindings::{JNIEnv, jclass, jboolean, jobjectArray};
use slow_interpreter::rust_jni::interface::util::runtime_class_from_object;
use slow_interpreter::rust_jni::get_all_methods;
use libjvm_utils::ptype_to_class_object;
use classfile_view::view::HasAccessFlags;
use classfile_view::view::method_view::MethodView;
use std::rc::Rc;
use slow_interpreter::java_values::{JavaValue, Object, ArrayObject};
use slow_interpreter::JVMState;
use slow_interpreter::stack_entry::StackEntry;

const METHOD_SIGNATURE: &'static str = "(Ljava/lang/Class;Ljava/lang/String;[Ljava/lang/Class;Ljava/lang/Class;[Ljava/lang/Class;IILjava/lang/String;[B[B[B)V";

#[no_mangle]
unsafe extern "system" fn JVM_GetClassDeclaredMethods(env: *mut JNIEnv, ofClass: jclass, publicOnly: jboolean) -> jobjectArray {
    let state = get_state(env);
    let frame = get_frame(env);
    let loader = frame.class_pointer.loader.clone();
    let temp1 = from_object(ofClass).unwrap();
    let temp2 = temp1.unwrap_normal_object().class_object_ptype.borrow();
    let class_ptype = temp2.as_ref().unwrap();
    if class_ptype.is_array() || class_ptype.is_primitive() {
        unimplemented!()
    }
    let runtime_class = runtime_class_from_object(ofClass, state, &frame).unwrap();
    let methods = get_all_methods(state, frame.clone(), runtime_class);
    let method_class = check_inited_class(state, &ClassName::method(), loader.clone());
    let mut object_array = vec![];
    //todo do we need to filter out constructors?
    methods.iter().filter(|(c, i)| {
        if publicOnly > 0 {
            c.class_view.method_view_i(*i).is_public()
        } else {
            true
        }
    }).for_each(|(c, i)| {
        //todo dupe?
        push_new_object(state,frame.clone(), &method_class);
        let method_object = frame.pop();
        object_array.push(method_object.clone());


        let method_view = c.class_view.method_view_i(*i);
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
            let field_class_name = c.class_view.name();
            //todo so if we are calling this on int.class that is caught by the unimplemented above.
            load_class_constant_by_type(state, &frame, &PTypeView::Ref(ReferenceTypeView::Class(field_class_name)));
            frame.pop()
        };
        let name = {
            let name = method_view.name();
            create_string_on_stack(state, &frame, name);
            frame.pop()
        };
        let parameterTypes = parameters_type_objects(state, &frame, &method_view);
        let returnType = {
            let rtype = method_view.desc().return_type;
            JavaValue::Object(ptype_to_class_object(state, &frame, &rtype))
        };
        dbg!(&name);
        dbg!(&parameterTypes);
        dbg!(&returnType);
        let exceptionTypes = exception_types_table(state,&frame,&method_view);
        let modifiers = get_modifers(&method_view);
        //todo what does slot do?
        let slot = JavaValue::Int(-1);
        let signature = get_signature(state, &frame, method_view);
        let annotations = JavaValue::empty_byte_array();
        let parameterAnnotations = JavaValue::empty_byte_array();
        let annotationDefault = JavaValue::empty_byte_array();
        let full_args = vec![method_object, clazz,name, parameterTypes, returnType, exceptionTypes, modifiers, slot, signature, annotations, parameterAnnotations, annotationDefault];
        //todo replace with wrapper object
        run_constructor(state, frame.clone(), method_class.clone(), full_args, METHOD_SIGNATURE.to_string());
    });
    let res = Arc::new(Object::object_array(object_array, PTypeView::Ref(ReferenceTypeView::Class(method_class.class_view.name())))).into();
    to_object(res)
}

fn get_signature(state: & JVMState, frame: &StackEntry, method_view: MethodView) -> JavaValue {
    create_string_on_stack(state, &frame, method_view.desc_str());
    frame.pop()
}

fn exception_types_table(state: & JVMState, frame: &StackEntry, method_view: &MethodView) -> JavaValue {
    let class_type = PTypeView::Ref(ReferenceTypeView::Class(ClassName::class()));//todo this should be a global const
    let exception_table: Vec<JavaValue> = method_view.code_attribute()
        .map(|x|&x.exception_table)
        .unwrap_or(&vec![])
        .iter()
        .map(|x|x.catch_type)
        .map(|x|if x == 0{
            ReferenceTypeView::Class(ClassName::throwable())
        }else {
            method_view.classview().constant_pool_view(x as usize).unwrap_class().class_name()
        })
        .map(|x|{
            PTypeView::Ref(x)
        })
        .map(|x|{
            ptype_to_class_object(state, frame,&x.to_ptype()).into()
        })
        .collect();
    JavaValue::Object(Some(Arc::new(Object::Array(ArrayObject { elems: RefCell::new(exception_table), elem_type: class_type.clone() }))))
}

fn parameters_type_objects(state: & JVMState, frame: &StackEntry, method_view: &MethodView) -> JavaValue {
    let class_type = PTypeView::Ref(ReferenceTypeView::Class(ClassName::class()));//todo this should be a global const
    let mut res = vec![];
    let parsed = method_view.desc();
    for param_type in parsed.parameter_types {
        res.push(JavaValue::Object(ptype_to_class_object(state, &frame, &param_type)));
    }

    JavaValue::Object(Some(Arc::new(Object::Array(ArrayObject { elems: RefCell::new(res), elem_type: class_type.clone() }))))
}


const CONSTRUCTOR_SIGNATURE: &'static str = "(Ljava/lang/Class;[Ljava/lang/Class;[Ljava/lang/Class;IILjava/lang/String;[B[B)V";

#[no_mangle]
unsafe extern "system" fn JVM_GetClassDeclaredConstructors(env: *mut JNIEnv, ofClass: jclass, publicOnly: jboolean) -> jobjectArray {
    let state = get_state(env);
    let frame = get_frame(env);
    let temp1 = from_object(ofClass).unwrap();
    let class_object_non_null = temp1.unwrap_normal_object().clone();
    let class_type = class_object_non_null.class_object_ptype.borrow().as_ref().unwrap().clone();
    if class_type.is_array() || class_type.is_primitive() {
        dbg!(class_type.is_primitive());
        unimplemented!()
    }
    let temp = runtime_class_from_object(ofClass, state, &frame).unwrap();
    let target_classfile = &temp.classfile;
    let constructors = target_classfile.lookup_method_name(&"<init>".to_string());
    let class_obj = runtime_class_from_object(ofClass, state, &frame);
    let loader = frame.class_pointer.loader.clone();
    let constructor_class = check_inited_class(state, &ClassName::new("java/lang/reflect/Constructor"), loader.clone());
    let mut object_array = vec![];

    constructors.clone().iter().filter(|(i, m)| {
        if publicOnly > 0 {
            m.access_flags & ACC_PUBLIC > 0
        } else {
            true
        }
    }).for_each(|(i, _)| {
        push_new_object(state,frame.clone(), &constructor_class);
        let constructor_object = frame.pop();
        object_array.push(constructor_object.clone());

        let method_view = temp.class_view.method_view_i(*i);

        let clazz = {
            let field_class_name = class_obj.as_ref().unwrap().class_view.name();
            //todo this doesn't cover the full generality of this, b/c we could be calling on int.class or array classes
            load_class_constant_by_type(state, &frame, &PTypeView::Ref(ReferenceTypeView::Class(field_class_name.clone())));
            frame.pop()
        };

        let parameter_types = parameters_type_objects(state, &frame, &method_view);
        let exceptionTypes = exception_types_table(state,&frame,&method_view);
        let modifiers = get_modifers(&method_view);
        //todo what does slot do?
        let slot = JavaValue::Int(-1);
        let signature = get_signature(state, &frame, method_view);

        //todo impl these
        let empty_byte_array = JavaValue::empty_byte_array();

        let full_args = vec![constructor_object, clazz, parameter_types, exceptionTypes, modifiers, slot, signature, empty_byte_array.clone(), empty_byte_array];
        run_constructor(state, frame.clone(), constructor_class.clone(), full_args, CONSTRUCTOR_SIGNATURE.to_string())
    });
    let res = Arc::new(Object::object_array(object_array, PTypeView::Ref(ReferenceTypeView::Class(constructor_class.class_view.name())))).into();
    to_object(res)
}


fn get_modifers(method_view: &MethodView) -> JavaValue {
    JavaValue::Int(method_view.access_flags() as i32)
}
