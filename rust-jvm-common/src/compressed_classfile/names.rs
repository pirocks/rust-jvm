use add_only_static_vec::{AddOnlyId, AddOnlyIdMap, AddOnlyVecIDType};

use crate::classnames::ClassName;
use crate::compressed_classfile::{CompressedClassfileString, CompressedParsedRefType};

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub struct CompressedClassName(pub CompressedClassfileString);

pub type CClassName = CompressedClassName;

impl CompressedClassName {
    const fn from_raw_id(raw_id: AddOnlyVecIDType) -> Self {
        Self {
            0: CompressedClassfileString { id: AddOnlyId(raw_id) }
        }
    }


    pub fn object() -> Self {
        Self::from_raw_id(JAVA_LANG_OBJECT)
    }

    pub const fn class() -> Self {
        Self::from_raw_id(JAVA_LANG_CLASS)
    }

    pub const fn string() -> Self {
        Self::from_raw_id(JAVA_LANG_STRING)
    }

    pub const fn throwable() -> Self {
        Self::from_raw_id(JAVA_LANG_THROWABLE)
    }

    pub const fn float() -> Self {
        Self::from_raw_id(JAVA_LANG_FLOAT)
    }

    pub const fn double() -> Self {
        Self::from_raw_id(JAVA_LANG_DOUBLE)
    }
    pub const fn int() -> Self {
        Self::from_raw_id(JAVA_LANG_INTEGER)
    }
    pub const fn long() -> Self {
        Self::from_raw_id(JAVA_LANG_LONG)
    }

    pub const fn character() -> Self {
        Self::from_raw_id(JAVA_LANG_CHARACTER)
    }

    pub const fn boolean() -> Self {
        Self::from_raw_id(JAVA_LANG_BOOLEAN)
    }

    pub const fn byte() -> Self {
        Self::from_raw_id(JAVA_LANG_BYTE)
    }

    pub const fn short() -> Self {
        Self::from_raw_id(JAVA_LANG_SHORT)
    }

    pub const fn void() -> Self {
        Self::from_raw_id(JAVA_LANG_VOID)
    }

    pub const fn method_type() -> Self {
        Self::from_raw_id(JAVA_LANG_INVOKE_METHODTYPE)
    }

    pub const fn method_type_form() -> Self {
        Self::from_raw_id(JAVA_LANG_INVOKE_METHODTYPEFORM)
    }

    pub const fn method_handle() -> Self {
        Self::from_raw_id(JAVA_LANG_INVOKE_METHODHANDLE)
    }

    pub const fn method_handles() -> Self {
        Self::from_raw_id(JAVA_LANG_INVOKE_METHODHANDLES)
    }

    pub const fn lookup() -> Self {
        Self::from_raw_id(JAVA_LANG_INVOKE_METHODHANDLES_LOOKUP)
    }

    pub const fn direct_method_handle() -> Self {
        Self::from_raw_id(JAVA_LANG_INVOKE_DIRECTMETHODHANDLE)
    }

    pub const fn member_name() -> Self {
        Self::from_raw_id(JAVA_LANG_INVOKE_MEMBERNAME)
    }

    pub const fn method() -> Self {
        Self::from_raw_id(JAVA_LANG_REFLECT_METHOD)
    }

    pub const fn system() -> Self {
        Self::from_raw_id(JAVA_LANG_SYSTEM)
    }

    pub const fn serializable() -> Self {
        Self::from_raw_id(JAVA_IO_SERIALIZABLE)
    }

    pub const fn cloneable() -> Self {
        Self::from_raw_id(JAVA_LANG_CLONEABLE)
    }

    pub const fn unsafe_() -> Self {
        Self::from_raw_id(SUN_MISC_UNSAFE)
    }

    pub const fn field() -> Self {
        Self::from_raw_id(JAVA_LANG_REFLECT_FIELD)
    }

    pub const fn properties() -> Self {
        Self::from_raw_id(JAVA_UTIL_PROPERTIES)
    }

    pub const fn thread() -> Self {
        Self::from_raw_id(JAVA_LANG_THREAD)
    }

    pub const fn thread_group() -> Self {
        Self::from_raw_id(JAVA_LANG_THREADGROUP)
    }

    pub const fn constructor() -> Self {
        Self::from_raw_id(JAVA_LANG_REFLECT_CONSTRUCTOR)
    }

