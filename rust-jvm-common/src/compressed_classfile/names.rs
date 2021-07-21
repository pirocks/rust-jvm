use std::fmt::{Debug, Formatter};

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use add_only_static_vec::{AddOnlyId, AddOnlyIdMap, AddOnlyVecIDType};

use crate::classnames::ClassName;
use crate::compressed_classfile::{CompressedClassfileString, CompressedParsedRefType};
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
        Self {
            0: CompressedClassfileString { id: AddOnlyId(raw_id) }
        }
    }


    pub fn object() -> Self {
        Self::from_raw_id(JAVA_LANG_OBJECT as AddOnlyVecIDType)
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
        Self::from_raw_id(JAVA_LANG_REFLECT_CONSTANT_POOL as AddOnlyVecIDType)
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
}


impl From<CompressedClassName> for CompressedParsedRefType {
    fn from(ccn: CompressedClassName) -> Self {
        Self::Class(ccn)
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, EnumIter)]
#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq)]
#[allow(non_snake_case)]
enum PredefinedStrings {
    JAVA_LANG_OBJECT,
    JAVA_LANG_CLASS,
    JAVA_LANG_STRING,
    JAVA_LANG_THROWABLE,
    JAVA_LANG_FLOAT,
    JAVA_LANG_DOUBLE,
    JAVA_LANG_INTEGER,
    JAVA_LANG_LONG,
    JAVA_LANG_CHARACTER,
    JAVA_LANG_BOOLEAN,
    JAVA_LANG_BYTE,
    JAVA_LANG_SHORT,
    JAVA_LANG_VOID,
    JAVA_LANG_INVOKE_METHOD_TYPE,
    JAVA_LANG_INVOKE_METHOD_TYPE_FORM,
    JAVA_LANG_INVOKE_METHOD_HANDLE,
    JAVA_LANG_INVOKE_METHOD_HANDLES,
    JAVA_LANG_INVOKE_METHOD_HANDLES_LOOKUP,
    JAVA_LANG_INVOKE_DIRECT_METHOD_HANDLE,
    JAVA_LANG_INVOKE_MEMBER_NAME,
    JAVA_LANG_REFLECT_METHOD,
    JAVA_LANG_SYSTEM,
    JAVA_IO_SERIALIZABLE,
    JAVA_LANG_CLONEABLE,
    SUN_MISC_UNSAFE,
    JAVA_LANG_REFLECT_FIELD,
    JAVA_UTIL_PROPERTIES,
    JAVA_LANG_THREAD,
    JAVA_LANG_THREADGROUP,
    JAVA_LANG_REFLECT_CONSTRUCTOR,
    JAVA_LANG_CLASSLOADER,
    JAVA_LANG_STACK_TRACE_ELEMENT,
    JAVA_LANG_ARRAY_OUT_OF_BOUNDS_EXCEPTION,
    JAVA_LANG_NULL_POINTER_EXCEPTION,
    JAVA_LANG_ILLEGAL_ARGUMENT_EXCEPTION,
    JAVA_LANG_CLASS_NOT_FOUND_EXCEPTION,
    JAVA_LANG_REFLECT_CONSTANT_POOL,
    JAVA_SECURITY_ACCESS_CONTROL_CONTEXT,
    JAVA_SECURITY_PROTECTION_DOMAIN,
    SUN_MISC_LAUNCHER,
    SUN_REFLECT_REFLECTION,
    JAVA_LANG_INVOKE_CALL_SITE,
    JAVA_LANG_INVOKE_LAMBDA_FORM_NAMED_FUNCTION,
    JAVA_NIO_HEAP_BYTE_BUFFER,
    SUN_MISC_LAUNCHER_EXT_CLASS_LOADER,
    JAVA_LANG_LINKAGE_ERROR,
    JAVA_LANG_INVOKE_INVOKERS,
    field_annotationData,
    field_annotationType,
    field_argCounts,
    field_argToSlotTable,
    field_arguments,
    field_basicType,
    field_btChar,
    field_btClass,
    field_cachedConstructor,
    field_classLoader,
    field_classRedefinedCount,
    field_classValueMap,
    field_clazz,
    field_constantPoolOop,
    field_daemon,
    field_detailMessage,
    field_enumConstantDirectory,
    field_enumConstants,
    field_erasedType,
    field_and_method_exit,
    field_flags,
    field_form,
    field_function,
    field_genericInfo,
    field_and_method_getContextClassLoader,
    field_index,
    field_invokers,
    field_and_method_isAlive,
    field_maxPriority,
    field_methodDescriptor,
    field_methodHandles,
    field_and_method_methodType,
    field_modifiers,
    field_name,
    field_names,
    field_newInstanceCallerCache,
    field_ordinal,
    field_and_method_parameterType,
    field_parameterTypes,
    field_parent,
    field_primCounts,
    field_priority,
    field_props,
    field_ptypes,
    field_reflectionData,
    field_resolution,
    field_returnType,
    field_rtype,
    field_and_method_run,
    field_signature,
    field_slot,
    field_theUnsafe,
    field_threadStatus,
    field_threads,
    field_tid,
    field_and_method_type,
    field_and_method_value,
    field_vmentry,
    field_and_method_inheritedAccessControlContext,
    field_IMPL_LOOKUP,
    field_member,
    field_slotToArgTable,
    constructor_init,
    constructor_clinit,
    method_clone,
    method_findSpecial,
    method_findStatic,
    method_findVirtual,
    method_fromMethodDescriptorString,
    method_getClass,
    method_getClassLoader,
    method_getExtClassLoader,
    method_getFieldType,
    method_getLauncher,
    method_getMethodType,
    method_getName,
    method_getTarget,
    method_hashCode,
    method_intern,
    method_internalMemberName,
    method_invoke,
    method_invokeBasic,
    method_invokeExact,
    method_isSameClassPackage,
    method_isStatic,
    method_length,
    method_linkToStatic,
    method_linkToVirtual,
    method_loadClass,
    method_lookup,
    method_objectFieldOffset,
    method_printStackTrace,
    method_publicLookup,
    method_setProperty,
    method_start,
    method_toString,
    method_initializeSystemClass,
}

