use std::sync::Arc;
use itertools::Either;
use classfile_view::view::ClassView;
use jvmti_jni_bindings::{jclass, jint, JNIEnv, jobjectArray, jstring};
use runtime_class_stuff::RuntimeClass;
use rust_jvm_common::compressed_classfile::compressed_descriptors::{CFieldDescriptor, CompressedFieldDescriptor};
use rust_jvm_common::compressed_classfile::compressed_types::CMethodDescriptor;
use rust_jvm_common::compressed_classfile::field_names::FieldName;
use rust_jvm_common::compressed_classfile::method_names::MethodName;
use rust_jvm_common::compressed_classfile::string_pool::{CCString};
use rust_jvm_common::descriptor_parser::{parse_field_descriptor, parse_method_descriptor};
use rust_jvm_common::mhn_consts::{IS_CONSTRUCTOR, IS_FIELD, IS_METHOD, SEARCH_INTERFACES, SEARCH_SUPERCLASSES};
use slow_interpreter::better_java_stack::frames::PushableFrame;
use slow_interpreter::exceptions::WasException;
use slow_interpreter::java_values::ExceptionReturn;
use slow_interpreter::jvm_state::JVMState;
use slow_interpreter::new_java_values::{NewJavaValueHandle};
use slow_interpreter::new_java_values::allocated_objects::AllocatedHandle;
use slow_interpreter::new_java_values::owned_casts::OwnedCastAble;
use slow_interpreter::rust_jni::jni_utils::{get_interpreter_state, get_state, get_throw};
use slow_interpreter::rust_jni::native_util::from_object_new;
use slow_interpreter::stdlib::java::lang::class::JClass;
use slow_interpreter::stdlib::java::lang::member_name::MemberName;
use slow_interpreter::stdlib::java::lang::reflect::constructor::Constructor;
use slow_interpreter::stdlib::java::lang::reflect::method::Method;
use slow_interpreter::stdlib::java::NewAsObjectOrJavaValue;
use slow_interpreter::throw_utils::throw_illegal_arg_res;
use slow_interpreter::utils::{field_object_from_view, get_all_fields, get_all_methods, unwrap_or_npe};

/// so this is completely undocumented
/// supported match flags IS_METHOD | IS_CONSTRUCTOR |  IS_FIELD | SEARCH_SUPERCLASSES | SEARCH_INTERFACES
///
///
#[no_mangle]
pub unsafe extern "system" fn Java_java_lang_invoke_MethodHandleNatives_getMembers(env: *mut JNIEnv, _: jclass, defc:jclass, match_name: jstring, match_sig: jstring, match_flags: jint, caller: jclass, skip: jint, results: jobjectArray) -> jint{
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    //class member is defined on
    let defc = match unwrap_or_npe(jvm, int_state, from_object_new(jvm, defc)){
        Ok(defc) => defc.cast_class(),
        Err(WasException{ exception_obj }) => {
            *get_throw(env) = Some(WasException{ exception_obj });
            return jint::invalid_default()
        }
    };
    //name to lookup on
    let match_name = NewJavaValueHandle::from_optional_object(from_object_new(jvm, match_name)).cast_string_maybe_null().map(|string| string.to_rust_string(jvm)).map(|str| jvm.string_pool.add_name(str, false));
    //signature to lookup on
    let match_sig = from_object_new(jvm, match_sig).map(|string| string.cast_string().to_rust_string(jvm)).map(|str| jvm.string_pool.add_name(str, false));
    //flags as defined above
    let match_flags = match_flags as u32;
    //caller class for access checks
    let _caller = NewJavaValueHandle::from_optional_object(from_object_new(jvm, caller)).cast_class(); //todo access check
    //seems to be where to start putting in array
    let skip = skip;
    //results arr
    let results = match unwrap_or_npe(jvm, int_state, from_object_new(jvm, results)){
        Ok(results) => results,
        Err(WasException{ exception_obj }) => {
            *get_throw(env) = Some(WasException{ exception_obj });
            return jint::invalid_default()
        }
    };

    match mhn_get_members(jvm, int_state, defc, match_name, match_sig, match_flags, _caller, skip, results){
        Ok(res) => res,
        Err(WasException{ exception_obj }) => {
            *get_throw(env) = Some(WasException{ exception_obj });
            return jint::invalid_default()
        }
    }
}

