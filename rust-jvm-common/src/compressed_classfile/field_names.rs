use add_only_static_vec::{AddOnlyId, AddOnlyVecIDType};
use crate::compressed_classfile::CompressedClassfileString;
use crate::compressed_classfile::names::PredefinedStrings;
use crate::compressed_classfile::names::PredefinedStrings::*;

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
        Self::from_raw_id(method_and_field_annotationType)
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
    pub fn field_ht() -> Self {
        Self::from_raw_id(field_ht)
    }
    pub fn field_getContextClassLoader() -> Self {
        Self::from_raw_id(field_and_method_getContextClassLoader)
    }
    pub fn field_key() -> Self {
        Self::from_raw_id(field_key)
    }
    pub fn field_int_len() -> Self {
        Self::from_raw_id(field_int_len)
    }
    pub fn field_val() -> Self {
        Self::from_raw_id(field_val)
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
    pub fn field_map() -> Self {
        Self::from_raw_id(field_map)
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
    #[allow(clippy::self_named_constructors)]
    pub fn field_name() -> Self {
        Self::from_raw_id(field_and_method_name)
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
    pub fn field_table() -> Self {
        Self::from_raw_id(field_table)
    }
    pub fn field_sizeCtl() -> Self {
        Self::from_raw_id(field_sizeCtl)
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
    pub fn field_hash() -> Self {
        Self::from_raw_id(field_hash)
    }
    pub fn field_next() -> Self {
        Self::from_raw_id(field_next)
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
    pub fn field_signum() -> Self {
        Self::from_raw_id(field_signum)
    }
    pub fn field_mag() -> Self {
        Self::from_raw_id(field_mag)
    }
    pub fn field_formalTypeParams() -> Self {
        Self::from_raw_id(field_formalTypeParams)
    }
}

