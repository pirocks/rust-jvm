use std::fmt::{Debug, Formatter};
use add_only_static_vec::{AddOnlyId, AddOnlyVecIDType};
use crate::compressed_classfile::CompressedClassfileString;
use crate::compressed_classfile::names::PredefinedStrings;
use crate::compressed_classfile::names::PredefinedStrings::*;

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
    pub fn method_equals() -> Self {
        Self::from_raw_id(method_equals)
    }
    pub fn method_findSpecial() -> Self {
        Self::from_raw_id(method_findSpecial)
    }
    pub fn method_findConstructor() -> Self {
        Self::from_raw_id(method_findConstructor)
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
    pub fn method_getComponentType() -> Self {
        Self::from_raw_id(method_getComponentType)
    }
    pub fn method_newArray() -> Self {
        Self::from_raw_id(method_newArray)
    }
    pub fn method_registerNatives() -> Self {
        Self::from_raw_id(method_registerNatives)
    }
    pub fn method_addressSize() -> Self {
        Self::from_raw_id(method_addressSize)
    }
    pub fn method_arraycopy() -> Self {
        Self::from_raw_id(method_arraycopy)
    }
    pub fn method_compareAndSwapLong() -> Self {
        Self::from_raw_id(method_compareAndSwapLong)
    }
    pub fn method_compareAndSwapInt() -> Self {
        Self::from_raw_id(method_compareAndSwapInt)
    }
    pub fn method_compareAndSwapObject() -> Self {
        Self::from_raw_id(method_compareAndSwapObject)
    }
    pub fn method_identityHashCode() -> Self {
        Self::from_raw_id(method_identityHashCode)
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
    pub fn method_getProperty() -> Self {
        Self::from_raw_id(method_getProperty)
    }
    pub fn method_start() -> Self {
        Self::from_raw_id(method_start)
    }
    pub fn method_toString() -> Self {
        Self::from_raw_id(method_toString)
    }
    pub fn method_annotationType() -> Self {
        Self::from_raw_id(method_and_field_annotationType)
    }
    pub fn method_type() -> Self {
        Self::from_raw_id(field_and_method_type)
    }
    pub fn method_value() -> Self {
        Self::from_raw_id(field_and_method_value)
    }
    #[allow(clippy::self_named_constructors)]
    pub fn method_name() -> Self {
        Self::from_raw_id(field_and_method_name)
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
    pub fn method_getGenericInterfaces() -> Self {
        Self::from_raw_id(method_getGenericInterfaces)
    }
    pub fn method_parameterType() -> Self {
        Self::from_raw_id(field_and_method_parameterType)
    }
    pub fn method_methodType() -> Self {
        Self::from_raw_id(field_and_method_methodType)
    }
    pub fn method_putIfAbsent() -> Self {
        Self::from_raw_id(method_putIfAbsent)
    }
    pub fn method_get() -> Self {
        Self::from_raw_id(method_get)
    }
    pub fn method_destructiveMulAdd() -> Self {
        Self::from_raw_id(method_destructiveMulAdd)
    }
    pub fn method_getLong() -> Self{
        Self::from_raw_id(method_getLong)
    }
    pub fn method_getIntVolatile() -> Self{
        Self::from_raw_id(method_getIntVolatile)
    }
    pub fn method_allocateMemory() -> Self{
        Self::from_raw_id(method_allocateMemory)
    }
    pub fn method_putLong() -> Self{
        Self::from_raw_id(method_putLong)
    }
    pub fn method_getByte() -> Self{
        Self::from_raw_id(method_getByte)
    }
    pub fn method_freeMemory() -> Self{
        Self::from_raw_id(method_freeMemory)
    }

    pub fn method_getInt() -> Self {
        Self::from_raw_id(method_getInt)
    }
    pub fn method_getObject() -> Self {
        Self::from_raw_id(method_getObject)
    }
    pub fn method_getBoolean() -> Self {
        Self::from_raw_id(method_getBoolean)
    }
    pub fn method_getShort() -> Self {
        Self::from_raw_id(method_getShort)
    }
    pub fn method_getChar() -> Self {
        Self::from_raw_id(method_getChar)
    }
    pub fn method_getFloat() -> Self {
        Self::from_raw_id(method_getFloat)
    }
    pub fn method_getDouble() -> Self {
        Self::from_raw_id(method_getDouble)
    }

    pub fn method_putInt() -> Self{
        Self::from_raw_id(method_putInt)
    }
    pub fn method_putObject() -> Self{
        Self::from_raw_id(method_putObject)
    }
    pub fn method_putBoolean() -> Self{
        Self::from_raw_id(method_putBoolean)
    }
    pub fn method_putByte() -> Self{
        Self::from_raw_id(method_putByte)
    }
    pub fn method_putShort() -> Self{
        Self::from_raw_id(method_putShort)
    }
    pub fn method_putChar() -> Self{
        Self::from_raw_id(method_putChar)
    }
    pub fn method_putFloat() -> Self{
        Self::from_raw_id(method_putFloat)
    }
    pub fn method_putDouble() -> Self{
        Self::from_raw_id(method_putDouble)
    }

    pub fn method_getObjectVolatile() -> Self{
        Self::from_raw_id(method_getObjectVolatile)
    }
    pub fn method_getBooleanVolatile() -> Self{
        Self::from_raw_id(method_getBooleanVolatile)
    }
    pub fn method_getByteVolatile() -> Self{
        Self::from_raw_id(method_getByteVolatile)
    }
    pub fn method_getShortVolatile() -> Self{
        Self::from_raw_id(method_getShortVolatile)
    }
    pub fn method_getCharVolatile() -> Self{
        Self::from_raw_id(method_getCharVolatile)
    }
    pub fn method_getLongVolatile() -> Self{
        Self::from_raw_id(method_getLongVolatile)
    }
    pub fn method_getFloatVolatile() -> Self{
        Self::from_raw_id(method_getFloatVolatile)
    }
    pub fn method_getDoubleVolatile() -> Self{
        Self::from_raw_id(method_getDoubleVolatile)
    }
    pub fn method_putIntVolatile() -> Self{
        Self::from_raw_id(method_putIntVolatile)
    }
    pub fn method_putObjectVolatile() -> Self{
        Self::from_raw_id(method_putObjectVolatile)
    }
    pub fn method_putBooleanVolatile() -> Self{
        Self::from_raw_id(method_putBooleanVolatile)
    }
    pub fn method_putByteVolatile() -> Self{
        Self::from_raw_id(method_putByteVolatile)
    }
    pub fn method_putShortVolatile() -> Self{
        Self::from_raw_id(method_putShortVolatile)
    }
    pub fn method_putCharVolatile() -> Self{
        Self::from_raw_id(method_putCharVolatile)
    }
    pub fn method_putLongVolatile() -> Self{
        Self::from_raw_id(method_putLongVolatile)
    }
    pub fn method_putFloatVolatile() -> Self{
        Self::from_raw_id(method_putFloatVolatile)
    }
    pub fn method_putDoubleVolatile() -> Self{
        Self::from_raw_id(method_putDoubleVolatile)
    }
}
