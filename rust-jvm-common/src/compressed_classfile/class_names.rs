use std::fmt::{Debug, Formatter};
use add_only_static_vec::{AddOnlyId, AddOnlyVecIDType};
use crate::compressed_classfile::{CompressedClassfileString};
use crate::compressed_classfile::compressed_types::CompressedParsedRefType;
use crate::compressed_classfile::names::PredefinedStrings::*;

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct CompressedClassName(pub CompressedClassfileString);

impl Debug for CompressedClassName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.id.0)
    }
}

pub type CClassName = CompressedClassName;

impl CompressedClassName {
    const fn from_raw_id(raw_id: AddOnlyVecIDType) -> Self {
        Self(CompressedClassfileString { id: AddOnlyId(raw_id) })
    }

    pub fn object() -> Self {
        Self::from_raw_id(JAVA_LANG_OBJECT as AddOnlyVecIDType)
    }

    pub fn invalid() -> Self {
        Self::from_raw_id(INVALID as AddOnlyVecIDType)
    }

    pub const fn class() -> Self {
        Self::from_raw_id(JAVA_LANG_CLASS as AddOnlyVecIDType)
    }

    pub const fn string() -> Self {
        Self::from_raw_id(JAVA_LANG_STRING as AddOnlyVecIDType)
    }

    pub const fn throwable() -> Self {
        Self::from_raw_id(JAVA_LANG_THROWABLE as AddOnlyVecIDType)
    }

    pub const fn float() -> Self {
        Self::from_raw_id(JAVA_LANG_FLOAT as AddOnlyVecIDType)
    }

    pub const fn double() -> Self {
        Self::from_raw_id(JAVA_LANG_DOUBLE as AddOnlyVecIDType)
    }

    pub const fn int() -> Self {
        Self::from_raw_id(JAVA_LANG_INTEGER as AddOnlyVecIDType)
    }

    pub const fn long() -> Self {
        Self::from_raw_id(JAVA_LANG_LONG as AddOnlyVecIDType)
    }

    pub const fn character() -> Self {
        Self::from_raw_id(JAVA_LANG_CHARACTER as AddOnlyVecIDType)
    }

    pub const fn boolean() -> Self {
        Self::from_raw_id(JAVA_LANG_BOOLEAN as AddOnlyVecIDType)
    }

    pub const fn byte() -> Self {
        Self::from_raw_id(JAVA_LANG_BYTE as AddOnlyVecIDType)
    }

    pub const fn short() -> Self {
        Self::from_raw_id(JAVA_LANG_SHORT as AddOnlyVecIDType)
    }

    pub const fn void() -> Self {
        Self::from_raw_id(JAVA_LANG_VOID as AddOnlyVecIDType)
    }

    pub const fn method_type() -> Self {
        Self::from_raw_id(JAVA_LANG_INVOKE_METHOD_TYPE as AddOnlyVecIDType)
    }

    pub const fn method_type_form() -> Self {
        Self::from_raw_id(JAVA_LANG_INVOKE_METHOD_TYPE_FORM as AddOnlyVecIDType)
    }

    pub const fn method_handle() -> Self {
        Self::from_raw_id(JAVA_LANG_INVOKE_METHOD_HANDLE as AddOnlyVecIDType)
    }

    pub const fn method_handles() -> Self {
        Self::from_raw_id(JAVA_LANG_INVOKE_METHOD_HANDLES as AddOnlyVecIDType)
    }

    pub const fn lookup() -> Self {
        Self::from_raw_id(JAVA_LANG_INVOKE_METHOD_HANDLES_LOOKUP as AddOnlyVecIDType)
    }

    pub const fn direct_method_handle() -> Self {
        Self::from_raw_id(JAVA_LANG_INVOKE_DIRECT_METHOD_HANDLE as AddOnlyVecIDType)
    }

    pub const fn member_name() -> Self {
        Self::from_raw_id(JAVA_LANG_INVOKE_MEMBER_NAME as AddOnlyVecIDType)
    }

