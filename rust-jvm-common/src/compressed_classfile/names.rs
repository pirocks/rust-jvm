use std::fmt::{Debug};

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use add_only_static_vec::{AddOnlyId, AddOnlyIdMap, AddOnlyVecIDType};

use crate::classnames::ClassName;
use crate::compressed_classfile::names::PredefinedStrings::*;

#[allow(non_camel_case_types)]
#[derive(Debug, EnumIter)]
#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq)]
#[allow(non_snake_case)]
#[allow(clippy::upper_case_acronyms)]
pub(crate) enum PredefinedStrings {
    INVALID,
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
    JAVA_LANG_REFLECT_GENERIC_DECLARATION,
    JAVA_LANG_REFLECT_TYPE,
    JAVA_LANG_REFLECT_ANNOTATED_ELEMENT,
    JAVA_LANG_CLONEABLE,
    SUN_MISC_UNSAFE,
    JAVA_LANG_REFLECT_FIELD,
    JAVA_UTIL_PROPERTIES,
    JAVA_UTIL_HASHTABLE,
    JAVA_UTIL_HASHTABLE_ENTRY,
    JAVA_UTIL_BIG_INTEGER,
    JAVA_UTIL_MUTABLE_BIG_INTEGER,
    JAVA_UTIL_CONCURRENT_CONCURRENT_HASHMAP,
    JAVA_UTIL_CONCURRENT_CONCURRENT_HASHMAP_NODE,
    JAVA_LANG_DEPRECATED,
    JAVA_LANG_THREAD,
    JAVA_LANG_THREADGROUP,
    JAVA_LANG_REFLECT_CONSTRUCTOR,
    JAVA_LANG_CLASSLOADER,
    JAVA_LANG_STACK_TRACE_ELEMENT,
    JAVA_LANG_ARRAY_OUT_OF_BOUNDS_EXCEPTION,
    JAVA_LANG_NULL_POINTER_EXCEPTION,
    JAVA_LANG_ILLEGAL_ARGUMENT_EXCEPTION,
    JAVA_LANG_CLASS_NOT_FOUND_EXCEPTION,
    SUN_REFLECT_CONSTANT_POOL,
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
    SUN_NIO_CS_FAST_CHARSET_PROVIDER,
    SUN_NIO_CS_STANDARD_CHARSETS,
    SUN_NIO_CS_STANDARD_CHARSETS_CACHE,
    SUN_UTIL_PRE_HASHED_MAP,
    JAVA_LANG_REFLECT_ARRAY,
    JAVA_LANG_COMPARABLE,
    JAVA_LANG_CLASS_CAST_EXCEPTION,
    JAVA_LANG_NO_SUCH_METHOD_ERROR,
    field_annotationData,
    method_and_field_annotationType,
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
    field_ht,
    field_and_method_getContextClassLoader,
    field_key,
    field_int_len,
    field_index,
    field_invokers,
    field_and_method_isAlive,
    field_map,
    field_maxPriority,
    field_methodDescriptor,
    field_methodHandles,
    field_and_method_methodType,
    field_modifiers,
    field_and_method_name,
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
    field_val,
    field_theUnsafe,
    field_threadStatus,
    field_threads,
    field_table,
    field_sizeCtl,
    field_tid,
    field_and_method_type,
    field_and_method_value,
    field_next,
    field_hash,
    field_vmentry,
    field_and_method_inheritedAccessControlContext,
    field_IMPL_LOOKUP,
    field_member,
    field_slotToArgTable,
    field_signum,
    field_mag,
    field_formalTypeParams,
    constructor_init,
    constructor_clinit,
    method_clone,
    method_equals,
    method_findSpecial,
    method_findStatic,
    method_findVirtual,
    method_fromMethodDescriptorString,
    method_getClass,
    method_getComponentType,
    method_newArray,
    method_arraycopy,
    method_compareAndSwapLong,
    method_compareAndSwapInt,
    method_compareAndSwapObject,
    method_identityHashCode,
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
    method_getProperty,
    method_start,
    method_toString,
    method_initializeSystemClass,
    method_getGenericInterfaces,
    method_putIfAbsent,
    method_get,
    method_destructiveMulAdd,
    SUN_REFLECT_GENERICS_TREE_CLASS_SIGNATURE,
    method_getLong,
    method_getByte,
    method_registerNatives,
    method_addressSize,
    method_getIntVolatile,
    method_allocateMemory,
    method_putLong,
    method_freeMemory
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
            JAVA_LANG_ARRAY_OUT_OF_BOUNDS_EXCEPTION => "java/lang/ArrayIndexOutOfBoundsException".to_string(),
            JAVA_LANG_NULL_POINTER_EXCEPTION => "java/lang/NullPointerException".to_string(),
            JAVA_LANG_ILLEGAL_ARGUMENT_EXCEPTION => "java/lang/IllegalArgumentException".to_string(),
            JAVA_LANG_CLASS_NOT_FOUND_EXCEPTION => "java/lang/ClassNotFoundException".to_string(),
            SUN_REFLECT_CONSTANT_POOL => "sun/reflect/ConstantPool".to_string(),
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
            JAVA_UTIL_HASHTABLE => "java/util/Hashtable".to_string(),
            field_annotationData => "annotationData".to_string(),
            method_and_field_annotationType => "annotationType".to_string(),
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
            field_and_method_name => "name".to_string(),
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
            method_getProperty => "getProperty".to_string(),
            method_setProperty => "setProperty".to_string(),
            method_start => "start".to_string(),
            method_toString => "toString".to_string(),
            method_initializeSystemClass => "initializeSystemClass".to_string(),
            constructor_init => "<init>".to_string(),
            constructor_clinit => "<clinit>".to_string(),
            method_linkToVirtual => "linkToVirtual".to_string(),
            field_map => "map".to_string(),
            field_table => "table".to_string(),
            field_key => "key".to_string(),
            field_next => "next".to_string(),
            field_hash => "hash".to_string(),
            SUN_NIO_CS_FAST_CHARSET_PROVIDER => "sun/nio/cs/FastCharsetProvider".to_string(),
            field_ht => "ht".to_string(),
            SUN_NIO_CS_STANDARD_CHARSETS => "sun/nio/cs/StandardCharsets".to_string(),
            SUN_NIO_CS_STANDARD_CHARSETS_CACHE => "sun/nio/cs/StandardCharsets$Cache".to_string(),
            SUN_UTIL_PRE_HASHED_MAP => "sun/util/PreHashedMap".to_string(),
            JAVA_UTIL_CONCURRENT_CONCURRENT_HASHMAP => "java/util/concurrent/ConcurrentHashMap".to_string(),
            field_sizeCtl => "sizeCtl".to_string(),
            method_putIfAbsent => "putIfAbsent".to_string(),
            field_val => "val".to_string(),
            JAVA_LANG_DEPRECATED => "java/lang/Deprecated".to_string(),
            method_equals => "equals".to_string(),
            method_get => "get".to_string(),
            JAVA_UTIL_CONCURRENT_CONCURRENT_HASHMAP_NODE => "java/util/concurrent/ConcurrentHashMap$Node".to_string(),
            JAVA_LANG_REFLECT_GENERIC_DECLARATION => "java/lang/reflect/GenericDeclaration".to_string(),
            JAVA_LANG_REFLECT_TYPE => "java/lang/reflect/Type".to_string(),
            JAVA_LANG_REFLECT_ANNOTATED_ELEMENT => "java/lang/reflect/AnnotatedElement".to_string(),
            JAVA_UTIL_HASHTABLE_ENTRY => "java/util/Hashtable$Entry".to_string(),
            JAVA_UTIL_BIG_INTEGER => "java/math/BigInteger".to_string(),
            JAVA_UTIL_MUTABLE_BIG_INTEGER => "java/math/MutableBigInteger".to_string(),
            field_int_len => "intLen".to_string(),
            field_signum => "signum".to_string(),
            field_mag => "mag".to_string(),
            method_destructiveMulAdd => "destructiveMulAdd".to_string(),
            INVALID => "__rust_jvm_invalid".to_string(),
            method_getGenericInterfaces => "getGenericInterfaces".to_string(),
            SUN_REFLECT_GENERICS_TREE_CLASS_SIGNATURE => "sun/reflect/generics/tree/ClassSignature".to_string(),
            field_formalTypeParams => "formalTypeParams".to_string(),
            method_arraycopy => "arraycopy".to_string(),
            method_compareAndSwapLong => "compareAndSwapLong".to_string(),
            method_compareAndSwapInt => "compareAndSwapInt".to_string(),
            method_compareAndSwapObject => "compareAndSwapObject".to_string(),
            method_identityHashCode => "identityHashCode".to_string(),
            method_getComponentType => "getComponentType".to_string(),
            JAVA_LANG_REFLECT_ARRAY => "java/lang/reflect/Array".to_string(),
            method_newArray => "newArray".to_string(),
            JAVA_LANG_COMPARABLE  => "java/lang/Comparable".to_string(),
            method_getLong => "getLong".to_string(),
            method_registerNatives => "registerNatives".to_string(),
            method_addressSize => "addressSize".to_string(),
            method_getIntVolatile => "getIntVolatile".to_string(),
            method_allocateMemory => "allocateMemory".to_string(),
            method_putLong => "putLong".to_string(),
            method_getByte => "getByte".to_string(),
            method_freeMemory => "freeMemory".to_string(),
            JAVA_LANG_CLASS_CAST_EXCEPTION => "java/lang/ClassCastException".to_string(),
            JAVA_LANG_NO_SUCH_METHOD_ERROR => "java/lang/NoSuchMethodError".to_string()
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