impl PredefinedStrings {
    pub fn underlying_string(&self) -> String {
        match self {
            JAVA_LANG_OBJECT => ClassName::object().get_referred_name().to_string(),
            JAVA_LANG_CLASS => ClassName::class().get_referred_name().to_string(),
            JAVA_LANG_STRING => ClassName::string().get_referred_name().to_string(),
            JAVA_LANG_THROWABLE => ClassName::throwable().get_referred_name().to_string(),
            JAVA_LANG_FLOAT => ClassName::float().get_referred_name().to_string(),
            JAVA_LANG_DOUBLE => ClassName::double().get_referred_name().to_string(),
            JAVA_LANG_INTEGER => ClassName::int().get_referred_name().to_string(),
            JAVA_LANG_LONG => ClassName::long().get_referred_name().to_string(),
            JAVA_LANG_CHARACTER => ClassName::character().get_referred_name().to_string(),
            JAVA_LANG_BOOLEAN => ClassName::boolean().get_referred_name().to_string(),
            JAVA_LANG_BYTE => ClassName::byte().get_referred_name().to_string(),
            JAVA_LANG_SHORT => ClassName::short().get_referred_name().to_string(),
            JAVA_LANG_VOID => ClassName::void().get_referred_name().to_string(),
            JAVA_LANG_INVOKE_METHOD_TYPE => ClassName::method_type().get_referred_name().to_string(),
            JAVA_LANG_INVOKE_METHOD_TYPE_FORM => ClassName::method_type_form().get_referred_name().to_string(),
            JAVA_LANG_INVOKE_METHOD_HANDLE => ClassName::method_handle().get_referred_name().to_string(),
            JAVA_LANG_INVOKE_METHOD_HANDLES => ClassName::method_handles().get_referred_name().to_string(),
            JAVA_LANG_INVOKE_METHOD_HANDLES_LOOKUP => ClassName::lookup().get_referred_name().to_string(),
            JAVA_LANG_INVOKE_DIRECT_METHOD_HANDLE => ClassName::direct_method_handle().get_referred_name().to_string(),
            JAVA_LANG_INVOKE_MEMBER_NAME => ClassName::member_name().get_referred_name().to_string(),
            JAVA_LANG_REFLECT_METHOD => ClassName::method().get_referred_name().to_string(),
            JAVA_LANG_SYSTEM => ClassName::system().get_referred_name().to_string(),
            JAVA_IO_SERIALIZABLE => ClassName::serializable().get_referred_name().to_string(),
            JAVA_LANG_CLONEABLE => ClassName::cloneable().get_referred_name().to_string(),
            SUN_MISC_UNSAFE => ClassName::unsafe_().get_referred_name().to_string(),
            JAVA_LANG_REFLECT_FIELD => ClassName::field().get_referred_name().to_string(),
            JAVA_UTIL_PROPERTIES => ClassName::properties().get_referred_name().to_string(),
            JAVA_LANG_THREAD => ClassName::thread().get_referred_name().to_string(),
            JAVA_LANG_THREADGROUP => ClassName::thread_group().get_referred_name().to_string(),
            JAVA_LANG_REFLECT_CONSTRUCTOR => ClassName::constructor().get_referred_name().to_string(),
            JAVA_LANG_CLASSLOADER => ClassName::classloader().get_referred_name().to_string(),
            JAVA_LANG_STACK_TRACE_ELEMENT => ClassName::stack_trace_element().get_referred_name().to_string(),
            JAVA_LANG_ARRAY_OUT_OF_BOUNDS_EXCEPTION => "java/lang/ArrayOutOfBoundsException".to_string(),
            JAVA_LANG_NULL_POINTER_EXCEPTION => "java/lang/NullPointerException".to_string(),
            JAVA_LANG_ILLEGAL_ARGUMENT_EXCEPTION => "java/lang/IllegalArgumentException".to_string(),
            JAVA_LANG_CLASS_NOT_FOUND_EXCEPTION => "java/lang/ClassNotFoundException".to_string(),
            JAVA_LANG_REFLECT_CONSTANT_POOL => "java/lang/reflect/ConstantPool".to_string(),
            JAVA_SECURITY_ACCESS_CONTROL_CONTEXT => "java/security/AccessControlContext".to_string(),
            JAVA_SECURITY_PROTECTION_DOMAIN => "java/security/ProtectionDomain".to_string(),
            SUN_MISC_LAUNCHER => "sun/misc/Launcher".to_string(),
            SUN_REFLECT_REFLECTION => "sun/reflect/Reflection".to_string(),
            JAVA_LANG_INVOKE_CALL_SITE => "java/lang/invoke/CallSite".to_string(),
            JAVA_LANG_INVOKE_LAMBDA_FORM_NAMED_FUNCTION => "java/lang/invoke/LambdaForm$NamedFunction".to_string(),
            JAVA_NIO_HEAP_BYTE_BUFFER => "java/nio/HeapByteBuffer".to_string(),
            SUN_MISC_LAUNCHER_EXT_CLASS_LOADER => "sun/misc/Launcher$ExtClassLoader".to_string(),
            JAVA_LANG_LINKAGE_ERROR => "java/lang/LinkageError".to_string(),
            JAVA_LANG_INVOKE_INVOKERS => "java/lang/invoke/Invokers".to_string(),
            field_annotationData => "annotationData".to_string(),
            field_annotationType => "annotationType".to_string(),
            field_argCounts => "argCounts".to_string(),
            field_argToSlotTable => "argToSlotTable".to_string(),
            field_arguments => "arguments".to_string(),
            field_basicType => "basicType".to_string(),
            field_btChar => "btChar".to_string(),
            field_btClass => "btClass".to_string(),
            field_cachedConstructor => "cachedConstructor".to_string(),
            field_classLoader => "classLoader".to_string(),
            field_classRedefinedCount => "classRedefinedCount".to_string(),
            field_classValueMap => "classValueMap".to_string(),
            field_clazz => "clazz".to_string(),
            field_constantPoolOop => "constantPoolOop".to_string(),
            field_daemon => "daemon".to_string(),
            field_detailMessage => "detailMessage".to_string(),
            field_enumConstantDirectory => "enumConstantDirectory".to_string(),
            field_enumConstants => "enumConstants".to_string(),
            field_erasedType => "erasedType".to_string(),
            field_and_method_exit => "exit".to_string(),
            field_flags => "flags".to_string(),
            field_form => "form".to_string(),
            field_function => "function".to_string(),
            field_genericInfo => "genericInfo".to_string(),
            field_and_method_getContextClassLoader => "getContextClassLoader".to_string(),
            field_index => "index".to_string(),
            field_invokers => "invokers".to_string(),
            field_and_method_isAlive => "isAlive".to_string(),
            field_maxPriority => "maxPriority".to_string(),
            field_methodDescriptor => "methodDescriptor".to_string(),
            field_methodHandles => "methodHandles".to_string(),
            field_and_method_methodType => "methodType".to_string(),
            field_modifiers => "modifiers".to_string(),
            field_name => "name".to_string(),
            field_names => "names".to_string(),
            field_newInstanceCallerCache => "newInstanceCallerCache".to_string(),
            field_ordinal => "ordinal".to_string(),
            field_and_method_parameterType => "parameterType".to_string(),
            field_parameterTypes => "parameterTypes".to_string(),
            field_parent => "parent".to_string(),
            field_primCounts => "primCounts".to_string(),
            field_priority => "priority".to_string(),
            field_props => "props".to_string(),
            field_ptypes => "ptypes".to_string(),
            field_reflectionData => "reflectionData".to_string(),
            field_resolution => "resolution".to_string(),
            field_returnType => "returnType".to_string(),
            field_rtype => "rtype".to_string(),
            field_and_method_run => "run".to_string(),
            field_signature => "signature".to_string(),
            field_slot => "slot".to_string(),
            field_theUnsafe => "theUnsafe".to_string(),
            field_threadStatus => "threadStatus".to_string(),
            field_threads => "threads".to_string(),
            field_tid => "tid".to_string(),
            field_and_method_type => "type".to_string(),
            field_and_method_value => "value".to_string(),
            field_vmentry => "vmentry".to_string(),
            field_and_method_inheritedAccessControlContext => "inheritedAccessControlContext".to_string(),
            field_IMPL_LOOKUP => "IMPL_LOOKUP".to_string(),
            field_member => "member".to_string(),
            field_slotToArgTable => "slotToArgTable".to_string(),
            method_clone => "clone".to_string(),
            method_findSpecial => "findSpecial".to_string(),
            method_findStatic => "findStatic".to_string(),
            method_findVirtual => "findVirtual".to_string(),
            method_fromMethodDescriptorString => "fromMethodDescriptorString".to_string(),
            method_getClass => "getClass".to_string(),
            method_getClassLoader => "getClassLoader".to_string(),
            method_getExtClassLoader => "getExtClassLoader".to_string(),
            method_getFieldType => "getFieldType".to_string(),
            method_getLauncher => "getLauncher".to_string(),
            method_getMethodType => "getMethodType".to_string(),
            method_getName => "getName".to_string(),
            method_getTarget => "getTarget".to_string(),
            method_hashCode => "hashCode".to_string(),
            method_intern => "intern".to_string(),
            method_internalMemberName => "internalMemberName".to_string(),
            method_invoke => "invoke".to_string(),
            method_invokeBasic => "invokeBasic".to_string(),
            method_invokeExact => "invokeExact".to_string(),
            method_isSameClassPackage => "isSameClassPackage".to_string(),
            method_isStatic => "isStatic".to_string(),
            method_length => "length".to_string(),
            method_linkToStatic => "linkToStatic".to_string(),
            method_loadClass => "loadClass".to_string(),
            method_lookup => "lookup".to_string(),
            method_objectFieldOffset => "objectFieldOffset".to_string(),
            method_printStackTrace => "printStackTrace".to_string(),
            method_publicLookup => "publicLookup".to_string(),
            method_setProperty => "setProperty".to_string(),
            method_start => "start".to_string(),
            method_toString => "toString".to_string(),
            method_initializeSystemClass => "initializeSystemClass".to_string(),
            constructor_init => "<init>".to_string(),
            constructor_clinit => "<clinit>".to_string(),
            method_linkToVirtual => "linkToVirtual".to_string()
        }
    }
}