    pub const fn classloader() -> Self {
        Self::from_raw_id(JAVA_LANG_CLASSLOADER)
    }

    pub const fn stack_trace_element() -> Self {
        Self::from_raw_id(JAVA_LANG_STACK_TRACE_ELEMENT)
    }

    pub const fn illegal_argument_exception() -> Self {
        Self::from_raw_id(JAVA_LANG_ILLEGAL_ARGUMENT_EXCEPTION)
    }

    pub const fn null_pointer_exception() -> Self {
        Self::from_raw_id(JAVA_LANG_NULL_POINTER_EXCEPTION)
    }

    pub const fn class_not_found_exception() -> Self {
        Self::from_raw_id(JAVA_LANG_CLASS_NOT_FOUND_EXCEPTION)
    }

    pub const fn array_out_of_bounds_exception() -> Self {
        Self::from_raw_id(JAVA_LANG_ARRAY_OUT_OF_BOUNDS_EXCEPTION)
    }

    pub const fn launcher() -> Self {
        Self::from_raw_id(SUN_MISC_LAUNCHER)
    }

    pub const fn reflection() -> Self {
        Self::from_raw_id(SUN_REFLECT_REFLECTION)
    }

    pub const fn constant_pool() -> Self {
        Self::from_raw_id(JAVA_LANG_REFLECT_CONSTANT_POOL)
    }

    pub const fn call_site() -> Self {
        Self::from_raw_id(JAVA_LANG_INVOKE_CALL_SITE)
    }

    pub const fn lambda_from_named_function() -> Self {
        Self::from_raw_id(JAVA_LANG_INVOKE_LAMBDA_FORM_NAMED_FUNCTION)
    }

    pub const fn heap_byte_buffer() -> Self {
        Self::from_raw_id(JAVA_NIO_HEAP_BYTE_BUFFER)
    }

    pub const fn access_control_context() -> Self {
        Self::from_raw_id(JAVA_SECURITY_ACCESS_CONTROL_CONTEXT)
    }

    pub const fn protection_domain() -> Self {
        Self::from_raw_id(JAVA_SECURITY_PROTECTION_DOMAIN)
    }

    pub const fn ext_class_loader() -> Self {
        Self::from_raw_id(SUN_MISC_LAUNCHER_EXT_CLASS_LOADER)
    }
    pub const fn method_handles_lookup() -> Self {
        Self::from_raw_id(JAVA_LANG_INVOKE_METHOD_HANDLES_LOOKUP)
    }

    pub const fn linkage_error() -> Self {
        Self::from_raw_id(JAVA_LANG_LINKAGE_ERROR)
    }

    pub const fn invokers() -> Self {
        Self::from_raw_id(JAVA_LANG_INVOKE_INVOKERS)
    }
}


impl From<CompressedClassName> for CompressedParsedRefType {
    fn from(ccn: CompressedClassName) -> Self {
        Self::Class(ccn)
    }
}


