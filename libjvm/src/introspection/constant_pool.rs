use std::hint::unreachable_unchecked;
use std::os::raw::{c_char, c_uchar};
use std::ptr::{null, null_mut};
use std::sync::Arc;

use by_address::ByAddress;
use wtf8::Wtf8Buf;

use classfile_view::view::ClassView;
use classfile_view::view::constant_info_view::{ConstantInfoView, InterfaceMethodrefView, MethodrefView};
use classfile_view::view::method_view::MethodView;
use classfile_view::view::ptype_view::PTypeView;
use jvmti_jni_bindings::{_jobject, jclass, jdouble, jfloat, jint, jlong, JNIEnv, jobject, jobjectArray, jstring, JVM_CONSTANT_Class, JVM_CONSTANT_Double, JVM_CONSTANT_Fieldref, JVM_CONSTANT_Float, JVM_CONSTANT_Integer, JVM_CONSTANT_InterfaceMethodref, JVM_CONSTANT_InvokeDynamic, JVM_CONSTANT_Long, JVM_CONSTANT_MethodHandle, JVM_CONSTANT_Methodref, JVM_CONSTANT_MethodType, JVM_CONSTANT_NameAndType, JVM_CONSTANT_String, JVM_CONSTANT_Unicode, JVM_CONSTANT_Utf8, lchmod};
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::compressed_classfile::CPDType;
use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName, MethodName};
use rust_jvm_common::descriptor_parser::parse_field_descriptor;
use rust_jvm_common::loading::{ClassLoadingError, LoaderName};
use slow_interpreter::class_loading::{check_initing_or_inited_class, check_loaded_class};
use slow_interpreter::class_objects::get_or_create_class_object;
use slow_interpreter::interpreter::WasException;
use slow_interpreter::interpreter_state::InterpreterStateGuard;
use slow_interpreter::java::lang::reflect::constant_pool::ConstantPool;
use slow_interpreter::java::lang::reflect::field::Field;
use slow_interpreter::java::lang::reflect::method::Method;
use slow_interpreter::java::lang::string::JString;
use slow_interpreter::java_values::{JavaValue, Object};
use slow_interpreter::jvm_state::JVMState;
use slow_interpreter::runtime_class::RuntimeClass;
use slow_interpreter::rust_jni::interface::field_object_from_view;
use slow_interpreter::rust_jni::interface::local_frame::new_local_ref_public;
use slow_interpreter::rust_jni::native_util::{from_jclass, get_interpreter_state, get_state, to_object};
use slow_interpreter::utils::{throw_array_out_of_bounds, throw_array_out_of_bounds_res, throw_illegal_arg, throw_illegal_arg_res};

//todo lots of duplication here, idk if should fix though

