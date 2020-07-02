use std::cell::RefCell;
use std::sync::Arc;
use classfile_view::view::ptype_view::{ReferenceTypeView, PTypeView};
use rust_jvm_common::classnames::{class_name, ClassName};
use slow_interpreter::rust_jni::native_util::{to_object, get_frame, get_state, from_object, from_jclass};
use slow_interpreter::interpreter_util::{run_constructor, push_new_object, check_inited_class};
use slow_interpreter::instructions::ldc::{create_string_on_stack, load_class_constant_by_type};
use rust_jvm_common::classfile::ACC_PUBLIC;
use jvmti_jni_bindings::{JNIEnv, jclass, jboolean, jobjectArray, jio_vfprintf};
use slow_interpreter::rust_jni::get_all_methods;
use libjvm_utils::ptype_to_class_object;
use classfile_view::view::HasAccessFlags;
use classfile_view::view::method_view::MethodView;

use slow_interpreter::java_values::{JavaValue, Object, ArrayObject};
use slow_interpreter::JVMState;
use slow_interpreter::stack_entry::StackEntry;
use std::ops::Deref;
use slow_interpreter::java::lang::class::JClass;
use slow_interpreter::runtime_class::RuntimeClass;
use classfile_view::loading::{Loader, LoaderArc};
use slow_interpreter::get_state_thread_frame;
use slow_interpreter::rust_jni::native_util::{get_frames, get_thread};

const METHOD_SIGNATURE: &'static str = "(Ljava/lang/Class;Ljava/lang/String;[Ljava/lang/Class;Ljava/lang/Class;[Ljava/lang/Class;IILjava/lang/String;[B[B[B)V";

#[no_mangle]
unsafe extern "system" fn JVM_GetClassDeclaredMethods(env: *mut JNIEnv, ofClass: jclass, publicOnly: jboolean) -> jobjectArray {
    get_state_thread_frame!(env,jvm,thread,frames,frame);
    let loader = frame.class_pointer.loader(jvm).clone();
    let of_class_obj = JavaValue::Object(from_object(ofClass)).cast_class();
    JVM_GetClassDeclaredMethods_impl(jvm,frame,publicOnly, loader, of_class_obj)
}

fn JVM_GetClassDeclaredMethods_impl(jvm: &'static JVMState, frame: &mut StackEntry, publicOnly: u8, loader: LoaderArc, of_class_obj: JClass) -> jobjectArray {
    let class_ptype = &of_class_obj.as_type();
    if class_ptype.is_array() || class_ptype.is_primitive() {
        unimplemented!()
    }
    let runtime_class = of_class_obj.as_runtime_class();
    let methods = get_all_methods(jvm, frame, runtime_class);
    let method_class = check_inited_class(jvm, &ClassName::method().into(), loader.clone());
    let mut object_array = vec![];
    //todo do we need to filter out constructors?
    methods.iter().filter(|(c, i)| {
        if publicOnly > 0 {
            c.view().method_view_i(*i).is_public()
        } else {
            true
        }
    }).for_each(|(c, i)| {
        //todo dupe?
        push_new_object(jvm, frame, &method_class, None);
        let method_object = frame.pop();
        object_array.push(method_object.clone());


        let method_view = c.view().method_view_i(*i);
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
            let field_class_name = c.view().name();
            //todo so if we are calling this on int.class that is caught by the unimplemented above.
            load_class_constant_by_type(jvm, frame, &PTypeView::Ref(ReferenceTypeView::Class(field_class_name)));
            frame.pop()
        };
        let name = {
            let name = method_view.name();
            create_string_on_stack(jvm, name);
            frame.pop()
        };
        let parameterTypes = parameters_type_objects(jvm, frame, &method_view);
        let returnType = {
            let rtype = method_view.desc().return_type;
            JavaValue::Object(ptype_to_class_object(jvm, frame, &rtype))
        };
        let exceptionTypes = exception_types_table(jvm, frame, &method_view);
        let modifiers = get_modifers(&method_view);
        //todo what does slot do?
        let slot = JavaValue::Int(-1);
        let signature = get_signature(jvm, frame, &method_view);
        let annotations = JavaValue::empty_byte_array(jvm);
        let parameterAnnotations = JavaValue::empty_byte_array(jvm);
        let annotationDefault = JavaValue::empty_byte_array(jvm);
        let full_args = vec![method_object, clazz, name, parameterTypes, returnType, exceptionTypes, modifiers, slot, signature, annotations, parameterAnnotations, annotationDefault];
        //todo replace with wrapper object
        run_constructor(jvm, frame, method_class.clone(), full_args, METHOD_SIGNATURE.to_string());
    });
    let res = Arc::new(Object::object_array(jvm, object_array, PTypeView::Ref(ReferenceTypeView::Class(method_class.view().name())))).into();
    unsafe {to_object(res)}
}

fn get_signature(state: &'static JVMState, frame: &mut StackEntry, method_view: &MethodView) -> JavaValue {
    create_string_on_stack(state, method_view.desc_str());
    frame.pop()
}