pub const JAVA_LANG_OBJECT: AddOnlyVecIDType = 0;
pub const JAVA_LANG_CLASS: AddOnlyVecIDType = 1;
pub const JAVA_LANG_STRING: AddOnlyVecIDType = 2;
pub const JAVA_LANG_THROWABLE: AddOnlyVecIDType = 3;
pub const JAVA_LANG_FLOAT: AddOnlyVecIDType = 4;
pub const JAVA_LANG_DOUBLE: AddOnlyVecIDType = 5;
pub const JAVA_LANG_INTEGER: AddOnlyVecIDType = 6;
pub const JAVA_LANG_LONG: AddOnlyVecIDType = 7;
pub const JAVA_LANG_CHARACTER: AddOnlyVecIDType = 8;
pub const JAVA_LANG_BOOLEAN: AddOnlyVecIDType = 9;
pub const JAVA_LANG_BYTE: AddOnlyVecIDType = 10;
pub const JAVA_LANG_SHORT: AddOnlyVecIDType = 11;
pub const JAVA_LANG_VOID: AddOnlyVecIDType = 12;
pub const JAVA_LANG_INVOKE_METHODTYPE: AddOnlyVecIDType = 13;
pub const JAVA_LANG_INVOKE_METHODTYPEFORM: AddOnlyVecIDType = 14;
pub const JAVA_LANG_INVOKE_METHODHANDLE: AddOnlyVecIDType = 15;
pub const JAVA_LANG_INVOKE_METHODHANDLES: AddOnlyVecIDType = 16;
pub const JAVA_LANG_INVOKE_METHODHANDLES_LOOKUP: AddOnlyVecIDType = 17;
pub const JAVA_LANG_INVOKE_DIRECTMETHODHANDLE: AddOnlyVecIDType = 18;
pub const JAVA_LANG_INVOKE_MEMBERNAME: AddOnlyVecIDType = 19;
pub const JAVA_LANG_REFLECT_METHOD: AddOnlyVecIDType = 20;
pub const JAVA_LANG_SYSTEM: AddOnlyVecIDType = 21;
pub const JAVA_IO_SERIALIZABLE: AddOnlyVecIDType = 22;
pub const JAVA_LANG_CLONEABLE: AddOnlyVecIDType = 23;
pub const SUN_MISC_UNSAFE: AddOnlyVecIDType = 24;
pub const JAVA_LANG_REFLECT_FIELD: AddOnlyVecIDType = 25;
pub const JAVA_UTIL_PROPERTIES: AddOnlyVecIDType = 26;
pub const JAVA_LANG_THREAD: AddOnlyVecIDType = 27;
pub const JAVA_LANG_THREADGROUP: AddOnlyVecIDType = 28;
pub const JAVA_LANG_REFLECT_CONSTRUCTOR: AddOnlyVecIDType = 29;
pub const JAVA_LANG_CLASSLOADER: AddOnlyVecIDType = 30;
pub const JAVA_LANG_STACK_TRACE_ELEMENT: AddOnlyVecIDType = 31;
pub const JAVA_LANG_ARRAY_OUT_OF_BOUNDS_EXCEPTION: AddOnlyVecIDType = 32;
pub const JAVA_LANG_NULL_POINTER_EXCEPTION: AddOnlyVecIDType = 33;
pub const JAVA_LANG_ILLEGAL_ARGUMENT_EXCEPTION: AddOnlyVecIDType = 34;
pub const JAVA_LANG_CLASS_NOT_FOUND_EXCEPTION: AddOnlyVecIDType = 35;
pub const JAVA_LANG_REFLECT_CONSTANT_POOL: AddOnlyVecIDType = 36;
pub const JAVA_SECURITY_ACCESS_CONTROL_CONTEXT: AddOnlyVecIDType = 37;
pub const JAVA_SECURITY_PROTECTION_DOMAIN: AddOnlyVecIDType = 38;
pub const SUN_MISC_LAUNCHER: AddOnlyVecIDType = 39;
pub const SUN_REFLECT_REFLECTION: AddOnlyVecIDType = 40;
pub const JAVA_LANG_INVOKE_CALL_SITE: AddOnlyVecIDType = 41;
pub const JAVA_LANG_INVOKE_LAMBDA_FORM_NAMED_FUNCTION: AddOnlyVecIDType = 42;
pub const JAVA_NIO_HEAP_BYTE_BUFFER: AddOnlyVecIDType = 43;
pub const SUN_MISC_LAUNCHER_EXT_CLASS_LOADER: AddOnlyVecIDType = 44;
pub const JAVA_LANG_INVOKE_METHOD_HANDLES_LOOKUP: AddOnlyVecIDType = 45;
pub const JAVA_LANG_LINKAGE_ERROR: AddOnlyVecIDType = 46;
pub const JAVA_LANG_INVOKE_INVOKERS: AddOnlyVecIDType = 47;


fn add_builtin_name(pool: &AddOnlyIdMap<String>, cname: ClassName, id: AddOnlyVecIDType) {
    let res = pool.push(cname.get_referred_name().to_string());
    assert_eq!(res, AddOnlyId(id));
}

