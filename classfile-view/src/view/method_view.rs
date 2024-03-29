use wtf8::Wtf8Buf;

use classfile_parser::attribute_infos::{annotation_default_to_bytes, parameter_annotations_to_bytes, runtime_annotations_to_bytes};
use rust_jvm_common::classfile::{AttributeType, Code, LineNumberTable, LocalVariableTableEntry, MethodInfo};
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::code::CompressedCode;
use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPDType};
use rust_jvm_common::compressed_classfile::method_names::MethodName;
use rust_jvm_common::compressed_classfile::string_pool::CCString;


use rust_jvm_common::descriptor_parser::{FieldDescriptor, parse_field_descriptor};
use rust_jvm_common::method_shape::{MethodShape, MethodShapeID, MethodShapeIDs};

use crate::view::{ClassBackedView, ClassView, HasAccessFlags};

pub struct MethodView<'cl> {
    pub(crate) class_view: &'cl ClassBackedView,
    pub(crate) method_i: u16,
}

impl Clone for MethodView<'_> {
    fn clone(&self) -> Self {
        Self { class_view: self.class_view, method_i: self.method_i }
    }
}

impl HasAccessFlags for MethodView<'_> {
    fn access_flags(&self) -> u16 {
        self.class_view.backing_class.methods[self.method_i as usize].access_flags
    }
}

impl MethodView<'_> {
    fn from(c: &ClassBackedView, i: u16) -> MethodView {
        MethodView { class_view: c, method_i: i }
    }

    pub fn classview(&self) -> &ClassBackedView {
        self.class_view
    }

    fn method_info(&self) -> &MethodInfo {
        &self.class_view.underlying_class.methods[self.method_i as usize]
    }

    pub fn name(&self) -> MethodName {
        MethodName(self.class_view.backing_class.methods[self.method_i as usize].name)
    }

    pub fn desc_str(&self) -> CCString {
        self.class_view.backing_class.methods[self.method_i as usize].descriptor_str
    }

    pub fn desc(&self) -> &CMethodDescriptor {
        &self.class_view.backing_class.methods[self.method_i as usize].descriptor
    }

    pub fn code_attribute(&self) -> Option<&CompressedCode> {
        self.class_view.backing_class.methods[self.method_i as usize].code.as_ref()
    }

    pub fn real_code_attribute(&self) -> Option<&Code> {
        self.class_view.underlying_class.methods[self.method_i as usize].code_attribute()
    }

    pub fn local_variable_attribute(&self) -> Option<Vec<LocalVariableView>> {
        match self.method_info().code_attribute() {
            None => None,
            Some(code) => code
                .attributes
                .iter()
                .find_map(|attr| match &attr.attribute_type {
                    AttributeType::LocalVariableTable(lvt) => Some(lvt),
                    _ => None,
                })
                .map(|lvt| {
                    lvt.local_variable_table
                        .iter()
                        .map(|entry| {
                            let local_variable_entry = entry;
                            LocalVariableView { method_view: self, local_variable_entry }
                        })
                        .collect::<Vec<LocalVariableView>>()
                }),
        }
    }

    pub fn is_signature_polymorphic(&self) -> bool {
        // from the spec:
        // A method is signature polymorphic if all of the following are true:
        // •  It is declared in the java.lang.invoke.MethodHandle class.
        // •  It has a single formal parameter of type Object[].
        // •  It has a return type of Object.
        // •  It has the ACC_VARARGS and ACC_NATIVE flags set.
        self.class_view.name() == CClassName::method_handle().into() && self.desc().arg_types.len() == 1 && self.desc().arg_types[0] == CPDType::array(CPDType::object()) && self.desc().return_type == CPDType::object() && self.is_varargs() && self.is_native()
        //todo
    }

    //todo this shouldn't be public but needs to be atm.
    pub fn method_i(&self) -> u16 {
        self.method_i
    }

    pub fn num_args(&self) -> u16 {
        self.desc().arg_types.len() as u16
    }

    pub fn line_number_table(&self) -> Option<&LineNumberTable> {
        self.method_info().code_attribute().and_then(|attr| {
            attr.attributes.iter().find_map(|attr| match &attr.attribute_type {
                AttributeType::LineNumberTable(lnt) => Some(lnt),
                _ => None,
            })
        })
    }

    pub fn method_shape(&self) -> MethodShape {
        MethodShape {
            name: self.name(),
            desc: self.desc().clone(),
        }
    }

    pub fn method_shape_id(&self, method_shapes: &MethodShapeIDs) -> MethodShapeID {
        method_shapes.lookup_method_shape_id(self.method_shape())
    }

    pub fn get_annotation_bytes(&self) -> Option<Vec<u8>> {
        self.method_info().runtime_visible_annotations().map(|annotations| runtime_annotations_to_bytes(annotations.annotations.clone()))
    }

    pub fn get_parameter_annotation_bytes(&self) -> Option<Vec<u8>> {
        self.method_info().parameter_annotations().map(|annotations| {
            parameter_annotations_to_bytes(annotations.clone().parameter_annotations)
        })
    }

    pub fn get_annotation_default_bytes(&self) -> Option<Vec<u8>> {
        self.method_info().annotation_default().map(|annotations| annotation_default_to_bytes(annotations.clone()))
    }

    pub fn local_var_slots(&self) -> u16 {
        let local_vars = self.desc().count_local_vars_needed() + if self.is_static() {
            0
        } else {
            1
        };
        self.code_attribute().map(|code| code.max_locals).unwrap_or(local_vars)
    }

    pub fn generic_signature(&self) -> Option<Wtf8Buf> {
        self.method_info().signature_annotation().map(|signature| {
            let i = signature.signature_index as usize;
            self.class_view.underlying_class.constant_pool[i].extract_string_from_utf8()
        })
    }
}

pub enum MethodIterator<'l> {
    ClassBacked { class_view: &'l ClassBackedView, i: usize },
    Empty {},
}

impl<'cl> Iterator for MethodIterator<'cl> {
    type Item = MethodView<'cl>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            MethodIterator::ClassBacked { class_view, i } => {
                if *i >= class_view.num_methods() {
                    return None;
                }
                let res = MethodView::from(class_view, *i as u16);
                *i += 1;
                Some(res)
            }
            MethodIterator::Empty { .. } => None,
        }
    }
}

pub struct LocalVariableView<'cl> {
    method_view: &'cl MethodView<'cl>,
    local_variable_entry: &'cl LocalVariableTableEntry,
}

impl LocalVariableView<'_> {
    pub fn variable_start_pc(&self) -> u16 {
        self.local_variable_entry.start_pc
    }

    pub fn name(&self) -> String {
        let cv = self.method_view.class_view;
        let name_i = self.local_variable_entry.name_index;
        cv.underlying_class.constant_pool[name_i as usize].extract_string_from_utf8().into_string().expect("should have validated this earlier maybe todo")
    }

    pub fn variable_length(&self) -> usize {
        self.local_variable_entry.length as usize
    }

    pub fn desc_str(&self) -> String {
        let cv = self.method_view.class_view;
        let desc_i = self.local_variable_entry.descriptor_index;
        cv.underlying_class.constant_pool[desc_i as usize].extract_string_from_utf8().into_string().expect("should have validated this earlier maybe todo")
    }

    pub fn desc(&self) -> FieldDescriptor {
        parse_field_descriptor(self.desc_str().as_str()).unwrap()
    }

    pub fn local_var_slot(&self) -> u16 {
        self.local_variable_entry.index
    }
}