pub fn mhn_get_members<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, defc: JClass<'gc>, match_name: Option<CCString>, match_sig: Option<CCString>, match_flags: u32, _caller: Option<JClass<'gc>>, skip: jint, results: AllocatedHandle<'gc>) -> Result<jint, WasException<'gc>> {
    let rc = defc.as_runtime_class(jvm);
    let view = rc.view();

    let search_super = (match_flags & SEARCH_SUPERCLASSES) > 0;
    let search_interface = (match_flags & SEARCH_INTERFACES) > 0;
    let is_method = (match_flags & IS_METHOD) > 0;
    let is_field = (match_flags & IS_FIELD) > 0;
    let is_constructor = (match_flags & IS_CONSTRUCTOR) > 0;
    let member_names = match match_sig {
        None => {
            let methods = if is_method { Either::Left(get_matching_methods(jvm, int_state, &match_name.map(|ccstr| MethodName(ccstr)), &rc, &view, search_super, search_interface, is_constructor, None)?.into_iter()) } else { Either::Right(std::iter::empty()) };
            let fields = if is_field { Either::Left(get_matching_fields(jvm, int_state, &match_name.map(|ccstr| FieldName(ccstr)), rc, view, search_super, search_interface, None)?.into_iter()) } else { Either::Right(std::iter::empty()) };
            methods.chain(fields).collect()
        }
        Some(match_sig) => match parse_field_descriptor(match_sig.to_str(&jvm.string_pool).as_str()) {
            None => match parse_method_descriptor(match_sig.to_str(&jvm.string_pool).as_str()) {
                None => {
                    throw_illegal_arg_res(jvm, int_state)?;
                    unreachable!()
                }
                Some(md) => {
                    assert!(is_method);
                    get_matching_methods(jvm, int_state, &match_name.map(|ccstr| MethodName(ccstr)), &rc, &view, search_super, search_interface, is_constructor, Some(CMethodDescriptor::from_legacy(md, &jvm.string_pool)))
                }
            },
            Some(fd) => {
                assert!(is_field);
                get_matching_fields(jvm, int_state, &match_name.map(|ccstr| FieldName(ccstr)), rc, view, search_super, search_interface, Some(CFieldDescriptor::from_legacy(fd, &jvm.string_pool)))
            }
        }?,
    };
    // let res_arr = results.mut_array();
    let mut i = skip;
    let len = member_names.len();
    for member in member_names {
        if i < results.unwrap_array().len() as jint {
            results.unwrap_array().set_i(i, member.new_java_value());
        }
        i += 1;
    }

    Ok(len as i32)
}

fn get_matching_fields<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, match_name: &Option<FieldName>, rc: Arc<RuntimeClass<'gc>>, view: Arc<dyn ClassView>, search_super: bool, search_interface: bool, fd: Option<CFieldDescriptor>) -> Result<Vec<MemberName<'gc>>, WasException<'gc>> {
    let filtered = get_all_fields(jvm, int_state, rc, search_interface)?.into_iter().filter(|(current_rc, method_i)| {
        let current_view = current_rc.view();
        if !search_super {
            if current_view.name() != view.name() {
                return false;
            }
        }
        let field = current_view.field(*method_i);
        (match &match_name {
            None => true,
            Some(match_name) => field.field_name() == *match_name,
        }) && (match &fd {
            None => true,
            Some(CompressedFieldDescriptor(field_type)) => field_type == &field.field_type(),
        })
    });
    let mut res = vec![];
    for (field_class, i) in filtered {
        let view = field_class.view();
        let field_view = view.field(i);
        let field_obj = field_object_from_view(jvm, int_state, field_class, field_view)?;
        res.push(MemberName::new_from_field(jvm, int_state, field_obj.cast_field())?)
    }
    Ok(res)
}

fn get_matching_methods<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, match_name: &Option<MethodName>, rc: &Arc<RuntimeClass<'gc>>, view: &Arc<dyn ClassView>, search_super: bool, search_interface: bool, is_constructor: bool, md: Option<CMethodDescriptor>) -> Result<Vec<MemberName<'gc>>, WasException<'gc>> {
    let filtered = get_all_methods(jvm, int_state, rc.clone(), search_interface)?.into_iter().filter(|(current_rc, method_i)| {
        let current_view = current_rc.view();
        if !search_super {
            if current_view.name() != view.name() {
                return false;
            }
        }
        let method = current_view.method_view_i(*method_i);
        (match &match_name {
            None => true,
            Some(match_name) => match_name == &method.name(),
        }) && (match &md {
            None => true,
            Some(md) => md == method.desc(),
        }) && if is_constructor { method.name() == MethodName::constructor_init() } else { true }
    });
    let mut res = vec![];
    for (method_class, i) in filtered {
        let view = method_class.view();
        let method_view = view.method_view_i(i);
        if method_view.name() == MethodName::constructor_init() {
            let constructor_obj = Constructor::constructor_object_from_method_view(jvm, int_state, &method_view)?;
            res.push(MemberName::new_from_constructor(jvm, int_state, constructor_obj)?)
        } else {
            let method_obj = Method::method_object_from_method_view(jvm, int_state, &method_view)?;
            res.push(MemberName::new_from_method(jvm, int_state, method_obj)?)
        }
    }
    Ok(res)
}
