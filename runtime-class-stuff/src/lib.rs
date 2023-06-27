#![allow(dead_code)]
#![allow(unused_variables)]
#![feature(int_roundings)]
#![feature(exclusive_range_pattern)]
#![feature(const_refs_to_cell)]
#![feature(strict_provenance_atomic_ptr)]
#![feature(vec_into_raw_parts)]
#![feature(entry_insert)]
#![feature(const_fmt_arguments_new)]

use std::collections::HashMap;
use std::fmt::{Debug, Error, Formatter};
use std::marker::PhantomData;
use std::num::NonZeroU8;
use std::ptr::NonNull;
use std::sync::{Arc, RwLock};

use classfile_view::view::{ArrayView, ClassBackedView, ClassView, HasAccessFlags, PrimitiveView};
use inheritance_tree::{ClassID, InheritanceTree};
use inheritance_tree::bit_vec_path::BitVecPaths;
use inheritance_tree::class_ids::ClassIDs;
use inheritance_tree::paths::{BitPath256, InheritanceClassIDPath};
use rust_jvm_common::compressed_classfile::compressed_types::CPDType;
use rust_jvm_common::compressed_classfile::string_pool::CompressedClassfileStringPool;
use rust_jvm_common::method_shape::MethodShape;

use crate::field_numbers::{FieldNameAndClass, FieldNumber, get_field_numbers};
use crate::method_numbers::{get_method_numbers, MethodNumber};
use crate::object_layout::ObjectLayout;
use crate::static_fields::{AllTheStaticFields, get_fields_static};

pub mod object_layout;
pub mod method_numbers;
pub mod field_numbers;
pub mod static_fields;
pub mod hidden_fields;
pub mod accessor;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ClassStatus {
    UNPREPARED,
    PREPARED,
    INITIALIZING,
    INITIALIZED,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum RuntimeClassPrimitive{
    Byte,
    Boolean,
    Short,
    Char,
    Int,
    Long,
    Float,
    Double,
    Void,
}

pub enum RuntimeClassRef<'gc, 'l> {
    Array(&'l RuntimeClassArray<'gc>),
    Object(&'l RuntimeClassClass<'gc>)
}

#[derive(Debug)]
pub enum RuntimeClass<'gc> {
    Primitive(RuntimeClassPrimitive),
    Array(RuntimeClassArray<'gc>),
    Object(RuntimeClassClass<'gc>),
}


impl<'gc> RuntimeClass<'gc> {
    pub fn cpdtype(&self) -> CPDType {
        match self {
            RuntimeClass::Primitive(RuntimeClassPrimitive::Byte) => CPDType::ByteType,
            RuntimeClass::Primitive(RuntimeClassPrimitive::Boolean) => CPDType::BooleanType,
            RuntimeClass::Primitive(RuntimeClassPrimitive::Short) => CPDType::ShortType,
            RuntimeClass::Primitive(RuntimeClassPrimitive::Char) => CPDType::CharType,
            RuntimeClass::Primitive(RuntimeClassPrimitive::Int) => CPDType::IntType,
            RuntimeClass::Primitive(RuntimeClassPrimitive::Long) => CPDType::LongType,
            RuntimeClass::Primitive(RuntimeClassPrimitive::Float) => CPDType::FloatType,
            RuntimeClass::Primitive(RuntimeClassPrimitive::Double) => CPDType::DoubleType,
            RuntimeClass::Primitive(RuntimeClassPrimitive::Void) => CPDType::VoidType,
            RuntimeClass::Array(arr) => CPDType::array(arr.sub_class.cpdtype()),
            RuntimeClass::Object(o) => CPDType::Class(o.class_view.name().unwrap_name()),
        }
    }
    pub fn view(&self) -> Arc<dyn ClassView> {
        match self {
            RuntimeClass::Primitive(RuntimeClassPrimitive::Byte) => Arc::new(PrimitiveView::Byte),
            RuntimeClass::Primitive(RuntimeClassPrimitive::Boolean) => Arc::new(PrimitiveView::Boolean),
            RuntimeClass::Primitive(RuntimeClassPrimitive::Short) => Arc::new(PrimitiveView::Short),
            RuntimeClass::Primitive(RuntimeClassPrimitive::Char) => Arc::new(PrimitiveView::Char),
            RuntimeClass::Primitive(RuntimeClassPrimitive::Int) => Arc::new(PrimitiveView::Int),
            RuntimeClass::Primitive(RuntimeClassPrimitive::Long) => Arc::new(PrimitiveView::Long),
            RuntimeClass::Primitive(RuntimeClassPrimitive::Float) => Arc::new(PrimitiveView::Float),
            RuntimeClass::Primitive(RuntimeClassPrimitive::Double) => Arc::new(PrimitiveView::Double),
            RuntimeClass::Primitive(RuntimeClassPrimitive::Void) => Arc::new(PrimitiveView::Void),
            RuntimeClass::Array(arr) => Arc::new(ArrayView { sub: arr.sub_class.view() }),
            RuntimeClass::Object(o) => o.class_view.clone(),
        }
    }
    pub fn status(&self) -> ClassStatus {
        match self {
            RuntimeClass::Primitive(RuntimeClassPrimitive::Byte) => ClassStatus::INITIALIZED,
            RuntimeClass::Primitive(RuntimeClassPrimitive::Boolean) => ClassStatus::INITIALIZED,
            RuntimeClass::Primitive(RuntimeClassPrimitive::Short) => ClassStatus::INITIALIZED,
            RuntimeClass::Primitive(RuntimeClassPrimitive::Char) => ClassStatus::INITIALIZED,
            RuntimeClass::Primitive(RuntimeClassPrimitive::Int) => ClassStatus::INITIALIZED,
            RuntimeClass::Primitive(RuntimeClassPrimitive::Long) => ClassStatus::INITIALIZED,
            RuntimeClass::Primitive(RuntimeClassPrimitive::Float) => ClassStatus::INITIALIZED,
            RuntimeClass::Primitive(RuntimeClassPrimitive::Double) => ClassStatus::INITIALIZED,
            RuntimeClass::Primitive(RuntimeClassPrimitive::Void) => ClassStatus::INITIALIZED,
            RuntimeClass::Array(a) => a.sub_class.status(),
            RuntimeClass::Object(o) => *o.status.read().unwrap(),
        }
    }

    pub fn set_status(&self, status: ClassStatus) {
        match self {
            RuntimeClass::Array(a) => a.sub_class.set_status(status),
            RuntimeClass::Object(o) => *o.status.write().unwrap() = status,
            _ => {}
        }
    }

    pub fn unwrap_class_class(&self) -> &RuntimeClassClass<'gc> {
        self.try_unwrap_class_class().unwrap()
    }

    pub fn unwrap_class_array(&self) -> &RuntimeClassArray<'gc> {
        match self {
            RuntimeClass::Array(arr) => arr,
            _ => panic!()
        }
    }