fn exception_types_table(jvm: &'static JVMState, frame: &mut StackEntry, method_view: &MethodView) -> JavaValue {
    let class_type = PTypeView::Ref(ReferenceTypeView::Class(ClassName::class()));//todo this should be a global const
    let exception_table: Vec<JavaValue> = method_view.code_attribute()
        .map(|x| &x.exception_table)
        .unwrap_or(&vec![])
        .iter()
        .map(|x| x.catch_type)
        .map(|x| if x == 0 {
            ReferenceTypeView::Class(ClassName::throwable())
        } else {
            method_view.classview().constant_pool_view(x as usize).unwrap_class().class_name()
        })
        .map(|x| {
            PTypeView::Ref(x)
        })
        .map(|x| {
            ptype_to_class_object(jvm, frame, &x.to_ptype()).into()
        })
        .collect();
    JavaValue::Object(Some(Arc::new(Object::Array(ArrayObject {
        elems: RefCell::new(exception_table),
        elem_type: class_type.clone(),
        monitor: jvm.thread_state.new_monitor("".to_string()),
    }))))
}

fn parameters_type_objects(jvm: &'static JVMState, frame: &mut StackEntry, method_view: &MethodView) -> JavaValue {
    let class_type = PTypeView::Ref(ReferenceTypeView::Class(ClassName::class()));//todo this should be a global const
    let mut res = vec![];
    let parsed = method_view.desc();
    for param_type in parsed.parameter_types {
        res.push(JavaValue::Object(ptype_to_class_object(jvm, frame, &param_type)));
    }

    JavaValue::Object(Some(Arc::new(Object::Array(ArrayObject {
        elems: RefCell::new(res),
        elem_type: class_type.clone(),
        monitor: jvm.thread_state.new_monitor("".to_string())
    }))))
}


const CONSTRUCTOR_SIGNATURE: &'static str = "(Ljava/lang/Class;[Ljava/lang/Class;[Ljava/lang/Class;IILjava/lang/String;[B[B)V";

#[no_mangle]
unsafe extern "system" fn JVM_GetClassDeclaredConstructors(env: *mut JNIEnv, ofClass: jclass, publicOnly: jboolean) -> jobjectArray {
    get_state_thread_frame!(env,jvm,thread,frames,frame);
    let temp1 = from_object(ofClass);
    let class_obj = JavaValue::Object(temp1).cast_class();
    let class_type = class_obj.as_type();
    JVM_GetClassDeclaredConstructors_impl(jvm,frame, &class_obj.as_runtime_class(), publicOnly > 0, class_type)
}

fn JVM_GetClassDeclaredConstructors_impl(jvm: &'static JVMState, frame: &mut StackEntry, class_obj: &RuntimeClass, publicOnly: bool, class_type: PTypeView) -> jobjectArray {
    if class_type.is_array() || class_type.is_primitive() {
        dbg!(class_type.is_primitive());
        unimplemented!()
    }
    let target_classview = &class_obj.view();
    let constructors = target_classview.lookup_method_name(&"<init>".to_string());
    let loader = frame.class_pointer.loader(jvm).clone();
    let constructor_class = check_inited_class(jvm, &ClassName::new("java/lang/reflect/Constructor").into(), loader.clone());
    let mut object_array = vec![];

    constructors.iter().filter(|m| {
        if publicOnly {
            m.is_public()
        } else {
            true
        }
    }).for_each(|m| {
        push_new_object(jvm, frame, &constructor_class, None);
        let constructor_object = frame.pop();
        object_array.push(constructor_object.clone());

        let method_view = m;

        let clazz = {
            let field_class_name = class_obj.view().name();
            //todo this doesn't cover the full generality of this, b/c we could be calling on int.class or array classes
            load_class_constant_by_type(jvm, frame, &PTypeView::Ref(ReferenceTypeView::Class(field_class_name.clone())));
            frame.pop()
        };

        let parameter_types = parameters_type_objects(jvm, frame, &method_view);
        let exceptionTypes = exception_types_table(jvm, frame, &method_view);
        let modifiers = get_modifers(&method_view);
        //todo what does slot do?
        let slot = JavaValue::Int(-1);
        let signature = get_signature(jvm, frame, &method_view);

        //todo impl these
        let empty_byte_array = JavaValue::empty_byte_array(jvm);

        let full_args = vec![constructor_object, clazz, parameter_types, exceptionTypes, modifiers, slot, signature, empty_byte_array.clone(), empty_byte_array];
        run_constructor(jvm, frame, constructor_class.clone(), full_args, CONSTRUCTOR_SIGNATURE.to_string())
    });
    let res = Arc::new(Object::object_array(jvm, object_array, PTypeView::Ref(ReferenceTypeView::Class(constructor_class.view().name())))).into();
    unsafe {to_object(res)}
}


fn get_modifers(method_view: &MethodView) -> JavaValue {
    JavaValue::Int(method_view.access_flags() as i32)
}
