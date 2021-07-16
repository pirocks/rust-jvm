#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]


use std::sync::Arc;

use itertools::Either;

use classfile_view::view::ClassView;
use rust_jvm_common::compressed_classfile::{CFieldDescriptor, CMethodDescriptor, CompressedFieldDescriptor};
use rust_jvm_common::compressed_classfile::names::{FieldName, MethodName};
use rust_jvm_common::descriptor_parser::{parse_field_descriptor, parse_method_descriptor};

use crate::{InterpreterStateGuard, JVMState};
use crate::interpreter::WasException;
use crate::java::lang::member_name::MemberName;
use crate::java::lang::reflect::constructor::Constructor;
use crate::java::lang::reflect::field::Field;
use crate::java::lang::reflect::method::Method;
use crate::java::lang::string::JString;
use crate::java_values::JavaValue;
use crate::runtime_class::RuntimeClass;
use crate::rust_jni::interface::field_object_from_view;
use crate::rust_jni::interface::misc::{get_all_fields, get_all_methods};
use crate::sun::misc::unsafe_::Unsafe;
use crate::utils::{throw_illegal_arg_res, unwrap_or_npe};

pub mod resolve;

pub fn MHN_getConstant<'gc_life>() -> Result<JavaValue<'gc_life>, WasException> {
    //so I have no idea what this is for, but openjdk does approx this so it should be fine.
    Ok(JavaValue::Int(0))
}

pub const BRIDGE: i32 = 64;
pub const VARARGS: i32 = 128;
pub const SYNTHETIC: i32 = 4096;
pub const ANNOTATION: i32 = 8192;
pub const ENUM: i32 = 16384;
pub const RECOGNIZED_MODIFIERS: i32 = 65535;
pub const IS_METHOD: u32 = 65536;
pub const IS_CONSTRUCTOR: u32 = 131072;
pub const IS_FIELD: u32 = 262144;
pub const IS_TYPE: u32 = 524288;
pub const CALLER_SENSITIVE: i32 = 1048576;
pub const ALL_ACCESS: i32 = 7;
pub const ALL_KINDS: i32 = 983040;
pub const IS_INVOCABLE: i32 = 196608;
pub const IS_FIELD_OR_METHOD: i32 = 327680;
pub const SEARCH_ALL_SUPERS: i32 = 3145728;
pub const REFERENCE_KIND_SHIFT: u32 = 24;
pub const REFERENCE_KIND_MASK: u32 = 0xF;
pub const SEARCH_SUPERCLASSES: u32 = 0x00100000;
pub const SEARCH_INTERFACES: u32 = 0x00200000;

pub mod init;

/// so this is completely undocumented
/// supported match flags IS_METHOD | IS_CONSTRUCTOR |  IS_FIELD | SEARCH_SUPERCLASSES | SEARCH_INTERFACES
///
pub fn Java_java_lang_invoke_MethodHandleNatives_getMembers(
    jvm: &'gc_life JVMState<'gc_life>,
    int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>,
    args: Vec<JavaValue<'gc_life>>,
) -> Result<JavaValue<'gc_life>, WasException> {
    //class member is defined on
    let defc = unwrap_or_npe(jvm, int_state, args[0].cast_class())?;
    //name to lookup on
    let match_name = args[1].cast_string().map(|string| string.to_rust_string(jvm));
    //signature to lookup on
    let matchSig = args[2].cast_string().map(|string| string.to_rust_string(jvm));
    //flags as defined above
    let matchFlags = args[3].unwrap_int() as u32;
    //caller class for access checks
    let _caller = args[4].cast_class();//todo access check
    //seems to be where to start putting in array
    let skip = args[5].unwrap_int();
    //results arr
    let results = args[6].unwrap_array();

    let rc = defc.as_runtime_class(jvm);
    let view = rc.view();


    let search_super = (matchFlags & SEARCH_SUPERCLASSES) > 0;
    let search_interface = (matchFlags & SEARCH_INTERFACES) > 0;
    let is_method = (matchFlags & IS_METHOD) > 0;
    let is_field = (matchFlags & IS_FIELD) > 0;
    let is_constructor = (matchFlags & IS_CONSTRUCTOR) > 0;
    let member_names = match matchSig {
        None => {
            let methods = if is_method {
                Either::Left(get_matching_methods(jvm, int_state, &match_name, &rc, &view, search_super, search_interface, is_constructor, None)?.into_iter())
            } else {
                Either::Right(std::iter::empty())
            };
            let fields = if is_field {
                Either::Left(get_matching_fields(jvm, int_state, &match_name, rc, view, search_super, search_interface, None)?.into_iter())
            } else {
                Either::Right(std::iter::empty())
            };
            methods.chain(fields).collect()
        }
        Some(matchSig) => {
            match parse_field_descriptor(matchSig.as_str()) {
                None => {
                    match parse_method_descriptor(matchSig.as_str()) {
                        None => {
                            throw_illegal_arg_res(jvm, int_state)?;
                            unreachable!()
                        }
                        Some(md) => {
                            assert!(is_method);
                            get_matching_methods(jvm, int_state, &match_name, &rc, &view, search_super, search_interface, is_constructor, Some(md))
                        }
                    }
                }
                Some(fd) => {
                    assert!(is_field);
                    get_matching_fields(jvm, int_state, &match_name, rc, view, search_super, search_interface, Some(fd))
                }
            }?
        }
    };
    // let res_arr = results.mut_array();
    let mut i = skip;
    let len = member_names.len();
    for member in member_names {
        if i < results.len() {
            results.set_i(jvm, i, member.java_value());
        }
        i += 1;
    }

    Ok(JavaValue::Int(len as i32))
}