    pub fn try_unwrap_class_class(&'_ self) -> Option<&'_ RuntimeClassClass<'gc>> {
        match self {
            RuntimeClass::Object(classclass) => Some(classclass),
            _ => None,
        }
    }

    pub fn try_unwrap_ref(&self) -> Option<RuntimeClassRef<'gc,'_>>{
        match self {
            RuntimeClass::Primitive(_) => {
                None
            }
            RuntimeClass::Array(arr) => {
                Some(RuntimeClassRef::Array(arr))
            }
            RuntimeClass::Object(obj) => {
                Some(RuntimeClassRef::Object(obj))
            }
        }
    }
}

#[derive(Debug)]
pub struct RuntimeClassArray<'gc> {
    pub sub_class: Arc<RuntimeClass<'gc>>,
    pub num_nested_arrs: NonZeroU8,
    pub serializable: Arc<RuntimeClass<'gc>>,
    pub cloneable: Arc<RuntimeClass<'gc>>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct FieldNumberAndFieldType {
    pub number: FieldNumber,
    pub cpdtype: CPDType,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct FieldNameAndFieldType {
    pub name: FieldNameAndClass,
    pub cpdtype: CPDType,
}

pub struct RuntimeClassClass<'gc> {
    pub class_view: Arc<dyn ClassView>,
    pub method_numbers: HashMap<MethodShape, MethodNumber>,
    pub method_numbers_reverse: HashMap<MethodNumber, MethodShape>,
    pub object_layout: ObjectLayout,
    pub recursive_num_methods: u32,
    pub parent: Option<Arc<RuntimeClass<'gc>>>,
    pub interfaces: Vec<Arc<RuntimeClass<'gc>>>,
    pub class_id_path: Option<Vec<ClassID>>,
    pub inheritance_tree_vec: Option<NonNull<BitPath256>>,
    //n/a for interfaces
    //class may not be prepared
    pub status: RwLock<ClassStatus>,
    phantom: PhantomData<&'gc ()>,
}


//todo refactor to make it impossible to create RuntimeClassClass without registering to array, box leak jvm state to static

