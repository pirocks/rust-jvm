use std::os::raw::{c_char, c_uchar};
use std::ptr::null_mut;
use std::sync::Arc;
use itertools::Itertools;
use wtf8::Wtf8Buf;

use classfile_view::view::constant_info_view::ConstantInfoView;
use classfile_view::view::ptype_view::PTypeView;
use jvmti_jni_bindings::{_jobject, jclass, jdouble, jfloat, jint, jlong, JNIEnv, jobject, jobjectArray, jstring, JVM_CONSTANT_Class, JVM_CONSTANT_Double, JVM_CONSTANT_Fieldref, JVM_CONSTANT_Float, JVM_CONSTANT_Integer, JVM_CONSTANT_InterfaceMethodref, JVM_CONSTANT_InvokeDynamic, JVM_CONSTANT_Long, JVM_CONSTANT_MethodHandle, JVM_CONSTANT_Methodref, JVM_CONSTANT_MethodType, JVM_CONSTANT_NameAndType, JVM_CONSTANT_String, JVM_CONSTANT_Utf8};
use runtime_class_stuff::RuntimeClass;
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::CPDType;
use rust_jvm_common::compressed_classfile::field_names::FieldName;
use rust_jvm_common::compressed_classfile::method_names::MethodName;
use rust_jvm_common::descriptor_parser::parse_field_descriptor;
use slow_interpreter::better_java_stack::frames::PushableFrame;
use slow_interpreter::class_loading::check_initing_or_inited_class;
use slow_interpreter::class_objects::get_or_create_class_object;
use slow_interpreter::exceptions::WasException;
use slow_interpreter::java_values::{ExceptionReturn, JavaValue};
use slow_interpreter::jvm_state::JVMState;
use slow_interpreter::new_java_values::java_value_common::JavaValueCommon;
use slow_interpreter::rust_jni::jni_utils::{get_throw, new_local_ref_public_new};
use slow_interpreter::rust_jni::jni_utils::{get_interpreter_state, get_state};
use slow_interpreter::rust_jni::native_util::{from_jclass, to_object, to_object_new};
use slow_interpreter::stdlib::java::lang::reflect::method::Method;
use slow_interpreter::stdlib::java::lang::string::JString;
use slow_interpreter::stdlib::java::NewAsObjectOrJavaValue;
use slow_interpreter::stdlib::sun::reflect::constant_pool::ConstantPool;
use slow_interpreter::throw_utils::{throw_array_out_of_bounds, throw_array_out_of_bounds_res, throw_illegal_arg, throw_illegal_arg_res};
use slow_interpreter::utils::{field_object_from_view, pushable_frame_todo};

//todo lots of duplication here, idk if should fix though