pub fn add_all_names(pool: &AddOnlyIdMap<String>) {
    add_builtin_name(pool, ClassName::object(), JAVA_LANG_OBJECT);
    add_builtin_name(pool, ClassName::class(), JAVA_LANG_CLASS);
    add_builtin_name(pool, ClassName::string(), JAVA_LANG_STRING);
    add_builtin_name(pool, ClassName::throwable(), JAVA_LANG_THROWABLE);
    add_builtin_name(pool, ClassName::float(), JAVA_LANG_FLOAT);
    add_builtin_name(pool, ClassName::double(), JAVA_LANG_DOUBLE);
    add_builtin_name(pool, ClassName::int(), JAVA_LANG_INTEGER);
    add_builtin_name(pool, ClassName::long(), JAVA_LANG_LONG);
    add_builtin_name(pool, ClassName::character(), JAVA_LANG_CHARACTER);
    add_builtin_name(pool, ClassName::boolean(), JAVA_LANG_BOOLEAN);
    add_builtin_name(pool, ClassName::byte(), JAVA_LANG_BYTE);
    add_builtin_name(pool, ClassName::short(), JAVA_LANG_SHORT);
    add_builtin_name(pool, ClassName::void(), JAVA_LANG_VOID);
    add_builtin_name(pool, ClassName::method_type(), JAVA_LANG_INVOKE_METHODTYPE);
    add_builtin_name(pool, ClassName::method_type_form(), JAVA_LANG_INVOKE_METHODTYPEFORM);
    add_builtin_name(pool, ClassName::method_handle(), JAVA_LANG_INVOKE_METHODHANDLE);
    add_builtin_name(pool, ClassName::method_handles(), JAVA_LANG_INVOKE_METHODHANDLES);
    add_builtin_name(pool, ClassName::lookup(), JAVA_LANG_INVOKE_METHODHANDLES_LOOKUP);
    add_builtin_name(pool, ClassName::direct_method_handle(), JAVA_LANG_INVOKE_DIRECTMETHODHANDLE);
    add_builtin_name(pool, ClassName::member_name(), JAVA_LANG_INVOKE_MEMBERNAME);
    add_builtin_name(pool, ClassName::method(), JAVA_LANG_REFLECT_METHOD);
    add_builtin_name(pool, ClassName::system(), JAVA_LANG_SYSTEM);
    add_builtin_name(pool, ClassName::serializable(), JAVA_IO_SERIALIZABLE);
    add_builtin_name(pool, ClassName::cloneable(), JAVA_LANG_CLONEABLE);
    add_builtin_name(pool, ClassName::unsafe_(), SUN_MISC_UNSAFE);
    add_builtin_name(pool, ClassName::field(), JAVA_LANG_REFLECT_FIELD);
    add_builtin_name(pool, ClassName::properties(), JAVA_UTIL_PROPERTIES);
    add_builtin_name(pool, ClassName::thread(), JAVA_LANG_THREAD);
    add_builtin_name(pool, ClassName::thread_group(), JAVA_LANG_THREADGROUP);
    add_builtin_name(pool, ClassName::constructor(), JAVA_LANG_REFLECT_CONSTRUCTOR);
    add_builtin_name(pool, ClassName::classloader(), JAVA_LANG_CLASSLOADER);
    add_builtin_name(pool, ClassName::stack_trace_element(), JAVA_LANG_STACK_TRACE_ELEMENT);
    add_builtin_name(pool, ClassName::Str("java/lang/ArrayOutOfBoundsException".to_string()), JAVA_LANG_ARRAY_OUT_OF_BOUNDS_EXCEPTION);
    add_builtin_name(pool, ClassName::Str("java/lang/NullPointerException".to_string()), JAVA_LANG_NULL_POINTER_EXCEPTION);
    add_builtin_name(pool, ClassName::Str("java/lang/IllegalArgumentException".to_string()), JAVA_LANG_ILLEGAL_ARGUMENT_EXCEPTION);
    add_builtin_name(pool, ClassName::Str("java/lang/ClassNotFoundException".to_string()), JAVA_LANG_CLASS_NOT_FOUND_EXCEPTION);
    add_builtin_name(pool, ClassName::Str("java/lang/reflect/ConstantPool".to_string()), JAVA_LANG_REFLECT_CONSTANT_POOL);
    add_builtin_name(pool, ClassName::Str("java/security/AccessControlContext".to_string()), JAVA_SECURITY_ACCESS_CONTROL_CONTEXT);
    add_builtin_name(pool, ClassName::Str("java/security/ProtectionDomain".to_string()), JAVA_SECURITY_PROTECTION_DOMAIN);
    add_builtin_name(pool, ClassName::Str("sun/misc/Launcher".to_string()), SUN_MISC_LAUNCHER);
    add_builtin_name(pool, ClassName::Str("sun/reflect/Reflection".to_string()), SUN_REFLECT_REFLECTION);
    add_builtin_name(pool, ClassName::Str("java/lang/invoke/CallSite".to_string()), JAVA_LANG_INVOKE_CALL_SITE);
    add_builtin_name(pool, ClassName::Str("java/nio/HeapByteBuffer".to_string()), JAVA_NIO_HEAP_BYTE_BUFFER);
    add_builtin_name(pool, ClassName::Str("java/lang/invoke/LambdaForm$NamedFunction".to_string()), JAVA_LANG_INVOKE_LAMBDA_FORM_NAMED_FUNCTION);
    add_builtin_name(pool, ClassName::Str("sun/misc/Launcher$ExtClassLoader".to_string()), SUN_MISC_LAUNCHER_EXT_CLASS_LOADER);
    add_builtin_name(pool, ClassName::Str("java/lang/invoke/MethodHandles$Lookup".to_string()), JAVA_LANG_INVOKE_METHOD_HANDLES_LOOKUP);
    add_builtin_name(pool, ClassName::Str("java/lang/LinkageError".to_string()), JAVA_LANG_LINKAGE_ERROR);
    add_builtin_name(pool, ClassName::Str("java/lang/invoke/Invokers".to_string()), JAVA_LANG_INVOKE_INVOKERS);
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct FieldName(pub CompressedClassfileString);

#[allow(non_snake_case)]
impl FieldName {
    pub fn field_annotationData() -> Self {
        todo!()
    }
    pub fn field_annotationType() -> Self {
        todo!()
    }
    pub fn field_argCounts() -> Self {
        todo!()
    }
    pub fn field_argToSlotTable() -> Self {
        todo!()
    }
    pub fn field_arguments() -> Self {
        todo!()
    }
    pub fn field_basicType() -> Self {
        todo!()
    }
    pub fn field_btChar() -> Self {
        todo!()
    }
    pub fn field_btClass() -> Self {
        todo!()
    }
    pub fn field_cachedConstructor() -> Self {
        todo!()
    }
    pub fn field_classLoader() -> Self {
        todo!()
    }
    pub fn field_classRedefinedCount() -> Self {
        todo!()
    }
    pub fn field_classValueMap() -> Self {
        todo!()
    }
    pub fn field_clazz() -> Self {
        todo!()
    }
    pub fn field_constantPoolOop() -> Self {
        todo!()
    }
    pub fn field_daemon() -> Self {
        todo!()
    }
    pub fn field_detailMessage() -> Self {
        todo!()
    }
    pub fn field_enumConstantDirectory() -> Self {
        todo!()
    }
    pub fn field_enumConstants() -> Self {
        todo!()
    }
    pub fn field_erasedType() -> Self {
        todo!()
    }
    pub fn field_exit() -> Self {
        todo!()
    }
    pub fn field_flags() -> Self {
        todo!()
    }
    pub fn field_form() -> Self {
        todo!()
    }
    pub fn field_function() -> Self {
        todo!()
    }
    pub fn field_genericInfo() -> Self {
        todo!()
    }
    pub fn field_getContextClassLoader() -> Self {
        todo!()
    }
    pub fn field_index() -> Self {
        todo!()
    }
    pub fn field_invokers() -> Self {
        todo!()
    }
    pub fn field_isAlive() -> Self {
        todo!()
    }
    pub fn field_maxPriority() -> Self {
        todo!()
    }
    pub fn field_methodDescriptor() -> Self {
        todo!()
    }
    pub fn field_methodHandles() -> Self {
        todo!()
    }
    pub fn field_methodType() -> Self {
        todo!()
    }
    pub fn field_modifiers() -> Self {
        todo!()
    }
    pub fn field_name() -> Self {
        todo!()
    }
    pub fn field_names() -> Self {
        todo!()
    }
    pub fn field_newInstanceCallerCache() -> Self {
        todo!()
    }
    pub fn field_ordinal() -> Self {
        todo!()
    }
    pub fn field_parameterType() -> Self {
        todo!()
    }
    pub fn field_parameterTypes() -> Self {
        todo!()
    }
    pub fn field_parent() -> Self {
        todo!()
    }
    pub fn field_primCounts() -> Self {
        todo!()
    }
    pub fn field_priority() -> Self {
        todo!()
    }
    pub fn field_props() -> Self {
        todo!()
    }
    pub fn field_ptypes() -> Self {
        todo!()
    }
    pub fn field_reflectionData() -> Self {
        todo!()
    }
    pub fn field_resolution() -> Self {
        todo!()
    }
    pub fn field_returnType() -> Self {
        todo!()
    }
    pub fn field_rtype() -> Self {
        todo!()
    }
    pub fn field_run() -> Self {
        todo!()
    }
    pub fn field_signature() -> Self {
        todo!()
    }
    pub fn field_slot() -> Self {
        todo!()
    }
    pub fn field_theUnsafe() -> Self {
        todo!()
    }
    pub fn field_threadStatus() -> Self {
        todo!()
    }
    pub fn field_threads() -> Self {
        todo!()
    }
    pub fn field_tid() -> Self {
        todo!()
    }
    pub fn field_type() -> Self {
        todo!()
    }
    pub fn field_value() -> Self {
        todo!()
    }
    pub fn field_vmentry() -> Self {
        todo!()
    }
    pub fn field_inheritedAccessControlContext() -> Self {
        todo!()
    }
    pub fn field_IMPL_LOOKUP() -> Self {
        todo!()
    }
    pub fn field_member() -> Self {
        todo!()
    }
    pub fn field_slotToArgTable() -> Self {
        todo!()
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct MethodName(pub CompressedClassfileString);

#[allow(non_snake_case)]
impl MethodName {
    pub fn constructor_init() -> Self {
        todo!()
    }

    pub fn constructor_clinit() -> Self {
        todo!()
    }

    pub fn method_clone() -> Self {
        todo!()
    }
    pub fn method_findSpecial() -> Self {
        todo!()
    }
    pub fn method_findStatic() -> Self {
        todo!()
    }
    pub fn method_findVirtual() -> Self {
        todo!()
    }
    pub fn method_fromMethodDescriptorString() -> Self {
        todo!()
    }
    pub fn method_getClass() -> Self {
        todo!()
    }
    pub fn method_getClassLoader() -> Self {
        todo!()
    }
    pub fn method_getExtClassLoader() -> Self {
        todo!()
    }
    pub fn method_getFieldType() -> Self {
        todo!()
    }
    pub fn method_getLauncher() -> Self {
        todo!()
    }
    pub fn method_getMethodType() -> Self {
        todo!()
    }
    pub fn method_getName() -> Self {
        todo!()
    }
    pub fn method_getTarget() -> Self {
        todo!()
    }
    pub fn method_hashCode() -> Self {
        todo!()
    }
    pub fn method_inheritedAccessControlContext() -> Self {
        todo!()
    }
    pub fn method_intern() -> Self {
        todo!()
    }
    pub fn method_internalMemberName() -> Self {
        todo!()
    }
    pub fn method_invoke() -> Self {
        todo!()
    }
    pub fn method_invokeBasic() -> Self {
        todo!()
    }
    pub fn method_invokeExact() -> Self {
        todo!()
    }
    pub fn method_isSameClassPackage() -> Self {
        todo!()
    }
    pub fn method_isStatic() -> Self {
        todo!()
    }
    pub fn method_length() -> Self {
        todo!()
    }
    pub fn method_linkToStatic() -> Self {
        todo!()
    }
    pub fn method_loadClass() -> Self {
        todo!()
    }
    pub fn method_lookup() -> Self {
        todo!()
    }
    pub fn method_objectFieldOffset() -> Self {
        todo!()
    }
    pub fn method_printStackTrace() -> Self {
        todo!()
    }
    pub fn method_publicLookup() -> Self {
        todo!()
    }
    pub fn method_run() -> Self {
        todo!()
    }
    pub fn method_setProperty() -> Self {
        todo!()
    }
    pub fn method_start() -> Self {
        todo!()
    }
    pub fn method_toString() -> Self {
        todo!()
    }
    pub fn method_type() -> Self {
        todo!()
    }
    pub fn method_value() -> Self {
        todo!()
    }
    pub fn method_exit() -> Self {
        todo!()
    }
    pub fn method_isAlive() -> Self { todo!() }
    pub fn method_getContextClassLoader() -> Self {
        todo!()
    }
    pub fn method_initializeSystemClass() -> Self {
        todo!()
    }
    pub fn method_parameterType() -> Self {
        todo!()
    }
    pub fn method_methodType() -> Self {
        todo!()
    }
}
