use std::collections::HashMap;
use std::ffi::c_void;
use std::mem::size_of;
use std::num::NonZeroUsize;
use std::ptr::NonNull;
use std::sync::Arc;
use itertools::Itertools;


use classfile_view::view::{ClassBackedView, ClassView, HasAccessFlags};
use jvmti_jni_bindings::{jboolean, jbyte, jchar, jdouble, jfloat, jint, jlong, jobject, jshort};
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::{CompressedParsedRefType, CPDType};



use crate::{FieldNameAndClass, FieldNameAndFieldType, FieldNumber, FieldNumberAndFieldType, get_field_numbers, RuntimeClass};
use crate::hidden_fields::{HiddenJVMField, HiddenJVMFieldAndFieldType};

#[derive(Clone)]
pub struct ObjectLayout {
    pub hidden_field_numbers: HashMap<HiddenJVMField, FieldNumberAndFieldType>,
    pub hidden_field_numbers_reverse: HashMap<FieldNumber, HiddenJVMFieldAndFieldType>,
    pub field_numbers: HashMap<FieldNameAndClass, FieldNumberAndFieldType>,
    pub field_numbers_reverse: HashMap<FieldNumber, FieldNameAndFieldType>,
    pub recursive_num_fields: u32,
    recursive_num_fields_non_hidden: u32,
}




fn reverse_hidden_fields(hidden_field_numbers_reverse: &HashMap<FieldNumber, HiddenJVMFieldAndFieldType>) -> HashMap<HiddenJVMField, FieldNumberAndFieldType> {
    hidden_field_numbers_reverse.clone().into_iter()
        .map(|(number, HiddenJVMFieldAndFieldType { name, cpdtype })| (name, FieldNumberAndFieldType { number, cpdtype }))
        .collect()
}


impl ObjectLayout {
    pub fn new<'gc>(class_view: &Arc<ClassBackedView>, parent: &Option<Arc<RuntimeClass<'gc>>>) -> Self {
        let field_numbers = get_field_numbers(class_view, &parent);
        let (field_numbers, field_numbers_reverse) = field_numbers.reverse_fields();
        let mut recursive_num_fields = field_numbers_reverse.len() as u32;
        assert_eq!(field_numbers_reverse.keys().map(|num|num.0).sorted().collect_vec(), (0..recursive_num_fields).collect_vec());
        //todo hidden fields won't work with non-final classes
        let hidden_fields = if class_view.name() == CompressedParsedRefType::Class(CClassName::class()) {
            HiddenJVMField::class_hidden_fields()
        } else {
            vec![]
        };
        if !hidden_fields.is_empty() {
            assert!(class_view.is_final());
        }

        let hidden_field_numbers_reverse: HashMap<FieldNumber, HiddenJVMFieldAndFieldType> = hidden_fields.into_iter().map(|HiddenJVMFieldAndFieldType { name, cpdtype }| {
            let field_number = FieldNumber(recursive_num_fields);
            recursive_num_fields += 1;
            (field_number, HiddenJVMFieldAndFieldType { name, cpdtype })
        }).collect();

        let hidden_field_numbers = reverse_hidden_fields(&hidden_field_numbers_reverse);

        assert_eq!(hidden_field_numbers.len(), hidden_field_numbers_reverse.len());
        let recursive_num_fields_non_hidden = field_numbers_reverse.len() as u32;
        Self {
            hidden_field_numbers,
            hidden_field_numbers_reverse,
            field_numbers,
            field_numbers_reverse,
            recursive_num_fields,
            recursive_num_fields_non_hidden,
        }
    }

    pub fn self_check(&self) {
        assert_eq!(self.field_numbers_reverse.len() + self.hidden_field_numbers.len(), self.recursive_num_fields as usize);
        assert_eq!(self.hidden_field_numbers.len(), self.hidden_field_numbers_reverse.len());
        assert_eq!(self.recursive_num_fields_non_hidden as usize, self.field_numbers_reverse.len());
    }

    pub fn field_entry_offset(&self, field_number: FieldNumber) -> usize {
        assert!(field_number.0 < self.recursive_num_fields());
        (field_number.0 as usize) * size_of::<u64>()
    }

    pub fn field_entry_type(&self, field_number: FieldNumber) -> CPDType{
        self.self_check();
        match &self.field_numbers_reverse.get(&field_number) {
            Some(FieldNameAndFieldType{  cpdtype, .. }) => *cpdtype,
            None => {
                self.hidden_field_numbers_reverse.get(&field_number).unwrap().cpdtype
            },
        }
    }

    pub fn field_entry_pointer(&self, object: NonNull<c_void>, field_number: FieldNumber) -> FieldAccessor {
        self.self_check();
        let inner_ptr = NonNull::new(unsafe { object.as_ptr().offset(self.field_entry_offset(field_number) as isize) }).unwrap();
        FieldAccessor{
            expected_type: self.field_entry_type(field_number),
            inner: inner_ptr
        }
    }

    pub fn recursive_num_fields(&self) -> u32 {
        self.recursive_num_fields
    }

    pub fn size(&self) -> NonZeroUsize {
        let res_size = self.recursive_num_fields() as usize * size_of::<u64>();
        //can't have zero size objects
        NonZeroUsize::new(res_size).unwrap_or(NonZeroUsize::new(1).unwrap())
    }
}

