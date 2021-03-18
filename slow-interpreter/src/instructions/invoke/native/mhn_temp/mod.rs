#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]


use std::sync::Arc;

use classfile_view::view::HasAccessFlags;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::descriptor_parser::parse_method_descriptor;

use crate::{InterpreterStateGuard, JVMState, StackEntry};
use crate::class_loading::assert_inited_or_initing_class;
use crate::class_objects::get_or_create_class_object;
use crate::instructions::invoke::static_::invoke_static_impl;
use crate::instructions::invoke::virtual_::invoke_virtual_method_i;
use crate::interpreter::WasException;
use crate::interpreter_util::{push_new_object, run_constructor};
use crate::java::lang::reflect::field::Field;
use crate::java::lang::string::JString;
use crate::java_values::{ArrayObject, JavaValue};
use crate::java_values::Object::Array;
use crate::runtime_class::RuntimeClass;
use crate::sun::misc::unsafe_::Unsafe;

pub mod resolve;

pub fn MHN_getConstant() -> Result<JavaValue, WasException> {
//todo
    Ok(JavaValue::Int(0))
}

pub const BRIDGE: i32 = 64;
pub const VARARGS: i32 = 128;
pub const SYNTHETIC: i32 = 4096;
pub const ANNOTATION: i32 = 8192;
pub const ENUM: i32 = 16384;
pub const RECOGNIZED_MODIFIERS: i32 = 65535;
pub const IS_METHOD: i32 = 65536;
pub const IS_CONSTRUCTOR: i32 = 131072;
pub const IS_FIELD: i32 = 262144;
pub const IS_TYPE: i32 = 524288;
pub const CALLER_SENSITIVE: i32 = 1048576;
pub const ALL_ACCESS: i32 = 7;
pub const ALL_KINDS: i32 = 983040;
pub const IS_INVOCABLE: i32 = 196608;
pub const IS_FIELD_OR_METHOD: i32 = 327680;
pub const SEARCH_ALL_SUPERS: i32 = 3145728;
pub const REFERENCE_KIND_SHIFT: u32 = 24;
pub const REFERENCE_KIND_MASK: u32 = 0xF;

pub mod init;

pub fn create_method_type(jvm: &JVMState, int_state: &mut InterpreterStateGuard, frame: &mut StackEntry, signature: &str) -> Result<(), WasException> {
    //todo should this actually be resolving or is that only for MHN_init. Why is this done in native code anyway
    //todo need to use MethodTypeForm.findForm
    let method_type_class = assert_inited_or_initing_class(jvm, int_state, ClassName::method_type().into());
    push_new_object(jvm, int_state, &method_type_class);
    let this = int_state.pop_current_operand_stack();
    let method_descriptor = parse_method_descriptor(signature).unwrap();
    let rtype = JavaValue::Object(get_or_create_class_object(jvm, PTypeView::from_ptype(&method_descriptor.return_type), int_state)?.into());

    let mut ptypes_as_classes: Vec<JavaValue> = vec![];
    for x in method_descriptor.parameter_types.iter() {
        let class_object = get_or_create_class_object(jvm, PTypeView::from_ptype(&x), int_state)?;
        ptypes_as_classes.push(JavaValue::Object(class_object.into()))
    }
    let class_type = PTypeView::Ref(ReferenceTypeView::Class(ClassName::class()));
    let ptypes = JavaValue::Object(Arc::new(Array(ArrayObject::new_array(
        jvm,
        int_state,
        ptypes_as_classes,
        class_type,
        jvm.thread_state.new_monitor("monitor for a method type".to_string()))?)).into());
    run_constructor(jvm, int_state, method_type_class, vec![this.clone(), rtype, ptypes], "([Ljava/lang/Class;Ljava/lang/Class;)V".to_string())?;
    frame.push(this);
    Ok(())
}


//todo this should go in some sort of utils
pub fn run_static_or_virtual(jvm: &JVMState, int_state: &mut InterpreterStateGuard, class: &Arc<RuntimeClass>, method_name: String, desc_str: String) -> Result<(), WasException> {
    let parsed_desc = parse_method_descriptor(desc_str.as_str()).unwrap();
    let view = class.view();
    let res_fun = view.lookup_method(&method_name, &parsed_desc);//todo move this into classview
    let method_view = match res_fun {
        Some(x) => x,
        None => unimplemented!(),
    };//todo and if this fails
    let md = method_view.desc();
    if method_view.is_static() {
        invoke_static_impl(jvm, int_state, md, class.clone(), method_view.method_i(), &method_view)
    } else {
        invoke_virtual_method_i(jvm, int_state, md, class.clone(), &method_view)
    }
}


pub fn Java_java_lang_invoke_MethodHandleNatives_getMembers(_args: &mut Vec<JavaValue>) -> Result<JavaValue, WasException> {
//static native int getMembers(Class<?> defc, String matchName, String matchSig,
// //          int matchFlags, Class<?> caller, int skip, MemberName[] results);
//     dbg!(args);
    //todo nyi
    // unimplemented!();
    Ok(JavaValue::Int(0))
}

pub fn Java_java_lang_invoke_MethodHandleNatives_objectFieldOffset(jvm: &JVMState, int_state: &mut InterpreterStateGuard, args: &mut Vec<JavaValue>) -> Result<JavaValue, WasException> {
    let member_name = args[0].cast_member_name();
    let name = member_name.get_name_func(jvm, int_state)?;
    let clazz = member_name.clazz();
    let field_type = member_name.get_field_type(jvm, int_state)?;
    let empty_string = JString::from_rust(jvm, int_state, "".to_string())?;
    let field = Field::init(jvm, int_state, clazz, name, field_type, 0, 0, empty_string, vec![])?;
    Ok(Unsafe::the_unsafe(jvm, int_state).object_field_offset(jvm, int_state, field)?)
}
