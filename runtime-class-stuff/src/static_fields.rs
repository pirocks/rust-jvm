use std::collections::{HashMap, HashSet};
use std::ffi::c_void;
use std::ptr::NonNull;
use std::sync::{Arc, RwLock};
use itertools::Itertools;
use classfile_view::view::{ClassView, HasAccessFlags};
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::CPDType;
use rust_jvm_common::compressed_classfile::field_names::FieldName;
use rust_jvm_common::compressed_classfile::string_pool::CompressedClassfileStringPool;
use crate::accessor::Accessor;
use crate::field_numbers::{FieldNameAndClass};
use crate::RuntimeClass;


pub struct StaticField {
    data: NonNull<c_void>,
    field_type: CPDType,
}

impl Accessor for StaticField {
    fn expected_type(&self) -> CPDType {
        self.field_type
    }

    fn read_impl<T>(&self) -> T {
        unsafe { self.data.cast::<T>().as_ptr().read() }
    }

    fn write_impl<T>(&self, to_write: T) {
        unsafe { self.data.cast::<T>().as_ptr().write(to_write); }
    }
}

impl StaticField {
    pub fn new(cpdtype: CPDType) -> Arc<Self> {
        //todo make box smaller to match array packing
        Arc::new(Self {
            data: NonNull::new(Box::into_raw(box 0u64)).unwrap().cast(),
            field_type: cpdtype,
        })
    }

    pub fn raw_address(&self) -> NonNull<c_void> {
        self.data
    }
}

struct AllTheStaticFieldsInner{
    fields: HashMap<CClassName, HashMap<FieldName, Arc<StaticField>>>,
    already_loaded: HashSet<CClassName>
}

pub struct AllTheStaticFields<'gc> {
    //todo I guess I need loader in here to
    inner: RwLock<AllTheStaticFieldsInner>,
    string_pool: &'gc CompressedClassfileStringPool,
}

impl<'gc> AllTheStaticFields<'gc> {
    pub fn new(string_pool: &'gc CompressedClassfileStringPool) -> Self {
        Self {
            inner: RwLock::new(AllTheStaticFieldsInner{
                fields: Default::default(),
                already_loaded: Default::default()
            }),
            string_pool,
        }
    }

    pub fn get(&self, field_name_and_class: FieldNameAndClass) -> Arc<StaticField> {
        let FieldNameAndClass{ field_name, class_name } = field_name_and_class;
        self.inner.read().unwrap().fields.get(&class_name).unwrap().get(&field_name).unwrap().clone()
    }

    //ordered by first class is Object and/or interfaces
    pub fn sink_class_load(&self, static_fields: Vec<(CClassName, Vec<(FieldName, CPDType)>)>) {
        let mut write_guard = self.inner.write().unwrap();
        let mut current_fields: HashMap<FieldName, Arc<StaticField>> = HashMap::new();
        for (class_name, fields) in static_fields {
            if !write_guard.already_loaded.contains(&class_name){
                for (field_name, field) in current_fields.iter() {
                    let field_name = *field_name;
                    write_guard.fields.entry(class_name).or_default().insert(field_name, field.clone());
                }
                for (field_name, cpdtype) in fields {
                    let new_static_field = StaticField::new(cpdtype);
                    current_fields.insert(field_name, new_static_field.clone());
                    write_guard.fields.entry(class_name).or_default().insert(field_name, new_static_field);
                }
            }else {
                for (field_name, cpdtype) in fields {
                    let static_field = write_guard.fields.get(&class_name).unwrap().get(&field_name).unwrap().clone();
                    current_fields.insert(field_name, static_field);
                }
            }
            write_guard.already_loaded.insert(class_name);
        }
    }
}


fn get_fields_static_impl(class_view: &Arc<dyn ClassView>, parent: &Option<Arc<RuntimeClass>>, interfaces: &[Arc<RuntimeClass>], fields_res: &mut Vec<(CClassName, Vec<(FieldName, CPDType)>)>) {
    if let Some(parent) = parent {
        let parent = parent.unwrap_class_class();
        get_fields_static_impl(&parent.class_view, &parent.parent, parent.interfaces.as_slice(), fields_res)
    }
    for interface in interfaces {
        let interface = interface.unwrap_class_class();
        get_fields_static_impl(&interface.class_view, &interface.parent, interface.interfaces.as_slice(), fields_res);
    }
    let class_name = class_view.name().unwrap_name();
    let this_class_fields = class_view.fields().filter(|field_view| field_view.is_static()).map(|field_view| (field_view.field_name(), field_view.field_type())).collect_vec();
    fields_res.push((class_name, this_class_fields));
}

// returns vec with object/interfaces first, actual object last
pub fn get_fields_static(class_view: &Arc<dyn ClassView>, parent: &Option<Arc<RuntimeClass>>, interfaces: &[Arc<RuntimeClass>]) -> Vec<(CClassName, Vec<(FieldName, CPDType)>)> {
    let mut temp_vec = vec![];
    get_fields_static_impl(class_view, parent, interfaces, &mut temp_vec);
    temp_vec
}