fn get_matching_fields(
    jvm: &'gc_life JVMState<'gc_life>,
    int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>,
    match_name: &Option<FieldName>,
    rc: Arc<RuntimeClass<'gc_life>>,
    view: Arc<dyn ClassView>,
    search_super: bool,
    search_interface: bool,
    fd: Option<CFieldDescriptor>,
) -> Result<Vec<MemberName<'gc_life>>, WasException> {
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
            Some(match_name) => field.field_name() == match_name
        }) && (match &fd {
            None => true,
            Some(CompressedFieldDescriptor(field_type)) => field_type == &field.field_type()
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

fn get_matching_methods(
    jvm: &'gc_life JVMState<'gc_life>,
    int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>,
    match_name: &Option<MethodName>,
    rc: &Arc<RuntimeClass<'gc_life>>,
    view: &Arc<dyn ClassView>,
    search_super: bool,
    search_interface: bool,
    is_constructor: bool,
    md: Option<CMethodDescriptor>,
) -> Result<Vec<MemberName<'gc_life>>, WasException> {
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
            Some(match_name) => match_name == &method.name()
        }) && (match &md {
            None => true,
            Some(md) => md == method.desc()
        }) && if is_constructor {
            method.name() == MethodName::constructor_init()
        } else {
            true
        }
    });
    let mut res = vec![];
    for (method_class, i) in filtered {
        let view = method_class.view();
        let method_view = view.method_view_i(i);
        if method_view.name().as_str() == MethodName::constructor_init() {
            let constructor_obj = Constructor::constructor_object_from_method_view(jvm, int_state, &method_view)?;
            res.push(MemberName::new_from_constructor(jvm, int_state, constructor_obj)?)
        } else {
            let method_obj = Method::method_object_from_method_view(jvm, int_state, &method_view)?;
            res.push(MemberName::new_from_method(jvm, int_state, method_obj)?)
        }
    }
    Ok(res)
}

pub fn Java_java_lang_invoke_MethodHandleNatives_objectFieldOffset(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, args: Vec<JavaValue<'gc_life>>) -> Result<JavaValue<'gc_life>, WasException> {
    let member_name = args[0].cast_member_name();
    let name = member_name.get_name_func(jvm, int_state)?.expect("null name?");
    let clazz = unwrap_or_npe(jvm, int_state, member_name.clazz(jvm))?;
    let field_type_option = member_name.get_field_type(jvm, int_state)?;
    let field_type = unwrap_or_npe(jvm, int_state, field_type_option)?;
    let empty_string = JString::from_rust(jvm, int_state, "".to_string())?;
    let field = Field::init(jvm, int_state, clazz, name, field_type, 0, 0, empty_string, vec![])?;
    Ok(Unsafe::the_unsafe(jvm, int_state).object_field_offset(jvm, int_state, field)?)
}
