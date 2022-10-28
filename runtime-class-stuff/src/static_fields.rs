use std::collections::{HashMap, HashSet};
use std::ffi::c_void;
use std::ops::Deref;
use std::ptr::NonNull;
use std::sync::{Arc, RwLock};
use classfile_view::view::ClassBackedView;
use jvmti_jni_bindings::{jboolean, jbyte, jchar, jdouble, jfloat, jint, jlong, jobject, jshort};
use rust_jvm_common::compressed_classfile::compressed_types::CPDType;
use rust_jvm_common::compressed_classfile::string_pool::CompressedClassfileStringPool;
use crate::accessor::Accessor;
use crate::field_numbers::{FieldNameAndClass, get_fields};
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

    //todo this dup is getting problematic make a trait?

    //todo make not public
    pub fn read_impl<T>(&self) -> T {
        unsafe { self.data.cast::<T>().as_ptr().read() }
    }

    pub fn read_boolean(&self) -> jboolean {
        assert_eq!(CPDType::BooleanType, self.field_type);
        self.read_impl()
    }

    pub fn read_byte(&self) -> jbyte {
        assert_eq!(CPDType::ByteType, self.field_type);
        self.read_impl()
    }

    pub fn read_short(&self) -> jshort {
        assert_eq!(CPDType::ShortType, self.field_type);
        self.read_impl()
    }

    pub fn read_char(&self) -> jchar {
        assert_eq!(CPDType::CharType, self.field_type);
        self.read_impl()
    }

    pub fn read_int(&self) -> jint {
        assert_eq!(CPDType::IntType, self.field_type);
        self.read_impl()
    }

    pub fn read_float(&self) -> jfloat {
        assert_eq!(CPDType::FloatType, self.field_type);
        self.read_impl()
    }

    pub fn read_long(&self) -> jlong {
        assert_eq!(CPDType::LongType, self.field_type);
        self.read_impl()
    }

    pub fn read_double(&self) -> jdouble {
        assert_eq!(CPDType::FloatType, self.field_type);
        self.read_impl()
    }

    pub fn read_object(&self) -> jobject {
        assert!(&self.field_type.try_unwrap_ref_type().is_some());
        self.read_impl()
    }

    //todo make not public
    pub fn write_impl<T>(&self, to_write: T) {
        unsafe { self.data.cast::<T>().as_ptr().write(to_write) }
    }

    pub fn write_boolean(&self, to_write: jboolean) {
        assert_eq!(CPDType::BooleanType, self.field_type);
        self.write_impl(to_write)
    }

    pub fn write_byte(&self, to_write: jbyte) {
        assert_eq!(CPDType::ByteType, self.field_type);
        self.write_impl(to_write)
    }

    pub fn write_short(&self, to_write: jshort) {
        assert_eq!(CPDType::ShortType, self.field_type);
        self.write_impl(to_write)
    }

    pub fn write_char(&self, to_write: jchar) {
        assert_eq!(CPDType::CharType, self.field_type);
        self.write_impl(to_write)
    }

    pub fn write_int(&self, to_write: jint) {
        assert_eq!(CPDType::IntType, self.field_type);
        self.write_impl(to_write)
    }

    pub fn write_float(&self, to_write: jfloat) {
        assert_eq!(CPDType::FloatType, self.field_type);
        self.write_impl(to_write)
    }

    pub fn write_long(&self, to_write: jlong) {
        assert_eq!(CPDType::LongType, self.field_type);
        self.write_impl(to_write)
    }
}

pub struct AllTheStaticFields<'gc> {
    //todo I guess I need loader in here to
    fields: RwLock<HashMap<FieldNameAndClass, Arc<StaticField>>>,
    string_pool: &'gc CompressedClassfileStringPool,
}

impl<'gc> AllTheStaticFields<'gc> {
    pub fn new(string_pool: &'gc CompressedClassfileStringPool) -> Self {
        Self {
            fields: Default::default(),
            string_pool,
        }
    }

    pub fn get(&self, field_name_and_class: FieldNameAndClass) -> Arc<StaticField> {
        self.fields.read().unwrap().get(&field_name_and_class).unwrap().clone()
    }

    pub fn sink_class_load(&self, static_fields: HashMap<FieldNameAndClass, (HashSet<FieldNameAndClass>, CPDType)>) {
        let mut write_guard = self.fields.write().unwrap();
        for (field_and_name_class, (aliases, cpdtype)) in static_fields {
            let static_field = write_guard.entry(field_and_name_class).or_insert(StaticField::new(cpdtype)).clone();
            for alias in aliases {
                dbg!(alias.class_name.0.to_str(&self.string_pool));
                dbg!(alias.field_name.0.to_str(&self.string_pool));
                let inserted = write_guard.entry(alias).or_insert(static_field.clone()).clone();
                assert!(Arc::ptr_eq(&inserted, &static_field));
            }
        }
    }
}


pub fn get_field_numbers_static(class_view: &Arc<ClassBackedView>, parent: &Option<Arc<RuntimeClass>>) -> HashMap<FieldNameAndClass, (HashSet<FieldNameAndClass>, CPDType)> {
    let mut temp_vec = vec![];
    get_fields(class_view.deref(), parent, true, &mut temp_vec);
    let mut res = HashMap::new();
    for (i, (class_name, fields)) in temp_vec.iter().enumerate() {
        let class_name = *class_name;
        let subclasses = &temp_vec[i..];
        for (field_name, cpdtype) in fields.into_iter().cloned() {
            let field_name_and_class = FieldNameAndClass { field_name, class_name };
            let mut alliases = HashSet::new();
            for (subclass, fields) in subclasses.into_iter().cloned() {
                if !fields.iter().any(|(sub_field_name, _)| {
                    *sub_field_name == field_name
                }) {
                    alliases.insert(FieldNameAndClass { field_name, class_name: subclass });
                }
            }
            res.insert(field_name_and_class, (alliases, cpdtype));
        }
    }
    res
}