#[no_mangle]
unsafe extern "system" fn JVM_GetClassConstantPool(env: *mut JNIEnv, cls: jclass) -> jobject {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let constant_pool = match ConstantPool::new(jvm, int_state, from_jclass(jvm, cls)) {
        Ok(constant_pool) => constant_pool,
        Err(WasException {}) => return null_mut(),
    };
    to_object(todo!()/*constant_pool.object().to_gc_managed().into()*/)
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetSize(env: *mut JNIEnv, constantPoolOop: jobject, jcpool: jobject) -> jint {
    let jvm = get_state(env);
    let runtime_class = from_jclass(jvm, constantPoolOop).as_runtime_class(jvm);
    let view = runtime_class.view();
    view.constant_pool_size() as jint
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetClassAt(env: *mut JNIEnv, constantPoolOop: jobject, jcpool: jobject, index: jint) -> jclass {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let rc = from_jclass(jvm, constantPoolOop).as_runtime_class(jvm);
    let view = rc.view();
    if index >= view.constant_pool_size() as jint {
        return throw_array_out_of_bounds(jvm, int_state, index);
    }
    match view.constant_pool_view(index as usize) {
        ConstantInfoView::Class(c) => match get_or_create_class_object(jvm, CPDType::Ref(c.class_ref_type()), int_state) {
            Ok(class_obj) => to_object(class_obj.to_gc_managed().into()),
            Err(_) => null_mut(),
        },
        _ => {
            return throw_illegal_arg(jvm, int_state);
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetClassAtIfLoaded(env: *mut JNIEnv, constantPoolOop: jobject, jcpool: jobject, index: jint) -> jclass {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let rc = from_jclass(jvm, constantPoolOop).as_runtime_class(jvm);
    let view = rc.view();
    if index >= view.constant_pool_size() as jint {
        return throw_array_out_of_bounds(jvm, int_state, index);
    }
    match view.constant_pool_view(index as usize) {
        ConstantInfoView::Class(c) => {
            let classes_guard = jvm.classes.read().unwrap();
            match classes_guard.get_class_obj(rc.cpdtype(), None /*todo should there be something here*/) {
                None => null_mut(),
                Some(obj) => to_object(obj.to_gc_managed().into()),
            }
        }
        _ => {
            return throw_illegal_arg(jvm, int_state);
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetMethodAt(env: *mut JNIEnv, constantPoolOop: jobject, jcpool: jobject, index: jint) -> jobject {
    match get_method(env, constantPoolOop, index, true) {
        Ok(method) => method,
        Err(WasException {}) => null_mut(),
    }
}

fn get_class_from_type_maybe(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life,'l>, ptype: CPDType, load_class: bool) -> Result<Option<Arc<RuntimeClass<'gc_life>>>, WasException> {
    Ok(if load_class {
        Some(check_initing_or_inited_class(jvm, int_state, ptype)?)
    } else {
        match jvm.classes.read().unwrap().get_class_obj(ptype, None /*todo should this be something*/) {
            None => return Ok(None),
            Some(rc) => Some(JavaValue::Object(todo!() /*rc.into()*/).to_new().cast_class().unwrap().as_runtime_class(jvm)),
        }
    })
}

unsafe fn get_method(env: *mut JNIEnv, constantPoolOop: jobject, index: i32, load_class: bool) -> Result<jobject, WasException> {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let rc = from_jclass(jvm, constantPoolOop).as_runtime_class(jvm);
    let view = rc.view();
    if index >= view.constant_pool_size() as jint {
        throw_array_out_of_bounds_res(jvm, int_state, index)?;
        unreachable!()
    }
    let method_obj = match view.constant_pool_view(index as usize) {
        ConstantInfoView::Methodref(method_ref) => {
            let method_ref_class = match get_class_from_type_maybe(jvm, int_state, CPDType::Ref(method_ref.class(&jvm.string_pool)), load_class)? {
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
            let method_ref_class = match get_class_from_type_maybe(jvm, int_state, CPDType::Ref(method_ref.class()), load_class)? {
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

    Ok(new_local_ref_public(todo!()/*method_obj.object().to_gc_managed().into()*/, int_state))
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetMethodAtIfLoaded(env: *mut JNIEnv, constantPoolOop: jobject, jcpool: jobject, index: jint) -> jobject {
    match get_method(env, constantPoolOop, index, false) {
        Ok(method) => method,
        Err(WasException {}) => null_mut(),
    }
}

unsafe fn get_field(env: *mut JNIEnv, constantPoolOop: jobject, index: i32, load_class: bool) -> Result<jobject, WasException> {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let rc = from_jclass(jvm, constantPoolOop).as_runtime_class(jvm);
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
            Ok(new_local_ref_public(method_obj.unwrap_object(), int_state))
        }
        _ => {
            return throw_illegal_arg_res(jvm, int_state);
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetFieldAt(env: *mut JNIEnv, constantPoolOop: jobject, jcpool: jobject, index: jint) -> jobject {
    match get_field(env, constantPoolOop, index, true) {
        Ok(method) => method,
        Err(WasException {}) => null_mut(),
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetFieldAtIfLoaded(env: *mut JNIEnv, constantPoolOop: jobject, jcpool: jobject, index: jint) -> jobject {
    match get_field(env, constantPoolOop, index, false) {
        Ok(method) => method,
        Err(WasException {}) => null_mut(),
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetMemberRefInfoAt(env: *mut JNIEnv, constantPoolOop: jobject, jcpool: jobject, index: jint) -> jobjectArray {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let rc = from_jclass(jvm, constantPoolOop).as_runtime_class(jvm);
    let view = rc.view();
    if index >= view.constant_pool_size() as jint {
        return throw_array_out_of_bounds(jvm, int_state, index);
    }
    let (class, name, desc_str) = match view.constant_pool_view(index as usize) {
        ConstantInfoView::Methodref(ref_) => {
            let class = PTypeView::from_compressed(&CPDType::Ref(ref_.class(&jvm.string_pool)), &jvm.string_pool).class_name_representation().replace(".", "/");
            let name = ref_.name_and_type().name(&jvm.string_pool);
            let desc_str = ref_.name_and_type().desc_str(&jvm.string_pool);
            (class, name, desc_str)
        }
        ConstantInfoView::InterfaceMethodref(ref_) => {
            let class = PTypeView::from_compressed(&CPDType::Ref(ref_.class()), &jvm.string_pool).class_name_representation().replace(".", "/");
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
            return throw_illegal_arg(jvm, int_state);
        }
    };
    let jv_vec = vec![
        match JString::from_rust(jvm, int_state, Wtf8Buf::from_string(class)) {
            Ok(class) => class.java_value(),
            Err(WasException {}) => return null_mut(),
        },
        match JString::from_rust(jvm, int_state, Wtf8Buf::from_string(name.to_str(&jvm.string_pool))) {
            Ok(name) => name.java_value(),
            Err(WasException {}) => return null_mut(),
        },
        match JString::from_rust(jvm, int_state, Wtf8Buf::from_string(desc_str.to_str(&jvm.string_pool))) {
            Ok(desc_str) => desc_str.java_value(),
            Err(WasException {}) => return null_mut(),
        },
    ];
    new_local_ref_public(todo!()/*JavaValue::new_vec_from_vec(jvm, jv_vec, CClassName::string().into()).unwrap_object()*/, int_state)
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetIntAt(env: *mut JNIEnv, constantPoolOop: jobject, jcpool: jobject, index: jint) -> jint {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let rc = from_jclass(jvm, constantPoolOop).as_runtime_class(jvm);
    let view = rc.view();
    if index >= view.constant_pool_size() as jint {
        return throw_array_out_of_bounds(jvm, int_state, index);
    }
    match view.constant_pool_view(index as usize) {
        ConstantInfoView::Integer(int_) => int_.int,
        _ => {
            return throw_illegal_arg(jvm, int_state);
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetLongAt(env: *mut JNIEnv, constantPoolOop: jobject, jcpool: jobject, index: jint) -> jlong {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let rc = from_jclass(jvm, constantPoolOop).as_runtime_class(jvm);
    let view = rc.view();
    if index >= view.constant_pool_size() as jint {
        return throw_array_out_of_bounds(jvm, int_state, index);
    }
    match view.constant_pool_view(index as usize) {
        ConstantInfoView::Long(long_) => long_.long,
        _ => {
            return throw_illegal_arg(jvm, int_state);
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetFloatAt(env: *mut JNIEnv, constantPoolOop: jobject, jcpool: jobject, index: jint) -> jfloat {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let rc = from_jclass(jvm, constantPoolOop).as_runtime_class(jvm);
    let view = rc.view();
    if index >= view.constant_pool_size() as jint {
        return throw_array_out_of_bounds(jvm, int_state, index);
    }
    match view.constant_pool_view(index as usize) {
        ConstantInfoView::Float(float_) => float_.float,
        _ => {
            return throw_illegal_arg(jvm, int_state);
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetDoubleAt(env: *mut JNIEnv, constantPoolOop: jobject, jcpool: jobject, index: jint) -> jdouble {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let rc = from_jclass(jvm, constantPoolOop).as_runtime_class(jvm);
    let view = rc.view();
    if index >= view.constant_pool_size() as jint {
        return throw_array_out_of_bounds(jvm, int_state, index);
    }
    match view.constant_pool_view(index as usize) {
        ConstantInfoView::Double(double_) => double_.double,
        _ => {
            return throw_illegal_arg(jvm, int_state);
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetStringAt(env: *mut JNIEnv, constantPoolOop: jobject, jcpool: jobject, index: jint) -> jstring {
    match ConstantPoolGetStringAt_impl(env, constantPoolOop, index) {
        Ok(res) => res,
        Err(_) => null_mut(),
    }
}

unsafe fn ConstantPoolGetStringAt_impl(env: *mut JNIEnv, constantPoolOop: *mut _jobject, index: i32) -> Result<jobject, WasException> {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let rc = from_jclass(jvm, constantPoolOop).as_runtime_class(jvm);
    let view = rc.view();
    if index >= view.constant_pool_size() as jint {
        throw_array_out_of_bounds_res(jvm, int_state, index)?;
    }
    match view.constant_pool_view(index as usize) {
        ConstantInfoView::String(string) => Ok(to_object(todo!()/*JString::from_rust(jvm, int_state, string.string())?.object().to_gc_managed().into()*/)),
        _ => {
            return throw_illegal_arg_res(jvm, int_state);
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_ConstantPoolGetUTF8At(env: *mut JNIEnv, constantPoolOop: jobject, jcpool: jobject, index: jint) -> jstring {
    match ConstantPoolGetUTF8At_impl(env, constantPoolOop, index) {
        Ok(res) => res,
        Err(WasException {}) => null_mut(),
    }
}

unsafe fn ConstantPoolGetUTF8At_impl(env: *mut JNIEnv, constantPoolOop: jobject, index: i32) -> Result<jobject, WasException> {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let rc = from_jclass(jvm, constantPoolOop).as_runtime_class(jvm);
    let view = rc.view();
    if index >= view.constant_pool_size() as jint {
        throw_array_out_of_bounds_res(jvm, int_state, index)?;
    }
    match view.constant_pool_view(index as usize) {
        ConstantInfoView::Utf8(utf8) => Ok(to_object(todo!()/*JString::from_rust(jvm, int_state, utf8.str.clone())?.object().to_gc_managed().into()*/)),
        _ => {
            return throw_illegal_arg_res(jvm, int_state);
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_GetClassCPTypes(env: *mut JNIEnv, cb: jclass, types: *mut c_uchar) {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
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
    let int_state = get_interpreter_state(env);
    let rc = from_jclass(jvm, cb).as_runtime_class(jvm);
    let view = rc.view();
    view.constant_pool_size() as i32
}

#[no_mangle]
unsafe extern "system" fn JVM_GetCPFieldNameUTF(env: *mut JNIEnv, cb: jclass, index: jint) -> *const c_char {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let rc = from_jclass(jvm, cb).as_runtime_class(jvm);
    let view = rc.view();
    if index >= view.constant_pool_size() as jint {
        return throw_array_out_of_bounds(jvm, int_state, index);
    }
    match view.constant_pool_view(index as usize) {
        ConstantInfoView::Fieldref(ref_) => jvm.native.native_interface_allocations.allocate_modified_string(ref_.name_and_type().name(&jvm.string_pool).to_str(&jvm.string_pool)),
        _ => {
            return throw_illegal_arg(jvm, int_state);
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_GetCPMethodNameUTF(env: *mut JNIEnv, cb: jclass, index: jint) -> *const c_char {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let rc = from_jclass(jvm, cb).as_runtime_class(jvm);
    let view = rc.view();
    if index >= view.constant_pool_size() as jint {
        return throw_array_out_of_bounds(jvm, int_state, index);
    }
    match view.constant_pool_view(index as usize) {
        ConstantInfoView::Methodref(ref_) => jvm.native.native_interface_allocations.allocate_modified_string(ref_.name_and_type().name(&jvm.string_pool).to_str(&jvm.string_pool)),
        ConstantInfoView::InterfaceMethodref(ref_) => jvm.native.native_interface_allocations.allocate_modified_string(ref_.name_and_type().name(&jvm.string_pool).to_str(&jvm.string_pool)),
        _ => {
            return throw_illegal_arg(jvm, int_state);
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_GetCPMethodSignatureUTF(env: *mut JNIEnv, cb: jclass, index: jint) -> *const c_char {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let rc = from_jclass(jvm, cb).as_runtime_class(jvm);
    let view = rc.view();
    if index >= view.constant_pool_size() as jint {
        return throw_array_out_of_bounds(jvm, int_state, index);
    }
    match view.constant_pool_view(index as usize) {
        ConstantInfoView::Methodref(ref_) => jvm.native.native_interface_allocations.allocate_modified_string(ref_.name_and_type().desc_str(&jvm.string_pool).to_str(&jvm.string_pool)),
        ConstantInfoView::InterfaceMethodref(ref_) => jvm.native.native_interface_allocations.allocate_modified_string(ref_.name_and_type().desc_str(&jvm.string_pool).to_str(&jvm.string_pool)),
        _ => {
            return throw_illegal_arg(jvm, int_state);
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_GetCPFieldSignatureUTF(env: *mut JNIEnv, cb: jclass, index: jint) -> *const c_char {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let rc = from_jclass(jvm, cb).as_runtime_class(jvm);
    let view = rc.view();
    if index >= view.constant_pool_size() as jint {
        return throw_array_out_of_bounds(jvm, int_state, index);
    }
    match view.constant_pool_view(index as usize) {
        ConstantInfoView::Fieldref(ref_) => jvm.native.native_interface_allocations.allocate_modified_string(ref_.name_and_type().desc_str(&jvm.string_pool).to_str(&jvm.string_pool)),
        _ => {
            return throw_illegal_arg(jvm, int_state);
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_GetCPClassNameUTF(env: *mut JNIEnv, cb: jclass, index: jint) -> *const c_char {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let rc = from_jclass(jvm, cb).as_runtime_class(jvm);
    let view = rc.view();
    if index >= view.constant_pool_size() as jint {
        return throw_array_out_of_bounds(jvm, int_state, index);
    }
    match view.constant_pool_view(index as usize) {
        ConstantInfoView::Class(class_) => jvm.native.native_interface_allocations.allocate_modified_string(PTypeView::from_compressed(&CPDType::Ref(class_.class_ref_type()), &jvm.string_pool).class_name_representation()),
        _ => {
            return throw_illegal_arg(jvm, int_state);
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_GetCPFieldClassNameUTF(env: *mut JNIEnv, cb: jclass, index: jint) -> *const c_char {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let rc = from_jclass(jvm, cb).as_runtime_class(jvm);
    let view = rc.view();
    if index >= view.constant_pool_size() as jint {
        return throw_array_out_of_bounds(jvm, int_state, index);
    }
    match view.constant_pool_view(index as usize) {
        ConstantInfoView::Fieldref(ref_) => jvm.native.native_interface_allocations.allocate_modified_string(ref_.name_and_type().name(&jvm.string_pool).to_str(&jvm.string_pool)),
        _ => {
            return throw_illegal_arg(jvm, int_state);
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_GetCPMethodClassNameUTF(env: *mut JNIEnv, cb: jclass, index: jint) -> *const c_char {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let rc = from_jclass(jvm, cb).as_runtime_class(jvm);
    let view = rc.view();
    if index >= view.constant_pool_size() as jint {
        return throw_array_out_of_bounds(jvm, int_state, index);
    }
    match view.constant_pool_view(index as usize) {
        ConstantInfoView::Methodref(ref_) => jvm.native.native_interface_allocations.allocate_modified_string(ref_.name_and_type().name(&jvm.string_pool).to_str(&jvm.string_pool)),
        ConstantInfoView::InterfaceMethodref(ref_) => jvm.native.native_interface_allocations.allocate_modified_string(ref_.name_and_type().name(&jvm.string_pool).to_str(&jvm.string_pool)),
        _ => {
            return throw_illegal_arg(jvm, int_state);
        }
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_GetCPFieldModifiers(env: *mut JNIEnv, cb: jclass, index: jint, calledClass: jclass) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetCPMethodModifiers(env: *mut JNIEnv, cb: jclass, index: jint, calledClass: jclass) -> jint {
    unimplemented!()
}