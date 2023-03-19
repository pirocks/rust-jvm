use std::num::NonZeroUsize;
use std::ptr::{NonNull, null, null_mut};
use inheritance_tree::ClassID;
use inheritance_tree::paths::BitPath256;
use interface_vtable::ITableRaw;
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::{CPDType, CPRefType};
use rust_jvm_common::loading::LoaderName;
use vtable::RawNativeVTable;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum AllocatedObjectType {
    Class {
        name: CClassName,
        loader: LoaderName,
        // size: usize,
        vtable: NonNull<RawNativeVTable>,
        itable: NonNull<ITableRaw>,
        inheritance_bit_vec: Option<NonNull<BitPath256>>,
        interfaces: *const ClassID,
        interfaces_len: usize,
    },
    ObjectArray {
        sub_type: CPRefType,
        sub_type_loader: LoaderName,
        // len: i32,
        object_vtable: NonNull<RawNativeVTable>,
        array_itable: NonNull<ITableRaw>,
        array_interfaces: *const ClassID,
        interfaces_len: usize,
    },
    PrimitiveArray {
        primitive_type: CPDType,
        // len: i32,
        object_vtable: NonNull<RawNativeVTable>,
        array_itable: NonNull<ITableRaw>,
        array_interfaces: *const ClassID,
        interfaces_len: usize,
    },
    RawConstantSize {
        id: usize
        /*size: usize*/
    },
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct AllocatedObjectTypeWithSize {
    pub allocated_object_type: AllocatedObjectType,
    pub size: NonZeroUsize,
}

impl AllocatedObjectType {
    pub fn inheritance_bit_vec(&self) -> *const BitPath256 {
        match self {
            AllocatedObjectType::Class { inheritance_bit_vec, .. } => inheritance_bit_vec.map(|x| x.as_ptr() as *const BitPath256).unwrap_or(null()),
            AllocatedObjectType::ObjectArray { .. } |
            AllocatedObjectType::PrimitiveArray { .. } |
            AllocatedObjectType::RawConstantSize { .. } => {
                null_mut()
            }
        }
    }


    pub fn vtable(&self) -> Option<NonNull<RawNativeVTable>> {
        match self {
            AllocatedObjectType::Class { vtable, .. } => {
                Some(*vtable)
            }
            AllocatedObjectType::ObjectArray { object_vtable, .. } => Some(*object_vtable),
            AllocatedObjectType::PrimitiveArray { object_vtable, .. } => Some(*object_vtable),
            AllocatedObjectType::RawConstantSize { .. } => None,
        }
    }

    pub fn itable(&self) -> Option<NonNull<ITableRaw>> {
        match self {
            AllocatedObjectType::Class { itable, .. } => {
                Some(*itable)
            }
            AllocatedObjectType::ObjectArray { array_itable, .. } => Some(*array_itable),
            AllocatedObjectType::PrimitiveArray { array_itable, .. } => Some(*array_itable),
            AllocatedObjectType::RawConstantSize { .. } => None,
        }
    }

    pub fn as_cpdtype(&self) -> CPDType {
        match self {
            AllocatedObjectType::Class { name, .. } => {
                (*name).into()
            }
            AllocatedObjectType::ObjectArray { sub_type, .. } => {
                CPDType::array(sub_type.to_cpdtype())
            }
            AllocatedObjectType::PrimitiveArray { primitive_type, .. } => {
                CPDType::array(*primitive_type)
            }
            AllocatedObjectType::RawConstantSize { .. } => {
                panic!()
            }
        }
    }

    pub fn interfaces_ptr(&self) -> *const ClassID {
        *match self {
            AllocatedObjectType::Class { interfaces, .. } => {
                interfaces
            }
            AllocatedObjectType::ObjectArray { array_interfaces, .. } => {
                array_interfaces
            }
            AllocatedObjectType::PrimitiveArray { array_interfaces, .. } => {
                array_interfaces
            }
            AllocatedObjectType::RawConstantSize { .. } => &null()
        }
    }

    pub fn interfaces_len(&self) -> usize {
        *match self {
            AllocatedObjectType::Class { interfaces_len, .. } => {
                interfaces_len
            }
            AllocatedObjectType::ObjectArray { interfaces_len, .. } => {
                interfaces_len
            }
            AllocatedObjectType::PrimitiveArray { interfaces_len, .. } => {
                interfaces_len
            }
            AllocatedObjectType::RawConstantSize { .. } => &0
        }
    }

    pub fn constant_size_type(&self) -> bool {
        match self {
            AllocatedObjectType::Class { .. } => true,
            AllocatedObjectType::ObjectArray { .. } => false,
            AllocatedObjectType::PrimitiveArray { .. } => false,
            AllocatedObjectType::RawConstantSize { .. } => true,
        }
    }

    pub fn array_subtype(&self) -> Option<CPDType> {
        match self {
            AllocatedObjectType::Class { .. } => None,
            AllocatedObjectType::ObjectArray { sub_type,.. } => {
                Some(sub_type.to_cpdtype())
            },
            AllocatedObjectType::PrimitiveArray { primitive_type,.. } => {
                Some(*primitive_type)
            },
            AllocatedObjectType::RawConstantSize { .. } => None,
        }
    }

    pub fn is_array(&self) -> bool{
        match self {
            AllocatedObjectType::Class { .. } => false,
            AllocatedObjectType::ObjectArray { .. } => true,
            AllocatedObjectType::PrimitiveArray { .. } => true,
            AllocatedObjectType::RawConstantSize { .. } => false,
        }
    }
}

