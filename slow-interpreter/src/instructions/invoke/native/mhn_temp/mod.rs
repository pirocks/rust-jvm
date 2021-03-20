#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]


use crate::{InterpreterStateGuard, JVMState};
use crate::interpreter::WasException;
use crate::java::lang::reflect::field::Field;
use crate::java::lang::string::JString;
use crate::java_values::JavaValue;
use crate::sun::misc::unsafe_::Unsafe;

pub mod resolve;

pub fn MHN_getConstant() -> Result<JavaValue, WasException> {
    //so I have no idea what this is for, but openjdk does approx this so it should be fine.
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


pub fn Java_java_lang_invoke_MethodHandleNatives_getMembers(args: &mut Vec<JavaValue>) -> Result<JavaValue, WasException> {
    //class member is defined on
    let defc = args[0].cast_class();
    //name to lookup on
    let match_name = args[1].cast_string();
    //signature to lookup on
    let matchSig = args[2].cast_string();
    //flags as defined above
    let matchFlags = args[3].unwrap_int();
    //caller class for access checks
    let _caller = args[4].cast_class();//todo access check
    //seems to be where to start putting in array
    let skip = args[5].cast_class();
    //results arr
    let results = args[6].unwrap_array();

    //todo this will be a mega chonker of a function to implement

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
