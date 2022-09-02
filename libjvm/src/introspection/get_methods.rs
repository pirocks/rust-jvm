use std::cell::RefCell;
use std::ops::Deref;
use std::ptr::null_mut;

use itertools::Itertools;

use another_jit_vm_ir::WasException;
use classfile_view::view::{ClassView, HasAccessFlags};
use classfile_view::view::method_view::MethodView;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use jvmti_jni_bindings::{_jobject, jboolean, jclass, jint, jio_vfprintf, JNIEnv, jobjectArray};
use runtime_class_stuff::RuntimeClass;
use rust_jvm_common::classfile::ACC_PUBLIC;
use rust_jvm_common::classnames::{class_name, ClassName};
use rust_jvm_common::compressed_classfile::{CompressedParsedRefType, CPDType};
use rust_jvm_common::compressed_classfile::names::{CClassName, MethodName};
use rust_jvm_common::loading::{LoaderIndex, LoaderName};
use slow_interpreter::better_java_stack::frames::PushableFrame;
use slow_interpreter::better_java_stack::opaque_frame::OpaqueFrame;
use slow_interpreter::class_loading::{assert_inited_or_initing_class, check_initing_or_inited_class};
use slow_interpreter::instructions::ldc::load_class_constant_by_type;
use slow_interpreter::interpreter_state::InterpreterStateGuard;
use slow_interpreter::interpreter_util::{new_object, run_constructor};
use slow_interpreter::java::lang::class::JClass;
use slow_interpreter::java::lang::reflect::constructor::Constructor;
use slow_interpreter::java::lang::reflect::method::Method;
use slow_interpreter::java::lang::string::JString;
use slow_interpreter::java::NewAsObjectOrJavaValue;
use slow_interpreter::java_values::{ArrayObject, JavaValue, Object};
use slow_interpreter::jvm_state::JVMState;
use slow_interpreter::new_java_values::NewJavaValueHandle;
use slow_interpreter::new_java_values::java_value_common::JavaValueCommon;
use slow_interpreter::new_java_values::unallocated_objects::{UnAllocatedObject, UnAllocatedObjectArray};
use slow_interpreter::rust_jni::interface::{get_interpreter_state, get_state};
use slow_interpreter::rust_jni::interface::local_frame::{new_local_ref_public, new_local_ref_public_new};
use slow_interpreter::rust_jni::interface::misc::get_all_methods;
use slow_interpreter::rust_jni::native_util::{from_jclass, from_object, from_object_new, to_object};
use slow_interpreter::stack_entry::StackEntry;

#[no_mangle]
unsafe extern "system" fn JVM_GetClassDeclaredMethods(env: *mut JNIEnv, ofClass: jclass, publicOnly: jboolean) -> jobjectArray {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let loader = int_state.current_loader(jvm);
    let of_class_obj = from_object_new(jvm, ofClass).unwrap().cast_class();
    let int_state = get_interpreter_state(env);
    match JVM_GetClassDeclaredMethods_impl(jvm, todo!()/*int_state*/, publicOnly, loader, of_class_obj) {
        Ok(res) => res,
        Err(_) => null_mut(),
    }
}

fn JVM_GetClassDeclaredMethods_impl<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, publicOnly: u8, loader: LoaderName, of_class_obj: JClass<'gc>) -> Result<jobjectArray, WasException> {
    let class_ptype = &of_class_obj.gc_lifeify().as_type(jvm);
    if class_ptype.is_array() || class_ptype.is_primitive() {
        unimplemented!()
    }
    let runtime_class = of_class_obj.gc_lifeify().as_runtime_class(jvm);
    let runtime_class_view = runtime_class.view();
    let methods = runtime_class_view.methods().map(|method| (runtime_class.clone(), method.method_i()));
    let mut temp : OpaqueFrame<'gc, '_> = todo!();
    let method_class = check_initing_or_inited_class(jvm, /*int_state*/&mut temp, CClassName::method().into())?;
    let mut object_array = vec![];
    let methods_owned = methods
        .filter(|(c, i)| {
            let c_view = c.view();
            let method_view = c_view.method_view_i(*i);
            let name = method_view.name();
            name != MethodName::constructor_clinit() && name != MethodName::constructor_init() && if publicOnly > 0 { method_view.is_public() } else { true }
        })
        .map(|(c, i)| {
            let c_view = c.view();
            let method_view = c_view.method_view_i(i);
            Method::method_object_from_method_view(jvm, int_state, &method_view).expect("todo")
        }).collect_vec();
    for method_owned in methods_owned.iter() {
        object_array.push(method_owned.new_java_value());
    }
    let whole_array_runtime_class = check_initing_or_inited_class(jvm, /*int_state*/&mut temp, CPDType::array(CClassName::method().into())).unwrap();
    let res = jvm.allocate_object(UnAllocatedObject::Array(
        UnAllocatedObjectArray { whole_array_runtime_class, elems: object_array }));
    unsafe { Ok(new_local_ref_public_new(Some(res.as_allocated_obj()), int_state)) }
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassDeclaredConstructors(env: *mut JNIEnv, ofClass: jclass, publicOnly: jboolean) -> jobjectArray {
    let jvm = get_state(env);
    let temp1 = from_object_new(jvm, ofClass);
    let class_obj = NewJavaValueHandle::from_optional_object(temp1).cast_class().expect("todo");
    let class_type = class_obj.as_type(jvm);
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    match JVM_GetClassDeclaredConstructors_impl(jvm, todo!()/*int_state*/, &class_obj.as_runtime_class(jvm), publicOnly > 0, class_type) {
        Ok(res) => res,
        Err(WasException {}) => null_mut(),
    }
}

fn JVM_GetClassDeclaredConstructors_impl<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, class_obj: &RuntimeClass, publicOnly: bool, class_type: CPDType) -> Result<jobjectArray, WasException> {
    if class_type.is_array() || class_type.is_primitive() {
        dbg!(class_type.is_primitive());
        unimplemented!()
    }
    let target_classview = &class_obj.view();
    let constructors = target_classview.lookup_method_name(MethodName::constructor_init());
    let loader = int_state.current_loader(jvm);
    let mut object_array = vec![];

    constructors.iter().filter(|m| if publicOnly { m.is_public() } else { true }).for_each(|m| {
        let constructor = Constructor::constructor_object_from_method_view(jvm, int_state, &m).expect("todo");
        object_array.push(constructor.new_java_value_handle())
    });
    let mut temp : OpaqueFrame<'gc, '_> = todo!();
    let whole_array_runtime_class = check_initing_or_inited_class(jvm, /*int_state*/&mut temp, CPDType::array(CClassName::constructor().into())).unwrap();
    let unallocated = UnAllocatedObject::Array(UnAllocatedObjectArray { whole_array_runtime_class, elems: object_array.iter().map(|handle| handle.as_njv()).collect_vec() });
    let res = jvm.allocate_object(unallocated);
    Ok(unsafe { new_local_ref_public_new(Some(res.as_allocated_obj()), int_state) })
}