// todo dup with array layout
#[derive(Copy, Clone)]
pub struct FieldAccessor{
    expected_type: CPDType,
    inner: NonNull<c_void>
}

impl FieldAccessor{
    fn read_impl<T>(self) -> T{
        unsafe { self.inner.cast::<T>().as_ptr().read() }
    }

    pub fn read_boolean(self) -> jboolean{
        assert_eq!(CPDType::BooleanType, self.expected_type);
        self.read_impl()
    }

    pub fn read_byte(self) -> jbyte{
        assert_eq!(CPDType::ByteType, self.expected_type);
        self.read_impl()
    }

    pub fn read_short(self) -> jshort{
        assert_eq!(CPDType::ShortType, self.expected_type);
        self.read_impl()
    }

    pub fn read_char(self) -> jchar{
        assert_eq!(CPDType::CharType, self.expected_type);
        self.read_impl()
    }

    pub fn read_int(self) -> jint{
        assert_eq!(CPDType::IntType, self.expected_type);
        self.read_impl()
    }

    pub fn read_float(self) -> jfloat{
        assert_eq!(CPDType::FloatType, self.expected_type);
        self.read_impl()
    }

    pub fn read_long(self) -> jlong{
        assert_eq!(CPDType::LongType, self.expected_type);
        self.read_impl()
    }

    pub fn read_double(self) -> jdouble{
        assert_eq!(CPDType::FloatType, self.expected_type);
        self.read_impl()
    }

    pub fn read_object(self) -> jobject{
        assert!(self.expected_type.try_unwrap_ref_type().is_some());
        self.read_impl()
    }


    fn write_impl<T>(self, to_write: T){
        unsafe { self.inner.cast::<T>().as_ptr().write(to_write) }
    }

    pub fn write_boolean(self, to_write: jboolean) {
        assert_eq!(CPDType::BooleanType, self.expected_type);
        self.write_impl(to_write)
    }

    pub fn write_byte(self, to_write: jbyte) {
        assert_eq!(CPDType::ByteType, self.expected_type);
        self.write_impl(to_write)
    }

    pub fn write_short(self, to_write: jshort) {
        assert_eq!(CPDType::ShortType, self.expected_type);
        self.write_impl(to_write)
    }

    pub fn write_char(self, to_write: jchar) {
        assert_eq!(CPDType::CharType, self.expected_type);
        self.write_impl(to_write)
    }

    pub fn write_int(self, to_write: jint) {
        assert_eq!(CPDType::IntType, self.expected_type);
        self.write_impl(to_write)
    }

    pub fn write_float(self, to_write: jfloat) {
        assert_eq!(CPDType::FloatType, self.expected_type);
        self.write_impl(to_write)
    }

    pub fn write_long(self, to_write: jlong) {
        assert_eq!(CPDType::LongType, self.expected_type);
        self.write_impl(to_write)
    }

    pub fn write_double(self, to_write: jdouble) {
        assert_eq!(CPDType::FloatType, self.expected_type);
        self.write_impl(to_write)
    }

    pub fn write_object(self, to_write: jobject) {
        assert!(self.expected_type.try_unwrap_ref_type().is_some());
        self.write_impl(to_write)
    }
}