fn add_builtin_name(pool: &AddOnlyIdMap<String>, str_: String, id: AddOnlyVecIDType) {
    let res = pool.push(str_);
    assert_eq!(res, AddOnlyId(id));
}


pub fn add_all_names(pool: &AddOnlyIdMap<String>) {
    for pre_defined in PredefinedStrings::iter() {
        add_builtin_name(pool, pre_defined.underlying_string(), pre_defined as AddOnlyVecIDType)
    }
}


#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct FieldName(pub CompressedClassfileString);

#[allow(non_snake_case)]
impl FieldName {
    fn from_raw_id(id: PredefinedStrings) -> Self {
        FieldName(CompressedClassfileString { id: AddOnlyId(id as AddOnlyVecIDType) })
    }

    pub fn field_annotationData() -> Self {
        Self::from_raw_id(field_annotationData)
    }
    pub fn field_annotationType() -> Self {
        Self::from_raw_id(field_annotationType)
    }
    pub fn field_argCounts() -> Self {
        Self::from_raw_id(field_argCounts)
    }
    pub fn field_argToSlotTable() -> Self {
        Self::from_raw_id(field_argToSlotTable)
    }
    pub fn field_arguments() -> Self {
        Self::from_raw_id(field_arguments)
    }
    pub fn field_basicType() -> Self {
        Self::from_raw_id(field_basicType)
    }
    pub fn field_btChar() -> Self {
        Self::from_raw_id(field_btChar)
    }
    pub fn field_btClass() -> Self {
        Self::from_raw_id(field_btClass)
    }
    pub fn field_cachedConstructor() -> Self {
        Self::from_raw_id(field_cachedConstructor)
    }
    pub fn field_classLoader() -> Self {
        Self::from_raw_id(field_classLoader)
    }
    pub fn field_classRedefinedCount() -> Self {
        Self::from_raw_id(field_classRedefinedCount)
    }
    pub fn field_classValueMap() -> Self {
        Self::from_raw_id(field_classValueMap)
    }
    pub fn field_clazz() -> Self {
        Self::from_raw_id(field_clazz)
    }
    pub fn field_constantPoolOop() -> Self {
        Self::from_raw_id(field_constantPoolOop)
    }
    pub fn field_daemon() -> Self {
        Self::from_raw_id(field_daemon)
    }
    pub fn field_detailMessage() -> Self {
        Self::from_raw_id(field_detailMessage)
    }
    pub fn field_enumConstantDirectory() -> Self {
        Self::from_raw_id(field_enumConstantDirectory)
    }
    pub fn field_enumConstants() -> Self {
        Self::from_raw_id(field_enumConstants)
    }
    pub fn field_erasedType() -> Self {
        Self::from_raw_id(field_erasedType)
    }
    pub fn field_exit() -> Self {
        Self::from_raw_id(field_and_method_exit)
    }
    pub fn field_flags() -> Self {
        Self::from_raw_id(field_flags)
    }
    pub fn field_form() -> Self {
        Self::from_raw_id(field_form)
    }
    pub fn field_function() -> Self {
        Self::from_raw_id(field_function)
    }
    pub fn field_genericInfo() -> Self {
        Self::from_raw_id(field_genericInfo)
    }
    pub fn field_getContextClassLoader() -> Self {
        Self::from_raw_id(field_and_method_getContextClassLoader)
    }
    pub fn field_index() -> Self {
        Self::from_raw_id(field_index)
    }
    pub fn field_invokers() -> Self {
        Self::from_raw_id(field_invokers)
    }
    pub fn field_isAlive() -> Self {
        Self::from_raw_id(field_and_method_isAlive)
    }
    pub fn field_maxPriority() -> Self {
        Self::from_raw_id(field_maxPriority)
    }
    pub fn field_methodDescriptor() -> Self {
        Self::from_raw_id(field_methodDescriptor)
    }
    pub fn field_methodHandles() -> Self {
        Self::from_raw_id(field_methodHandles)
    }
    pub fn field_methodType() -> Self {
        Self::from_raw_id(field_and_method_methodType)
    }
    pub fn field_modifiers() -> Self {
        Self::from_raw_id(field_modifiers)
    }
    pub fn field_name() -> Self {
        Self::from_raw_id(field_name)
    }
    pub fn field_names() -> Self {
        Self::from_raw_id(field_names)
    }
    pub fn field_newInstanceCallerCache() -> Self {
        Self::from_raw_id(field_newInstanceCallerCache)
    }
    pub fn field_ordinal() -> Self {
        Self::from_raw_id(field_ordinal)
    }
    pub fn field_parameterType() -> Self {
        Self::from_raw_id(field_and_method_parameterType)
    }
    pub fn field_parameterTypes() -> Self {
        Self::from_raw_id(field_parameterTypes)
    }
    pub fn field_parent() -> Self {
        Self::from_raw_id(field_parent)
    }
    pub fn field_primCounts() -> Self {
        Self::from_raw_id(field_primCounts)
    }
    pub fn field_priority() -> Self {
        Self::from_raw_id(field_priority)
    }
    pub fn field_props() -> Self {
        Self::from_raw_id(field_props)
    }
    pub fn field_ptypes() -> Self {
        Self::from_raw_id(field_ptypes)
    }
    pub fn field_reflectionData() -> Self {
        Self::from_raw_id(field_reflectionData)
    }
    pub fn field_resolution() -> Self {
        Self::from_raw_id(field_resolution)
    }
    pub fn field_returnType() -> Self {
        Self::from_raw_id(field_returnType)
    }
    pub fn field_rtype() -> Self {
        Self::from_raw_id(field_rtype)
    }
    pub fn field_run() -> Self {
        Self::from_raw_id(field_and_method_run)
    }
    pub fn field_signature() -> Self {
        Self::from_raw_id(field_signature)
    }
    pub fn field_slot() -> Self {
        Self::from_raw_id(field_slot)
    }
    pub fn field_theUnsafe() -> Self {
        Self::from_raw_id(field_theUnsafe)
    }
    pub fn field_threadStatus() -> Self {
        Self::from_raw_id(field_threadStatus)
    }
    pub fn field_threads() -> Self {
        Self::from_raw_id(field_threads)
    }
    pub fn field_tid() -> Self {
        Self::from_raw_id(field_tid)
    }
    pub fn field_type() -> Self {
        Self::from_raw_id(field_and_method_type)
    }
    pub fn field_value() -> Self {
        Self::from_raw_id(field_and_method_value)
    }
    pub fn field_vmentry() -> Self {
        Self::from_raw_id(field_vmentry)
    }
    pub fn field_inheritedAccessControlContext() -> Self {
        Self::from_raw_id(field_and_method_inheritedAccessControlContext)
    }
    pub fn field_IMPL_LOOKUP() -> Self {
        Self::from_raw_id(field_IMPL_LOOKUP)
    }
    pub fn field_member() -> Self {
        Self::from_raw_id(field_member)
    }
    pub fn field_slotToArgTable() -> Self {
        Self::from_raw_id(field_slotToArgTable)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct MethodName(pub CompressedClassfileString);

impl Debug for MethodName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.id.0)
    }
}