impl<'gc> RuntimeClassClass<'gc> {
    pub fn new_new(inheritance_tree: &InheritanceTree,
                   all_the_static_fields: &AllTheStaticFields,
                   bit_vec_paths: &mut BitVecPaths,
                   class_view: Arc<ClassBackedView>,
                   parent: Option<Arc<RuntimeClass<'gc>>>,
                   interfaces: Vec<Arc<RuntimeClass<'gc>>>,
                   status: RwLock<ClassStatus>,
                   _string_pool: &CompressedClassfileStringPool,
                   class_ids: &ClassIDs,
    ) -> Self {
        let class_id_path = get_class_id_path(&(class_view.clone() as Arc<dyn ClassView>), &parent, class_ids);
        let (recursive_num_methods, method_numbers) = get_method_numbers(&(class_view.clone() as Arc<dyn ClassView>), &parent, interfaces.as_slice());
        Self::new(inheritance_tree, all_the_static_fields, bit_vec_paths, class_view, parent, interfaces, status, method_numbers, recursive_num_methods, class_id_path,_string_pool)
    }

    pub fn new(
        inheritance_tree: &InheritanceTree,
        all_the_static_fields: &AllTheStaticFields,
        bit_vec_paths: &mut BitVecPaths,
        class_view: Arc<ClassBackedView>,
        parent: Option<Arc<RuntimeClass<'gc>>>,
        interfaces: Vec<Arc<RuntimeClass<'gc>>>,
        status: RwLock<ClassStatus>,
        method_numbers: HashMap<MethodShape, MethodNumber>,
        recursive_num_methods: u32,
        class_id_path: Vec<ClassID>,
        _string_pool: &CompressedClassfileStringPool,
    ) -> Self {
        let inheritance_tree_vec = if !class_view.is_interface() {
            match inheritance_tree.insert(&InheritanceClassIDPath::Borrowed { inner: class_id_path.as_slice() }).ok() {
                None => None,
                Some(inheritance_tree_vec) => {
                    let id = bit_vec_paths.lookup_or_add(inheritance_tree_vec);
                    Some(bit_vec_paths.get_ptr_from_id(id))
                }
            }
        } else {
            None
        };

        all_the_static_fields.sink_class_load(get_fields_static(&(class_view.clone() as Arc<dyn ClassView>), &parent, interfaces.as_slice()));

        let method_numbers_reverse = method_numbers.iter()
            .map(|(method_shape, method_number)| (*method_number, method_shape.clone()))
            .collect();
        let object_layout = ObjectLayout::new(&class_view, &parent);
        assert!(recursive_num_methods >= method_numbers.len() as u32);
        Self {
            class_view,
            method_numbers,
            method_numbers_reverse,
            object_layout,
            recursive_num_methods,
            parent,
            interfaces,
            class_id_path: Some(class_id_path),
            inheritance_tree_vec,
            status,
            phantom: Default::default(),
        }
    }

    pub fn num_vars(&self, static_: bool) -> usize {
        self.class_view.fields().filter(|field| {
            if static_ {
                field.is_static()
            } else {
                !field.is_static()
            }
        }).count() + self.parent.as_ref().map(|parent| parent.unwrap_class_class().num_vars(static_)).unwrap_or(0)
    }

    pub fn num_virtual_methods(&self) -> usize {
        self.class_view.methods().filter(|method| !method.is_static()).count() + self.parent.as_ref().map(|parent| parent.unwrap_class_class().num_virtual_methods()).unwrap_or(0)
    }
}

fn get_class_id_path<'gc>(class_view: &Arc<dyn ClassView>, parent: &Option<Arc<RuntimeClass<'gc>>>, class_ids: &ClassIDs) -> Vec<ClassID> {
    let mut res = vec![];
    get_class_id_path_impl(class_view, parent, class_ids, &mut res);
    res
}


fn get_class_id_path_impl<'gc>(class_view: &Arc<dyn ClassView>, parent: &Option<Arc<RuntimeClass<'gc>>>, class_ids: &ClassIDs, res: &mut Vec<ClassID>) {
    let class_id = class_ids.get_id_or_add(class_view.name().to_cpdtype());
    if let Some(parent) = parent {
        let class = parent.unwrap_class_class();
        get_class_id_path_impl(&class.class_view, &class.parent, class_ids, res);
    }
    res.push(class_id);
}

#[allow(unreachable_code)]
impl<'gc> Debug for RuntimeClassClass<'gc> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{:?}:{:?}", self.class_view.name(), todo!()/*self.static_vars*/)
    }
}


impl<'gc> From<RuntimeClassClass<'gc>> for RuntimeClass<'gc> {
    fn from(rcc: RuntimeClassClass<'gc>) -> Self {
        Self::Object(rcc)
    }
}
