use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::fmt::{Debug, Error, Formatter};
use std::sync::{Arc, RwLock};

use classfile_view::view::{ArrayView, ClassBackedView, ClassView, HasAccessFlags, PrimitiveView};
use inheritance_tree::{ClassID, InheritanceTree};
use inheritance_tree::class_ids::ClassIDs;
use inheritance_tree::paths::InheritanceClassIDPath;
use rust_jvm_common::compressed_classfile::{CompressedClassfileStringPool, CPDType};
use rust_jvm_common::compressed_classfile::names::FieldName;
use rust_jvm_common::method_shape::MethodShape;
use rust_jvm_common::NativeJavaValue;

use crate::field_numbers::{FieldNumber, get_field_numbers, get_field_numbers_static};
use crate::method_numbers::{get_method_numbers, MethodNumber};

pub mod method_numbers;
pub mod field_numbers;


#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ClassStatus {
    UNPREPARED,
    PREPARED,
    INITIALIZING,
    INITIALIZED,
}

#[derive(Debug)]
pub enum RuntimeClass<'gc> {
    Byte,
    Boolean,
    Short,
    Char,
    Int,
    Long,
    Float,
    Double,
    Void,
    Array(RuntimeClassArray<'gc>),
    Object(RuntimeClassClass<'gc>),
    Top,
}


impl<'gc> RuntimeClass<'gc> {
    pub fn cpdtype(&self) -> CPDType {
        match self {
            RuntimeClass::Byte => CPDType::ByteType,
            RuntimeClass::Boolean => CPDType::BooleanType,
            RuntimeClass::Short => CPDType::ShortType,
            RuntimeClass::Char => CPDType::CharType,
            RuntimeClass::Int => CPDType::IntType,
            RuntimeClass::Long => CPDType::LongType,
            RuntimeClass::Float => CPDType::FloatType,
            RuntimeClass::Double => CPDType::DoubleType,
            RuntimeClass::Void => CPDType::VoidType,
            RuntimeClass::Array(arr) => CPDType::array(arr.sub_class.cpdtype()),
            RuntimeClass::Object(o) => CPDType::Class(o.class_view.name().unwrap_name()),
            RuntimeClass::Top => panic!(),
        }
    }
    pub fn view(&self) -> Arc<dyn ClassView> {
        match self {
            RuntimeClass::Byte => Arc::new(PrimitiveView::Byte),
            RuntimeClass::Boolean => Arc::new(PrimitiveView::Boolean),
            RuntimeClass::Short => Arc::new(PrimitiveView::Short),
            RuntimeClass::Char => Arc::new(PrimitiveView::Char),
            RuntimeClass::Int => Arc::new(PrimitiveView::Int),
            RuntimeClass::Long => Arc::new(PrimitiveView::Long),
            RuntimeClass::Float => Arc::new(PrimitiveView::Float),
            RuntimeClass::Double => Arc::new(PrimitiveView::Double),
            RuntimeClass::Void => Arc::new(PrimitiveView::Void),
            RuntimeClass::Array(arr) => Arc::new(ArrayView { sub: arr.sub_class.view() }),
            RuntimeClass::Object(o) => o.class_view.clone(),
            RuntimeClass::Top => panic!(),
        }
    }
    pub fn status(&self) -> ClassStatus {
        match self {
            RuntimeClass::Byte => ClassStatus::INITIALIZED,
            RuntimeClass::Boolean => ClassStatus::INITIALIZED,
            RuntimeClass::Short => ClassStatus::INITIALIZED,
            RuntimeClass::Char => ClassStatus::INITIALIZED,
            RuntimeClass::Int => ClassStatus::INITIALIZED,
            RuntimeClass::Long => ClassStatus::INITIALIZED,
            RuntimeClass::Float => ClassStatus::INITIALIZED,
            RuntimeClass::Double => ClassStatus::INITIALIZED,
            RuntimeClass::Void => ClassStatus::INITIALIZED,
            RuntimeClass::Array(a) => a.sub_class.status(),
            RuntimeClass::Object(o) => *o.status.read().unwrap(),
            RuntimeClass::Top => panic!(),
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
}

#[derive(Debug)]
pub struct RuntimeClassArray<'gc> {
    pub sub_class: Arc<RuntimeClass<'gc>>,
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
    pub name: FieldName,
    pub cpdtype: CPDType,
}

pub struct RuntimeClassClass<'gc> {
    pub class_view: Arc<dyn ClassView>,
    pub field_numbers: HashMap<FieldName, FieldNumberAndFieldType>,
    pub field_numbers_reverse: HashMap<FieldNumber, FieldNameAndFieldType>,
    pub method_numbers: HashMap<MethodShape, MethodNumber>,
    pub method_numbers_reverse: HashMap<MethodNumber, MethodShape>,
    pub recursive_num_fields: u32,
    pub recursive_num_methods: u32,
    pub static_field_numbers: HashMap<FieldName, FieldNumberAndFieldType>,
    pub static_field_numbers_reverse: HashMap<FieldNumber, FieldNameAndFieldType>,
    pub static_vars: Vec<UnsafeCell<NativeJavaValue<'gc>>>,
    pub parent: Option<Arc<RuntimeClass<'gc>>>,
    pub interfaces: Vec<Arc<RuntimeClass<'gc>>>,
    pub class_id_path: Option<Vec<ClassID>>,
    //n/a for interfaces
    //class may not be prepared
    pub status: RwLock<ClassStatus>,
}


//todo refactor to make it impossible to create RuntimeClassClass without registering to array, box leak jvm state to static

impl<'gc> RuntimeClassClass<'gc> {
    pub fn new_new(inheritance_tree: &InheritanceTree,
                   class_view: Arc<ClassBackedView>,
                   parent: Option<Arc<RuntimeClass<'gc>>>,
                   interfaces: Vec<Arc<RuntimeClass<'gc>>>,
                   status: RwLock<ClassStatus>,
                   _string_pool: &CompressedClassfileStringPool,
                   class_ids: &ClassIDs
    ) -> Self {
        let class_id_path = get_class_id_path(&(class_view.clone() as Arc<dyn ClassView>), &parent, class_ids);
        let (recursive_num_fields, field_numbers) = get_field_numbers(&class_view, &parent);
        let (recursive_num_methods, method_numbers) = get_method_numbers(&(class_view.clone() as Arc<dyn ClassView>), &parent, interfaces.as_slice());
        let (recursive_num_static_fields, static_field_numbers) = get_field_numbers_static(&class_view, &parent);
        Self::new(inheritance_tree, class_view, recursive_num_fields, parent, interfaces, status, field_numbers, recursive_num_methods, method_numbers, recursive_num_static_fields, static_field_numbers, class_id_path)
    }

    pub fn new(
        inheritance_tree: &InheritanceTree,
        class_view: Arc<dyn ClassView>,
        recursive_num_fields: u32,
        parent: Option<Arc<RuntimeClass<'gc>>>,
        interfaces: Vec<Arc<RuntimeClass<'gc>>>,
        status: RwLock<ClassStatus>,
        field_numbers: HashMap<FieldName, (FieldNumber, CPDType)>,
        recursive_num_methods: u32,
        method_numbers: HashMap<MethodShape, MethodNumber>,
        recursive_num_static_fields: u32,
        static_field_numbers: HashMap<FieldName, (FieldNumber, CPDType)>,
        class_id_path: Vec<ClassID>,
    ) -> Self {
        fn reverse_fields(field_numbers: HashMap<FieldName, (FieldNumber, CPDType)>) -> (HashMap<FieldName, FieldNumberAndFieldType>, HashMap<FieldNumber, FieldNameAndFieldType>) {
            let reverse = field_numbers.clone().into_iter()
                .map(|(name, (number, cpdtype))| (number, FieldNameAndFieldType { name, cpdtype }))
                .collect();
            let forward = field_numbers.into_iter()
                .map(|(name, (number, cpdtype))| (name, FieldNumberAndFieldType { number, cpdtype }))
                .collect();
            (forward, reverse)
        }

        if !class_view.is_interface(){
            inheritance_tree.insert(&InheritanceClassIDPath::Borrowed { inner: class_id_path.as_slice() });
        }

        let (field_numbers, field_numbers_reverse) = reverse_fields(field_numbers);

        let (static_field_numbers, static_field_numbers_reverse) = reverse_fields(static_field_numbers);

        let method_numbers_reverse = method_numbers.iter()
            .map(|(method_shape, method_number)| (method_number.clone(), method_shape.clone()))
            .collect();
        assert!(recursive_num_fields >= field_numbers.len() as u32);
        assert!(recursive_num_methods >= method_numbers.len() as u32);
        Self {
            class_view,
            field_numbers,
            field_numbers_reverse,
            method_numbers,
            method_numbers_reverse,
            recursive_num_fields,
            recursive_num_methods,
            static_field_numbers,
            static_field_numbers_reverse,
            static_vars: (0..recursive_num_static_fields).map(|_| UnsafeCell::new(NativeJavaValue { as_u64: 0 })).collect(),
            parent,
            interfaces,
            class_id_path: Some(class_id_path),
            status,
        }
    }

    pub fn num_vars(&self) -> usize {
        self.class_view.fields().filter(|field| !field.is_static()).count() + self.parent.as_ref().map(|parent| parent.unwrap_class_class().num_vars()).unwrap_or(0)
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


impl<'gc> std::convert::From<RuntimeClassClass<'gc>> for RuntimeClass<'gc> {
    fn from(rcc: RuntimeClassClass<'gc>) -> Self {
        Self::Object(rcc)
    }
}
