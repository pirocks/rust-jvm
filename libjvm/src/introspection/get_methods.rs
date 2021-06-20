use std::cell::RefCell;
use std::ops::Deref;
use std::ptr::null_mut;
use std::sync::Arc;

use classfile_view::loading::{LoaderIndex, LoaderName};
use classfile_view::view::{ClassView, HasAccessFlags};
use classfile_view::view::method_view::MethodView;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use jvmti_jni_bindings::{_jobject, jboolean, jclass, jint, jio_vfprintf, JNIEnv, jobjectArray};
use rust_jvm_common::classfile::ACC_PUBLIC;
use rust_jvm_common::classnames::{class_name, ClassName};
use slow_interpreter::class_loading::check_initing_or_inited_class;
use slow_interpreter::instructions::ldc::load_class_constant_by_type;
use slow_interpreter::interpreter::WasException;
use slow_interpreter::interpreter_state::InterpreterStateGuard;
use slow_interpreter::interpreter_util::{push_new_object, run_constructor};
use slow_interpreter::java::lang::class::JClass;
use slow_interpreter::java::lang::reflect::constructor::Constructor;
use slow_interpreter::java::lang::reflect::method::Method;
use slow_interpreter::java::lang::string::JString;
use slow_interpreter::java_values::{ArrayObject, JavaValue, Object};
use slow_interpreter::jvm_state::JVMState;
use slow_interpreter::runtime_class::RuntimeClass;
use slow_interpreter::rust_jni::interface::local_frame::new_local_ref_public;
use slow_interpreter::rust_jni::interface::misc::get_all_methods;
use slow_interpreter::rust_jni::native_util::{from_jclass, from_object, get_interpreter_state, get_state, to_object};
use slow_interpreter::stack_entry::StackEntry;

#[no_mangle]
unsafe extern "system" fn JVM_GetClassDeclaredMethods(env: *mut JNIEnv, ofClass: jclass, publicOnly: jboolean) -> jobjectArray {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let loader = int_state.current_loader().clone();
    let of_class_obj = JavaValue::Object(todo!()/*from_jclass(jvm,ofClass)*/).cast_class().expect("todo");
    let int_state = get_interpreter_state(env);
    match JVM_GetClassDeclaredMethods_impl(jvm, int_state, publicOnly, loader, of_class_obj) {
        Ok(res) => res,
        Err(_) => null_mut()
    }
}

fn JVM_GetClassDeclaredMethods_impl(jvm: &'_ JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, '_>, publicOnly: u8, loader: LoaderName, of_class_obj: JClass<'gc_life>) -> Result<jobjectArray, WasException> {
    let class_ptype = &of_class_obj.as_type(jvm);
    if class_ptype.is_array() || class_ptype.is_primitive() {
        unimplemented!()
    }
    let runtime_class = of_class_obj.as_runtime_class(jvm);
    let runtime_class_view = runtime_class.view();
    let methods = runtime_class_view.methods().map(|method| (runtime_class.clone(), method.method_i()));
    let method_class = check_initing_or_inited_class(jvm, int_state, ClassName::method().into())?;
    let mut object_array = vec![];
    methods.filter(|(c, i)| {
        let c_view = c.view();
        let method_view = c_view.method_view_i(*i);
        if publicOnly > 0 {
            method_view.is_public()
        } else {
            let name = method_view.name();
            name != "<clinit>" && name != "<init>"
        }
    }).for_each(|(c, i)| {
        let c_view = c.view();
        let method_view = c_view.method_view_i(i);
        let method = Method::method_object_from_method_view(jvm, int_state, &method_view).expect("todo");
        object_array.push(method.java_value());
    });
    let res = jvm.allocate_object(Object::object_array(jvm, int_state, object_array, method_class.view().type_())?).into();
    unsafe { Ok(new_local_ref_public(res, int_state)) }
}


#[no_mangle]
unsafe extern "system" fn JVM_GetClassDeclaredConstructors(env: *mut JNIEnv, ofClass: jclass, publicOnly: jboolean) -> jobjectArray {
    let jvm = get_state(env);
    let temp1 = from_object(jvm, ofClass);
    let class_obj = JavaValue::Object(temp1).cast_class().expect("todo");
    let class_type = class_obj.as_type(jvm);
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    match JVM_GetClassDeclaredConstructors_impl(jvm, int_state, &class_obj.as_runtime_class(jvm), publicOnly > 0, class_type) {
        Ok(res) => res,
        Err(WasException {}) => null_mut()
    }
}

fn JVM_GetClassDeclaredConstructors_impl(jvm: &'_ JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, '_>, class_obj: &RuntimeClass, publicOnly: bool, class_type: PTypeView) -> Result<jobjectArray, WasException> {
    if class_type.is_array() || class_type.is_primitive() {
        dbg!(class_type.is_primitive());
        unimplemented!()
    }
    let target_classview = &class_obj.view();
    let constructors = target_classview.lookup_method_name(&"<init>".to_string());
    let loader = int_state.current_loader().clone();
    let mut object_array = vec![];

    constructors.iter().filter(|m| {
        if publicOnly {
            m.is_public()
        } else {
            true
        }
    }).for_each(|m| {
        let constructor = Constructor::constructor_object_from_method_view(jvm, int_state, &m).expect("todo");
        object_array.push(constructor.java_value())
    });
    let res = jvm.allocate_object(Object::object_array(jvm, int_state, object_array, ClassName::constructor().into())?).into();
    Ok(unsafe { new_local_ref_public(res, int_state) })
}
