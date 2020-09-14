use std::cell::RefCell;
use std::ops::Deref;
use std::sync::Arc;

use classfile_view::loading::{Loader, LoaderArc};
use classfile_view::view::HasAccessFlags;
use classfile_view::view::method_view::MethodView;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use jvmti_jni_bindings::{jboolean, jclass, jio_vfprintf, JNIEnv, jobjectArray};
use libjvm_utils::ptype_to_class_object;
use rust_jvm_common::classfile::ACC_PUBLIC;
use rust_jvm_common::classnames::{class_name, ClassName};
use slow_interpreter::instructions::ldc::load_class_constant_by_type;
use slow_interpreter::interpreter_state::InterpreterStateGuard;
use slow_interpreter::interpreter_util::{check_inited_class, push_new_object, run_constructor};
use slow_interpreter::java::lang::class::JClass;
use slow_interpreter::java::lang::string::JString;
use slow_interpreter::java_values::{ArrayObject, JavaValue, Object};
use slow_interpreter::jvm_state::JVMState;
use slow_interpreter::runtime_class::RuntimeClass;
use slow_interpreter::rust_jni::interface::local_frame::new_local_ref_public;
use slow_interpreter::rust_jni::interface::misc::get_all_methods;
use slow_interpreter::rust_jni::native_util::{from_jclass, from_object, get_interpreter_state, get_state, to_object};
use slow_interpreter::stack_entry::StackEntry;

const METHOD_SIGNATURE: &'static str = "(Ljava/lang/Class;Ljava/lang/String;[Ljava/lang/Class;Ljava/lang/Class;[Ljava/lang/Class;IILjava/lang/String;[B[B[B)V";

#[no_mangle]
unsafe extern "system" fn JVM_GetClassDeclaredMethods(env: *mut JNIEnv, ofClass: jclass, publicOnly: jboolean) -> jobjectArray {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let loader = int_state.current_loader(jvm).clone();
    let of_class_obj = JavaValue::Object(from_object(ofClass)).cast_class();
    let int_state = get_interpreter_state(env);
    JVM_GetClassDeclaredMethods_impl(jvm, int_state, publicOnly, loader, of_class_obj)
}

fn JVM_GetClassDeclaredMethods_impl(jvm: &JVMState, int_state: &mut InterpreterStateGuard, publicOnly: u8, loader: LoaderArc, of_class_obj: JClass) -> jobjectArray {
    let class_ptype = &of_class_obj.as_type();
    if class_ptype.is_array() || class_ptype.is_primitive() {
        unimplemented!()
    }
    let runtime_class = of_class_obj.as_runtime_class();
    let methods = get_all_methods(jvm, int_state, runtime_class);
    let method_class = check_inited_class(jvm, int_state, &ClassName::method().into(), loader.clone());
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
        push_new_object(jvm, int_state, &method_class, None);
        let method_object = int_state.pop_current_operand_stack();
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
            load_class_constant_by_type(jvm, int_state, &PTypeView::Ref(ReferenceTypeView::Class(field_class_name)));
            int_state.pop_current_operand_stack()
        };
        let name = {
            let name = method_view.name();
            JString::from(jvm, int_state, name).intern(jvm, int_state).java_value()
        };
        let parameterTypes = parameters_type_objects(jvm, int_state, &method_view);
        let returnType = {
            let rtype = method_view.desc().return_type;
            JavaValue::Object(ptype_to_class_object(jvm, int_state, &rtype))
        };
        let exceptionTypes = exception_types_table(jvm, int_state, &method_view);
        let modifiers = get_modifers(&method_view);
        //todo what does slot do?
        let slot = JavaValue::Int(-1);
        let signature = get_signature(jvm, int_state, &method_view);
        let annotations = JavaValue::empty_byte_array(jvm, int_state);
        let parameterAnnotations = JavaValue::empty_byte_array(jvm, int_state);
        let annotationDefault = JavaValue::empty_byte_array(jvm, int_state);
        let full_args = vec![method_object, clazz, name, parameterTypes, returnType, exceptionTypes, modifiers, slot, signature, annotations, parameterAnnotations, annotationDefault];
        //todo replace with wrapper object
        run_constructor(jvm, int_state, method_class.clone(), full_args, METHOD_SIGNATURE.to_string());
    });
    let res = Arc::new(Object::object_array(jvm, int_state, object_array, PTypeView::Ref(ReferenceTypeView::Class(method_class.view().name())))).into();
    unsafe { new_local_ref_public(res, int_state) }
}