#[no_mangle]
unsafe extern "system" fn JVM_GetClassConstantPool(env: *mut JNIEnv, cls: jclass) -> jobject {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let constant_pool = match ConstantPool::new(jvm, int_state, from_jclass(jvm, cls)) {
        Ok(constant_pool) => constant_pool,
        Err(WasException { exception_obj }) => {
            *get_throw(env) = Some(WasException { exception_obj });
            return jobject::invalid_default();
        }
    };
    to_object_new(constant_pool.full_object_ref().into())
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetSize(env: *mut JNIEnv, _constantPoolOop: jobject, jcpool: jobject) -> jint {
    let jvm = get_state(env);
    let runtime_class = from_jclass(jvm, jcpool).as_runtime_class(jvm);
    let view = runtime_class.view();
    view.constant_pool_size() as jint
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetClassAt(env: *mut JNIEnv, _constantPoolOop: jobject, jcpool: jobject, index: jint) -> jclass {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let throw = get_throw(env);
    let rc = from_jclass(jvm, jcpool).as_runtime_class(jvm);
    let view = rc.view();
    if index >= view.constant_pool_size() as jint {
        return throw_array_out_of_bounds(jvm, int_state, throw, index);
    }
    match view.constant_pool_view(index as usize) {
        ConstantInfoView::Class(c) => match get_or_create_class_object(jvm, c.class_ref_type(&jvm.string_pool).to_cpdtype(), pushable_frame_todo()) {
            Ok(class_obj) => to_object(class_obj.to_gc_managed().into()),
            Err(_) => null_mut(),
        },
        _ => {
            return throw_illegal_arg(jvm, int_state, throw);
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetClassAtIfLoaded(env: *mut JNIEnv, _constantPoolOop: jobject, jcpool: jobject, index: jint) -> jclass {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let throw = get_throw(env);
    let rc = from_jclass(jvm, jcpool).as_runtime_class(jvm);
    let view = rc.view();
    if index >= view.constant_pool_size() as jint {
        return throw_array_out_of_bounds(jvm, int_state, throw, index);
    }
    match view.constant_pool_view(index as usize) {
        ConstantInfoView::Class(_c) => {
            let classes_guard = jvm.classes.read().unwrap();
            match classes_guard.get_class_obj(rc.cpdtype(), None /*todo should there be something here*/) {
                None => null_mut(),
                Some(obj) => to_object(obj.to_gc_managed().into()),
            }
        }
        _ => {
            return throw_illegal_arg(jvm, int_state, throw);
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetMethodAt(env: *mut JNIEnv, _constantPoolOop: jobject, jcpool: jobject, index: jint) -> jobject {
    match get_method(env, jcpool, index, true) {
        Ok(method) => method,
        Err(WasException { exception_obj }) => {
            *get_throw(env) = Some(WasException { exception_obj });
            jobject::invalid_default()
        }
    }
}

fn get_class_from_type_maybe<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, ptype: CPDType, load_class: bool) -> Result<Option<Arc<RuntimeClass<'gc>>>, WasException<'gc>> {
    Ok(if load_class {
        Some(check_initing_or_inited_class(jvm, int_state, ptype)?)
    } else {
        match jvm.classes.read().unwrap().get_class_obj(ptype, None /*todo should this be something*/) {
            None => return Ok(None),
            Some(_) => Some(JavaValue::Object(todo!() /*rc.into()*/).to_new().cast_class().unwrap().as_runtime_class(jvm)),
        }
    })
}

unsafe fn get_method<'gc>(env: *mut JNIEnv, jcpool: jobject, index: i32, load_class: bool) -> Result<jobject, WasException<'gc>> {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let rc = from_jclass(jvm, jcpool).as_runtime_class(jvm);
    let view = rc.view();
    if index >= view.constant_pool_size() as jint {
        throw_array_out_of_bounds_res(jvm, int_state, index)?;
        unreachable!()
    }
    let method_obj = match view.constant_pool_view(index as usize) {
        ConstantInfoView::Methodref(method_ref) => {
            let method_ref_class = match get_class_from_type_maybe(jvm, int_state, method_ref.class(&jvm.string_pool).to_cpdtype(), load_class)? {
                None => return Ok(null_mut()),
                Some(method_ref_class) => method_ref_class,
            };
            let name = method_ref.name_and_type().name(&jvm.string_pool);
            let method_desc = method_ref.name_and_type().desc_method(&jvm.string_pool);
            let view = method_ref_class.view();
            let method_view = view.lookup_method(MethodName(name), &method_desc).unwrap();
            Method::method_object_from_method_view(jvm, int_state, &method_view)?
        }
        ConstantInfoView::InterfaceMethodref(method_ref) => {
            let method_ref_class = match get_class_from_type_maybe(jvm, int_state, method_ref.class().to_cpdtype(), load_class)? {
                None => return Ok(null_mut()),
                Some(method_ref_class) => method_ref_class,
            };
            let name = method_ref.name_and_type().name(&jvm.string_pool);
            let method_desc = method_ref.name_and_type().desc_method(&jvm.string_pool);
            let view = method_ref_class.view();
            let method_view = view.lookup_method(MethodName(name), &method_desc).unwrap();
            Method::method_object_from_method_view(jvm, int_state, &method_view)?
        }
        _ => {
            return throw_illegal_arg_res(jvm, int_state);
        }
    };

    Ok(new_local_ref_public_new(Some(method_obj.object().as_allocated_obj()), int_state))
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetMethodAtIfLoaded(env: *mut JNIEnv, _constantPoolOop: jobject, jcpool: jobject, index: jint) -> jobject {
    match get_method(env, jcpool, index, false) {
        Ok(method) => method,
        Err(WasException { exception_obj }) => {
            *get_throw(env) = Some(WasException{ exception_obj });
            null_mut()
        }
    }
}

unsafe fn get_field<'gc>(env: *mut JNIEnv, jcpool: jobject, index: i32, load_class: bool) -> Result<jobject, WasException<'gc>> {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let rc = from_jclass(jvm, jcpool).as_runtime_class(jvm);
    let view = rc.view();
    if index >= view.constant_pool_size() as jint {
        throw_array_out_of_bounds_res(jvm, int_state, index)?;
        unreachable!()
    }
    match view.constant_pool_view(index as usize) {
        ConstantInfoView::Fieldref(field_ref) => {
            let field_rc = match get_class_from_type_maybe(jvm, int_state, CPDType::from_ptype(&parse_field_descriptor(field_ref.class().as_str()).unwrap().field_type, &jvm.string_pool), load_class)? {
                None => return Ok(null_mut()),
                Some(field_rc) => field_rc,
            };
            let name = field_ref.name_and_type().name(&jvm.string_pool);
            let view = field_rc.view();
            let field_view = view.fields().find(|f| f.field_name() == FieldName(name)).unwrap();
            let method_obj = field_object_from_view(jvm, int_state, field_rc, field_view)?;
            Ok(new_local_ref_public_new(method_obj.unwrap_object().as_ref().map(|handle| handle.as_allocated_obj()), todo!()/*int_state*/))
        }
        _ => {
            return throw_illegal_arg_res(jvm, int_state);
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetFieldAt(env: *mut JNIEnv, _constantPoolOop: jobject, jcpool: jobject, index: jint) -> jobject {
    match get_field(env, jcpool, index, true) {
        Ok(method) => method,
        Err(WasException { exception_obj }) => {
            *get_throw(env) = Some(WasException{ exception_obj });
            null_mut()
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetFieldAtIfLoaded(env: *mut JNIEnv, _constantPoolOop: jobject, jcpool: jobject, index: jint) -> jobject {
    match get_field(env, jcpool, index, false) {
        Ok(method) => method,
        Err(WasException { exception_obj }) => {
            *get_throw(env) = Some(WasException{ exception_obj });
            null_mut()
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetMemberRefInfoAt(env: *mut JNIEnv, _constantPoolOop: jobject, jcpool: jobject, index: jint) -> jobjectArray {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let throw = get_throw(env);
    let rc = from_jclass(jvm, jcpool).as_runtime_class(jvm);
    let view = rc.view();
    if index >= view.constant_pool_size() as jint {
        return throw_array_out_of_bounds(jvm, int_state, throw, index);
    }
    let (class, name, desc_str) = match view.constant_pool_view(index as usize) {
        ConstantInfoView::Methodref(ref_) => {
            let class = PTypeView::from_compressed(ref_.class(&jvm.string_pool).to_cpdtype(), &jvm.string_pool).class_name_representation().replace(".", "/");
            let name = ref_.name_and_type().name(&jvm.string_pool);
            let desc_str = ref_.name_and_type().desc_str(&jvm.string_pool);
            (class, name, desc_str)
        }
        ConstantInfoView::InterfaceMethodref(ref_) => {
            let class = PTypeView::from_compressed(ref_.class().to_cpdtype(), &jvm.string_pool).class_name_representation().replace(".", "/");
            let name = ref_.name_and_type().name(&jvm.string_pool);
            let desc_str = ref_.name_and_type().desc_str(&jvm.string_pool);
            (class, name, desc_str)
        }
        ConstantInfoView::Fieldref(ref_) => {
            let class = ref_.class();
            let name = ref_.name_and_type().name(&jvm.string_pool);
            let desc_str = ref_.name_and_type().desc_str(&jvm.string_pool);
            (class, name, desc_str)
        }
        _ => {
            return throw_illegal_arg(jvm, int_state, throw);
        }
    };
    let jv_vec_owned = vec![
        match JString::from_rust(jvm, int_state, Wtf8Buf::from_string(class)) {
            Ok(class) => class.new_java_value_handle(),
            Err(WasException { exception_obj }) => {
                *throw = Some(WasException { exception_obj });
                return null_mut();
            }
        },
        match JString::from_rust(jvm, int_state, Wtf8Buf::from_string(name.to_str(&jvm.string_pool))) {
            Ok(name) => name.new_java_value_handle(),
            Err(WasException { exception_obj }) => {
                *throw = Some(WasException { exception_obj });
                return null_mut();
            }
        },
        match JString::from_rust(jvm, int_state, Wtf8Buf::from_string(desc_str.to_str(&jvm.string_pool))) {
            Ok(desc_str) => desc_str.new_java_value_handle(),
            Err(WasException { exception_obj }) => {
                *throw = Some(WasException { exception_obj });
                return null_mut();
            }
        },
    ];
    new_local_ref_public_new(Some(JavaValue::new_vec_from_vec(jvm, jv_vec_owned.iter().map(|owned| owned.as_njv()).collect_vec(), CClassName::string().into()).as_allocated_obj()), int_state)
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetIntAt(env: *mut JNIEnv, _constantPoolOop: jobject, jcpool: jobject, index: jint) -> jint {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let throw = get_throw(env);
    let rc = from_jclass(jvm, jcpool).as_runtime_class(jvm);
    let view = rc.view();
    if index >= view.constant_pool_size() as jint {
        return throw_array_out_of_bounds(jvm, int_state, throw, index);
    }
    match view.constant_pool_view(index as usize) {
        ConstantInfoView::Integer(int_) => int_.int,
        _ => {
            return throw_illegal_arg(jvm, int_state, throw);
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetLongAt(env: *mut JNIEnv, _constantPoolOop: jobject, jcpool: jobject, index: jint) -> jlong {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let throw = get_throw(env);
    let rc = from_jclass(jvm, jcpool).as_runtime_class(jvm);
    let view = rc.view();
    if index >= view.constant_pool_size() as jint {
        return throw_array_out_of_bounds(jvm, int_state, throw, index);
    }
    match view.constant_pool_view(index as usize) {
        ConstantInfoView::Long(long_) => long_.long,
        _ => {
            return throw_illegal_arg(jvm, int_state, throw);
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetFloatAt(env: *mut JNIEnv, _constantPoolOop: jobject, jcpool: jobject, index: jint) -> jfloat {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let throw = get_throw(env);
    let rc = from_jclass(jvm, jcpool).as_runtime_class(jvm);
    let view = rc.view();
    if index >= view.constant_pool_size() as jint {
        return throw_array_out_of_bounds(jvm, int_state, throw, index);
    }
    match view.constant_pool_view(index as usize) {
        ConstantInfoView::Float(float_) => float_.float,
        _ => {
            return throw_illegal_arg(jvm, int_state, throw);
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetDoubleAt(env: *mut JNIEnv, _constantPoolOop: jobject, jcpool: jobject, index: jint) -> jdouble {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let throw = get_throw(env);
    let rc = from_jclass(jvm, jcpool).as_runtime_class(jvm);
    let view = rc.view();
    if index >= view.constant_pool_size() as jint {
        return throw_array_out_of_bounds(jvm, int_state, throw, index);
    }
    match view.constant_pool_view(index as usize) {
        ConstantInfoView::Double(double_) => double_.double,
        _ => {
            return throw_illegal_arg(jvm, int_state, throw);
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetStringAt(env: *mut JNIEnv, constantPoolOop: jobject, _jcpool: jobject, index: jint) -> jstring {
    match ConstantPoolGetStringAt_impl(env, constantPoolOop, index) {
        Ok(res) => res,
        Err(_) => null_mut(),
    }
}

unsafe fn ConstantPoolGetStringAt_impl<'gc>(env: *mut JNIEnv, _constantPoolOop: *mut _jobject, _index: i32) -> Result<jobject, WasException<'gc>> {
    let jvm = get_state(env);
    let _int_state = get_interpreter_state(env);
    let _rc = from_jclass(jvm, todo!()/*jcpool*/).as_runtime_class(jvm);
    let view = _rc.view();
    if _index >= view.constant_pool_size() as jint {
        throw_array_out_of_bounds_res(jvm, _int_state, _index)?;
    }
    match view.constant_pool_view(_index as usize) {
        ConstantInfoView::String(_string) => Ok(to_object(todo!()/*JString::from_rust(jvm, pushable_frame_todo(), string.string())?.object().to_gc_managed().into()*/)),
        _ => {
            return throw_illegal_arg_res(jvm, _int_state);
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetUTF8At(env: *mut JNIEnv, _constantPoolOop: jobject, jcpool: jobject, index: jint) -> jstring {
    match ConstantPoolGetUTF8At_impl(env, jcpool, jcpool, index) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            *get_throw(env) = Some(WasException { exception_obj });
            return jstring::invalid_default();
        }
    }
}

unsafe fn ConstantPoolGetUTF8At_impl<'gc>(env: *mut JNIEnv, _constantPoolOop: jobject, jcpool: jclass, index: i32) -> Result<jobject, WasException<'gc>> {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let rc = from_jclass(jvm, jcpool).as_runtime_class(jvm);
    let view = rc.view();
    if index >= view.constant_pool_size() as jint {
        throw_array_out_of_bounds_res(jvm, int_state, index)?;
    }
    match view.constant_pool_view(index as usize) {
        ConstantInfoView::Utf8(utf8) => Ok(to_object_new(JString::from_rust(jvm, int_state, utf8.str.clone())?.full_object_ref().into())),
        _ => {
            return throw_illegal_arg_res(jvm, int_state);
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassCPTypes(env: *mut JNIEnv, cb: jclass, types: *mut c_uchar) {
    let jvm = get_state(env);
    let rc = from_jclass(jvm, cb).as_runtime_class(jvm);
    let view = rc.view();
    for (i, constant_pool) in (0..view.constant_pool_size()).map(|i| (i, view.constant_pool_view(i))) {
        types.offset(i as isize).write(match constant_pool {
            ConstantInfoView::Utf8(_) => JVM_CONSTANT_Utf8,
            ConstantInfoView::Integer(_) => JVM_CONSTANT_Integer,
            ConstantInfoView::Float(_) => JVM_CONSTANT_Float,
            ConstantInfoView::Long(_) => JVM_CONSTANT_Long,
            ConstantInfoView::Double(_) => JVM_CONSTANT_Double,
            ConstantInfoView::Class(_) => JVM_CONSTANT_Class,
            ConstantInfoView::String(_) => JVM_CONSTANT_String,
            ConstantInfoView::Fieldref(_) => JVM_CONSTANT_Fieldref,
            ConstantInfoView::Methodref(_) => JVM_CONSTANT_Methodref,
            ConstantInfoView::InterfaceMethodref(_) => JVM_CONSTANT_InterfaceMethodref,
            ConstantInfoView::NameAndType(_) => JVM_CONSTANT_NameAndType,
            ConstantInfoView::MethodHandle(_) => JVM_CONSTANT_MethodHandle,
            ConstantInfoView::MethodType(_) => JVM_CONSTANT_MethodType,
            ConstantInfoView::InvokeDynamic(_) => JVM_CONSTANT_InvokeDynamic,
            ConstantInfoView::LiveObject(_) => panic!(),
        } as c_uchar)
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassCPEntriesCount(env: *mut JNIEnv, cb: jclass) -> jint {
    let jvm = get_state(env);
    let rc = from_jclass(jvm, cb).as_runtime_class(jvm);
    let view = rc.view();
    view.constant_pool_size() as i32
}

#[no_mangle]
unsafe extern "system" fn JVM_GetCPFieldNameUTF(env: *mut JNIEnv, cb: jclass, index: jint) -> *const c_char {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let throw = get_throw(env);
    let rc = from_jclass(jvm, cb).as_runtime_class(jvm);
    let view = rc.view();
    if index >= view.constant_pool_size() as jint {
        return throw_array_out_of_bounds(jvm, int_state, throw, index);
    }
    match view.constant_pool_view(index as usize) {
        ConstantInfoView::Fieldref(ref_) => jvm.native.native_interface_allocations.allocate_modified_string(ref_.name_and_type().name(&jvm.string_pool).to_str(&jvm.string_pool)),
        _ => {
            return throw_illegal_arg(jvm, int_state, throw);
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_GetCPMethodNameUTF(env: *mut JNIEnv, cb: jclass, index: jint) -> *const c_char {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let throw = get_throw(env);
    let rc = from_jclass(jvm, cb).as_runtime_class(jvm);
    let view = rc.view();
    if index >= view.constant_pool_size() as jint {
        return throw_array_out_of_bounds(jvm, int_state, throw, index);
    }
    match view.constant_pool_view(index as usize) {
        ConstantInfoView::Methodref(ref_) => jvm.native.native_interface_allocations.allocate_modified_string(ref_.name_and_type().name(&jvm.string_pool).to_str(&jvm.string_pool)),
        ConstantInfoView::InterfaceMethodref(ref_) => jvm.native.native_interface_allocations.allocate_modified_string(ref_.name_and_type().name(&jvm.string_pool).to_str(&jvm.string_pool)),
        _ => {
            return throw_illegal_arg(jvm, int_state, throw);
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_GetCPMethodSignatureUTF(env: *mut JNIEnv, cb: jclass, index: jint) -> *const c_char {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let throw = get_throw(env);
    let rc = from_jclass(jvm, cb).as_runtime_class(jvm);
    let view = rc.view();
    if index >= view.constant_pool_size() as jint {
        return throw_array_out_of_bounds(jvm, int_state, throw, index);
    }
    match view.constant_pool_view(index as usize) {
        ConstantInfoView::Methodref(ref_) => jvm.native.native_interface_allocations.allocate_modified_string(ref_.name_and_type().desc_str(&jvm.string_pool).to_str(&jvm.string_pool)),
        ConstantInfoView::InterfaceMethodref(ref_) => jvm.native.native_interface_allocations.allocate_modified_string(ref_.name_and_type().desc_str(&jvm.string_pool).to_str(&jvm.string_pool)),
        _ => {
            return throw_illegal_arg(jvm, int_state, throw);
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_GetCPFieldSignatureUTF(env: *mut JNIEnv, cb: jclass, index: jint) -> *const c_char {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let throw = get_throw(env);
    let rc = from_jclass(jvm, cb).as_runtime_class(jvm);
    let view = rc.view();
    if index >= view.constant_pool_size() as jint {
        return throw_array_out_of_bounds(jvm, int_state, throw, index);
    }
    match view.constant_pool_view(index as usize) {
        ConstantInfoView::Fieldref(ref_) => jvm.native.native_interface_allocations.allocate_modified_string(ref_.name_and_type().desc_str(&jvm.string_pool).to_str(&jvm.string_pool)),
        _ => {
            return throw_illegal_arg(jvm, int_state, throw);
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_GetCPClassNameUTF(env: *mut JNIEnv, cb: jclass, index: jint) -> *const c_char {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let throw = get_throw(env);
    let rc = from_jclass(jvm, cb).as_runtime_class(jvm);
    let view = rc.view();
    if index >= view.constant_pool_size() as jint {
        return throw_array_out_of_bounds(jvm, int_state, throw, index);
    }
    match view.constant_pool_view(index as usize) {
        ConstantInfoView::Class(class_) => jvm.native.native_interface_allocations.allocate_modified_string(PTypeView::from_compressed(class_.class_ref_type(&jvm.string_pool).to_cpdtype(), &jvm.string_pool).class_name_representation()),
        _ => {
            return throw_illegal_arg(jvm, int_state, throw);
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_GetCPFieldClassNameUTF(env: *mut JNIEnv, cb: jclass, index: jint) -> *const c_char {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let throw = get_throw(env);
    let rc = from_jclass(jvm, cb).as_runtime_class(jvm);
    let view = rc.view();
    if index >= view.constant_pool_size() as jint {
        return throw_array_out_of_bounds(jvm, int_state, throw, index);
    }
    match view.constant_pool_view(index as usize) {
        ConstantInfoView::Fieldref(ref_) => jvm.native.native_interface_allocations.allocate_modified_string(ref_.name_and_type().name(&jvm.string_pool).to_str(&jvm.string_pool)),
        _ => {
            return throw_illegal_arg(jvm, int_state, throw);
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_GetCPMethodClassNameUTF(env: *mut JNIEnv, cb: jclass, index: jint) -> *const c_char {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let throw = get_throw(env);
    let rc = from_jclass(jvm, cb).as_runtime_class(jvm);
    let view = rc.view();
    if index >= view.constant_pool_size() as jint {
        return throw_array_out_of_bounds(jvm, int_state, throw, index);
    }
    match view.constant_pool_view(index as usize) {
        ConstantInfoView::Methodref(ref_) => jvm.native.native_interface_allocations.allocate_modified_string(ref_.name_and_type().name(&jvm.string_pool).to_str(&jvm.string_pool)),
        ConstantInfoView::InterfaceMethodref(ref_) => jvm.native.native_interface_allocations.allocate_modified_string(ref_.name_and_type().name(&jvm.string_pool).to_str(&jvm.string_pool)),
        _ => {
            return throw_illegal_arg(jvm, int_state, throw);
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_GetCPFieldModifiers(_env: *mut JNIEnv, _cb: jclass, _index: jint, _calledClass: jclass) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetCPMethodModifiers(_env: *mut JNIEnv, _cb: jclass, _index: jint, _calledClass: jclass) -> jint {
    unimplemented!()
}