#[allow(non_snake_case)]
impl MethodName {
    fn from_raw_id(id: PredefinedStrings) -> Self {
        MethodName(CompressedClassfileString { id: AddOnlyId(id as AddOnlyVecIDType) })
    }

    pub fn constructor_init() -> Self {
        Self::from_raw_id(constructor_init)
    }

    pub fn constructor_clinit() -> Self {
        Self::from_raw_id(constructor_clinit)
    }

    pub fn method_clone() -> Self {
        Self::from_raw_id(method_clone)
    }
    pub fn method_findSpecial() -> Self {
        Self::from_raw_id(method_findSpecial)
    }
    pub fn method_findStatic() -> Self {
        Self::from_raw_id(method_findStatic)
    }
    pub fn method_findVirtual() -> Self {
        Self::from_raw_id(method_findVirtual)
    }
    pub fn method_fromMethodDescriptorString() -> Self {
        Self::from_raw_id(method_fromMethodDescriptorString)
    }
    pub fn method_getClass() -> Self {
        Self::from_raw_id(method_getClass)
    }
    pub fn method_getClassLoader() -> Self {
        Self::from_raw_id(method_getClassLoader)
    }
    pub fn method_getExtClassLoader() -> Self {
        Self::from_raw_id(method_getExtClassLoader)
    }
    pub fn method_getFieldType() -> Self {
        Self::from_raw_id(method_getFieldType)
    }
    pub fn method_getLauncher() -> Self {
        Self::from_raw_id(method_getLauncher)
    }
    pub fn method_getMethodType() -> Self {
        Self::from_raw_id(method_getMethodType)
    }
    pub fn method_getName() -> Self {
        Self::from_raw_id(method_getName)
    }
    pub fn method_getTarget() -> Self {
        Self::from_raw_id(method_getTarget)
    }
    pub fn method_hashCode() -> Self {
        Self::from_raw_id(method_hashCode)
    }
    pub fn method_inheritedAccessControlContext() -> Self {
        Self::from_raw_id(field_and_method_inheritedAccessControlContext)
    }
    pub fn method_intern() -> Self {
        Self::from_raw_id(method_intern)
    }
    pub fn method_internalMemberName() -> Self {
        Self::from_raw_id(method_internalMemberName)
    }
    pub fn method_invoke() -> Self {
        Self::from_raw_id(method_invoke)
    }
    pub fn method_invokeBasic() -> Self {
        Self::from_raw_id(method_invokeBasic)
    }
    pub fn method_invokeExact() -> Self {
        Self::from_raw_id(method_invokeExact)
    }
    pub fn method_isSameClassPackage() -> Self {
        Self::from_raw_id(method_isSameClassPackage)
    }
    pub fn method_isStatic() -> Self {
        Self::from_raw_id(method_isStatic)
    }
    pub fn method_length() -> Self {
        Self::from_raw_id(method_length)
    }
    pub fn method_linkToStatic() -> Self {
        Self::from_raw_id(method_linkToStatic)
    }
    pub fn method_linkToVirtual() -> Self {
        Self::from_raw_id(method_linkToVirtual)
    }
    pub fn method_loadClass() -> Self {
        Self::from_raw_id(method_loadClass)
    }
    pub fn method_lookup() -> Self {
        Self::from_raw_id(method_lookup)
    }
    pub fn method_objectFieldOffset() -> Self {
        Self::from_raw_id(method_objectFieldOffset)
    }
    pub fn method_printStackTrace() -> Self {
        Self::from_raw_id(method_printStackTrace)
    }
    pub fn method_publicLookup() -> Self {
        Self::from_raw_id(method_publicLookup)
    }
    pub fn method_run() -> Self {
        Self::from_raw_id(field_and_method_run)
    }
    pub fn method_setProperty() -> Self {
        Self::from_raw_id(method_setProperty)
    }
    pub fn method_start() -> Self {
        Self::from_raw_id(method_start)
    }
    pub fn method_toString() -> Self {
        Self::from_raw_id(method_toString)
    }
    pub fn method_type() -> Self {
        Self::from_raw_id(field_and_method_type)
    }
    pub fn method_value() -> Self {
        Self::from_raw_id(field_and_method_value)
    }
    pub fn method_exit() -> Self {
        Self::from_raw_id(field_and_method_exit)
    }
    pub fn method_isAlive() -> Self {
        Self::from_raw_id(field_and_method_isAlive)
    }
    pub fn method_getContextClassLoader() -> Self {
        Self::from_raw_id(field_and_method_getContextClassLoader)
    }
    pub fn method_initializeSystemClass() -> Self {
        Self::from_raw_id(method_initializeSystemClass)
    }
    pub fn method_parameterType() -> Self {
        Self::from_raw_id(field_and_method_parameterType)
    }
    pub fn method_methodType() -> Self {
        Self::from_raw_id(field_and_method_methodType)
    }
}