    pub const fn method() -> Self {
        Self::from_raw_id(JAVA_LANG_REFLECT_METHOD as AddOnlyVecIDType)
    }

    pub const fn system() -> Self {
        Self::from_raw_id(JAVA_LANG_SYSTEM as AddOnlyVecIDType)
    }

    pub const fn serializable() -> Self {
        Self::from_raw_id(JAVA_IO_SERIALIZABLE as AddOnlyVecIDType)
    }

    pub const fn generic_declaration() -> Self {
        Self::from_raw_id(JAVA_LANG_REFLECT_GENERIC_DECLARATION as AddOnlyVecIDType)
    }

    pub const fn type_() -> Self {
        Self::from_raw_id(JAVA_LANG_REFLECT_TYPE as AddOnlyVecIDType)
    }

    pub const fn annotated_element() -> Self {
        Self::from_raw_id(JAVA_LANG_REFLECT_ANNOTATED_ELEMENT as AddOnlyVecIDType)
    }

    pub const fn cloneable() -> Self {
        Self::from_raw_id(JAVA_LANG_CLONEABLE as AddOnlyVecIDType)
    }

    pub const fn unsafe_() -> Self {
        Self::from_raw_id(SUN_MISC_UNSAFE as AddOnlyVecIDType)
    }

    pub const fn field() -> Self {
        Self::from_raw_id(JAVA_LANG_REFLECT_FIELD as AddOnlyVecIDType)
    }

    pub const fn properties() -> Self {
        Self::from_raw_id(JAVA_UTIL_PROPERTIES as AddOnlyVecIDType)
    }

    pub const fn hashtable() -> Self {
        Self::from_raw_id(JAVA_UTIL_HASHTABLE as AddOnlyVecIDType)
    }

    pub const fn hashtable_entry() -> Self {
        Self::from_raw_id(JAVA_UTIL_HASHTABLE_ENTRY as AddOnlyVecIDType)
    }

    pub const fn big_integer() -> Self {
        Self::from_raw_id(JAVA_UTIL_BIG_INTEGER as AddOnlyVecIDType)
    }

    pub const fn mutable_big_integer() -> Self {
        Self::from_raw_id(JAVA_UTIL_MUTABLE_BIG_INTEGER as AddOnlyVecIDType)
    }

    pub const fn thread() -> Self {
        Self::from_raw_id(JAVA_LANG_THREAD as AddOnlyVecIDType)
    }

    pub const fn thread_group() -> Self {
        Self::from_raw_id(JAVA_LANG_THREADGROUP as AddOnlyVecIDType)
    }

    pub const fn constructor() -> Self {
        Self::from_raw_id(JAVA_LANG_REFLECT_CONSTRUCTOR as AddOnlyVecIDType)
    }

    pub const fn classloader() -> Self {
        Self::from_raw_id(JAVA_LANG_CLASSLOADER as AddOnlyVecIDType)
    }

    pub const fn stack_trace_element() -> Self {
        Self::from_raw_id(JAVA_LANG_STACK_TRACE_ELEMENT as AddOnlyVecIDType)
    }

    pub const fn illegal_argument_exception() -> Self {
        Self::from_raw_id(JAVA_LANG_ILLEGAL_ARGUMENT_EXCEPTION as AddOnlyVecIDType)
    }

    pub const fn null_pointer_exception() -> Self {
        Self::from_raw_id(JAVA_LANG_NULL_POINTER_EXCEPTION as AddOnlyVecIDType)
    }

    pub const fn class_not_found_exception() -> Self {
        Self::from_raw_id(JAVA_LANG_CLASS_NOT_FOUND_EXCEPTION as AddOnlyVecIDType)
    }

    pub const fn array_out_of_bounds_exception() -> Self {
        Self::from_raw_id(JAVA_LANG_ARRAY_OUT_OF_BOUNDS_EXCEPTION as AddOnlyVecIDType)
    }

    pub const fn launcher() -> Self {
        Self::from_raw_id(SUN_MISC_LAUNCHER as AddOnlyVecIDType)
    }