fn get_signature(state: &JVMState, int_state: &mut InterpreterStateGuard, method_view: &MethodView) -> JavaValue {
    JString::from(state, int_state, method_view.desc_str()).intern(state, int_state).java_value()
}

fn exception_types_table(jvm: &JVMState, int_state: &mut InterpreterStateGuard, method_view: &MethodView) -> JavaValue {
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
            ptype_to_class_object(jvm, int_state, &x.to_ptype()).into()
        })
        .collect();
    JavaValue::Object(Some(Arc::new(Object::Array(ArrayObject::new_array(
        jvm,
        int_state,
        exception_table,
        class_type,
        jvm.thread_state.new_monitor("".to_string()),
    )))))
}

fn parameters_type_objects(jvm: &JVMState, int_state: &mut InterpreterStateGuard, method_view: &MethodView) -> JavaValue {
    let class_type = PTypeView::Ref(ReferenceTypeView::Class(ClassName::class()));//todo this should be a global const
    let mut res = vec![];
    let parsed = method_view.desc();
    for param_type in parsed.parameter_types {
        res.push(JavaValue::Object(ptype_to_class_object(jvm, int_state, &param_type)));
    }

    JavaValue::Object(Some(Arc::new(Object::Array(ArrayObject::new_array(
        jvm,
        int_state,
        res,
        class_type,
        jvm.thread_state.new_monitor("".to_string()),
    )))))
}


const CONSTRUCTOR_SIGNATURE: &'static str = "(Ljava/lang/Class;[Ljava/lang/Class;[Ljava/lang/Class;IILjava/lang/String;[B[B)V";

#[no_mangle]
unsafe extern "system" fn JVM_GetClassDeclaredConstructors(env: *mut JNIEnv, ofClass: jclass, publicOnly: jboolean) -> jobjectArray {
    let temp1 = from_object(ofClass);
    let class_obj = JavaValue::Object(temp1).cast_class();
    let class_type = class_obj.as_type();
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    JVM_GetClassDeclaredConstructors_impl(jvm, int_state, &class_obj.as_runtime_class(), publicOnly > 0, class_type)
}

fn JVM_GetClassDeclaredConstructors_impl(jvm: &JVMState, int_state: &mut InterpreterStateGuard, class_obj: &RuntimeClass, publicOnly: bool, class_type: PTypeView) -> jobjectArray {
    if class_type.is_array() || class_type.is_primitive() {
        dbg!(class_type.is_primitive());
        unimplemented!()
    }
    let target_classview = &class_obj.view();
    let constructors = target_classview.lookup_method_name(&"<init>".to_string());
    let loader = int_state.current_loader(jvm).clone();
    let constructor_class = check_inited_class(jvm, int_state, &ClassName::new("java/lang/reflect/Constructor").into(), loader.clone());
    let mut object_array = vec![];

    constructors.iter().filter(|m| {
        if publicOnly {
            m.is_public()
        } else {
            true
        }
    }).for_each(|m| {
        push_new_object(jvm, int_state, &constructor_class, None);
        let constructor_object = int_state.pop_current_operand_stack();
        object_array.push(constructor_object.clone());

        let method_view = m;

        let clazz = {
            let field_class_name = class_obj.view().name();
            //todo this doesn't cover the full generality of this, b/c we could be calling on int.class or array classes
            load_class_constant_by_type(jvm, int_state, &PTypeView::Ref(ReferenceTypeView::Class(field_class_name.clone())));
            int_state.pop_current_operand_stack()
        };

        let parameter_types = parameters_type_objects(jvm, int_state, &method_view);
        let exceptionTypes = exception_types_table(jvm, int_state, &method_view);
        let modifiers = get_modifers(&method_view);
        //todo what does slot do?
        let slot = JavaValue::Int(-1);
        let signature = get_signature(jvm, int_state, &method_view);

        //todo impl these
        let empty_byte_array = JavaValue::empty_byte_array(jvm, int_state);

        let full_args = vec![constructor_object, clazz, parameter_types, exceptionTypes, modifiers, slot, signature, empty_byte_array.clone(), empty_byte_array];
        run_constructor(jvm, int_state, constructor_class.clone(), full_args, CONSTRUCTOR_SIGNATURE.to_string())
    });
    let res = Arc::new(Object::object_array(jvm, int_state, object_array, PTypeView::Ref(ReferenceTypeView::Class(constructor_class.view().name())))).into();
    unsafe { new_local_ref_public(res, int_state) }
}


fn get_modifers(method_view: &MethodView) -> JavaValue {
    JavaValue::Int(method_view.access_flags() as i32)
}