    pub const fn reflection() -> Self {
        Self::from_raw_id(SUN_REFLECT_REFLECTION as AddOnlyVecIDType)
    }

    pub const fn constant_pool() -> Self {
        Self::from_raw_id(SUN_REFLECT_CONSTANT_POOL as AddOnlyVecIDType)
    }

    pub const fn call_site() -> Self {
        Self::from_raw_id(JAVA_LANG_INVOKE_CALL_SITE as AddOnlyVecIDType)
    }

    pub const fn lambda_from_named_function() -> Self {
        Self::from_raw_id(JAVA_LANG_INVOKE_LAMBDA_FORM_NAMED_FUNCTION as AddOnlyVecIDType)
    }

    pub const fn heap_byte_buffer() -> Self {
        Self::from_raw_id(JAVA_NIO_HEAP_BYTE_BUFFER as AddOnlyVecIDType)
    }

    pub const fn access_control_context() -> Self {
        Self::from_raw_id(JAVA_SECURITY_ACCESS_CONTROL_CONTEXT as AddOnlyVecIDType)
    }

    pub const fn protection_domain() -> Self {
        Self::from_raw_id(JAVA_SECURITY_PROTECTION_DOMAIN as AddOnlyVecIDType)
    }

    pub const fn ext_class_loader() -> Self {
        Self::from_raw_id(SUN_MISC_LAUNCHER_EXT_CLASS_LOADER as AddOnlyVecIDType)
    }
    pub const fn method_handles_lookup() -> Self {
        Self::from_raw_id(JAVA_LANG_INVOKE_METHOD_HANDLES_LOOKUP as AddOnlyVecIDType)
    }

    pub const fn linkage_error() -> Self {
        Self::from_raw_id(JAVA_LANG_LINKAGE_ERROR as AddOnlyVecIDType)
    }

    pub const fn invokers() -> Self {
        Self::from_raw_id(JAVA_LANG_INVOKE_INVOKERS as AddOnlyVecIDType)
    }

    pub const fn fast_charset_provider() -> Self {
        Self::from_raw_id(SUN_NIO_CS_FAST_CHARSET_PROVIDER as AddOnlyVecIDType)
    }

    pub const fn standard_charsets() -> Self {
        Self::from_raw_id(SUN_NIO_CS_STANDARD_CHARSETS as AddOnlyVecIDType)
    }

    pub const fn standard_charsets_cache() -> Self {
        Self::from_raw_id(SUN_NIO_CS_STANDARD_CHARSETS_CACHE as AddOnlyVecIDType)
    }

    pub const fn pre_hashed_map() -> Self {
        Self::from_raw_id(SUN_UTIL_PRE_HASHED_MAP as AddOnlyVecIDType)
    }

    pub const fn concurrent_hash_map() -> Self {
        Self::from_raw_id(JAVA_UTIL_CONCURRENT_CONCURRENT_HASHMAP as AddOnlyVecIDType)
    }

    pub const fn concurrent_hash_map_node() -> Self {
        Self::from_raw_id(JAVA_UTIL_CONCURRENT_CONCURRENT_HASHMAP_NODE as AddOnlyVecIDType)
    }

    pub const fn deprecated() -> Self {
        Self::from_raw_id(JAVA_LANG_DEPRECATED as AddOnlyVecIDType)
    }

    pub const fn class_signature() -> Self {
        Self::from_raw_id(SUN_REFLECT_GENERICS_TREE_CLASS_SIGNATURE as AddOnlyVecIDType)
    }

    pub const fn array() -> Self {
        Self::from_raw_id(JAVA_LANG_REFLECT_ARRAY as AddOnlyVecIDType)
    }

    pub const fn comparable() -> Self {
        Self::from_raw_id(JAVA_LANG_COMPARABLE as AddOnlyVecIDType)
    }
}

impl From<CompressedClassName> for CompressedParsedRefType {
    fn from(ccn: CompressedClassName) -> Self {
        Self::Class(ccn)
    }
}

