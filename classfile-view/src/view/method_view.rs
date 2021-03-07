use descriptor_parser::{FieldDescriptor, MethodDescriptor, parse_field_descriptor, parse_method_descriptor};
use rust_jvm_common::classfile::{AttributeType, Code, LineNumberTable, LocalVariableTableEntry, MethodInfo};
use rust_jvm_common::classnames::ClassName;

use crate::view::{ClassBackedView, ClassView, HasAccessFlags};
use crate::view::ptype_view::PTypeView;

pub struct MethodView<'cl> {
    pub(crate) class_view: &'cl ClassBackedView,
    pub(crate) method_i: usize,
}

impl Clone for MethodView<'_> {
    fn clone(&self) -> Self {
        Self {
            class_view: self.class_view,
            method_i: self.method_i,
        }
    }
}

impl HasAccessFlags for MethodView<'_> {
    fn access_flags(&self) -> u16 {
        self.class_view.backing_class.methods[self.method_i].access_flags
    }
}

impl MethodView<'_> {
    fn from(c: &ClassBackedView, i: usize) -> MethodView {
        MethodView { class_view: c, method_i: i }
    }

    pub fn classview(&self) -> &ClassBackedView {
        self.class_view
    }

    fn method_info(&self) -> &MethodInfo {
        &self.class_view.backing_class.methods[self.method_i]
    }

    pub fn name(&self) -> String {
        self.method_info().method_name(&self.class_view.backing_class)
    }

    pub fn desc_str(&self) -> String {
        self.method_info().descriptor_str(&self.class_view.backing_class)
    }

    pub fn desc(&self) -> MethodDescriptor {
        let guard = self.class_view.descriptor_index.read().unwrap();
        match &guard[self.method_i] {
            None => {
                let parsed = parse_method_descriptor(self.desc_str().as_str()).unwrap();
                std::mem::drop(guard);
                self.class_view.descriptor_index.write().unwrap()[self.method_i] = Some(parsed.clone());
                parsed
            }
            Some(res) => res.clone(),
        }
    }

    pub fn code_attribute(&self) -> Option<&Code> {
        self.method_info().code_attribute()//todo get a Code view
    }

    pub fn local_variable_attribute(&self) -> Option<Vec<LocalVariableView>> {
        match self.method_info().code_attribute() {
            None => None,
            Some(code) => {
                code.attributes.iter().find_map(|attr| match &attr.attribute_type {
                    AttributeType::LocalVariableTable(lvt) => Some(lvt),
                    _ => None,
                }).map(|lvt| lvt.local_variable_table.iter().map(|entry| {
                    let local_variable_entry = entry;
                    LocalVariableView { method_view: self, local_variable_entry }
                }).collect::<Vec<LocalVariableView>>())
            }
        }
    }

    pub fn is_signature_polymorphic(&self) -> bool {
        // from the spec:
        // A method is signature polymorphic if all of the following are true:
        // •  It is declared in the java.lang.invoke.MethodHandle class.
        // •  It has a single formal parameter of type Object[].
        // •  It has a return type of Object.
        // •  It has the ACC_VARARGS and ACC_NATIVE flags set.
        self.class_view.name() == ClassName::method_handle() &&
            self.desc().parameter_types.len() == 1 &&
            self.desc().parameter_types[0] == PTypeView::array(PTypeView::object()).to_ptype() &&
            self.desc().return_type == PTypeView::object().to_ptype() &&
            self.is_varargs() &&
            self.is_native()
    }

    //todo this shouldn't be public but needs to be atm.
    pub fn method_i(&self) -> usize {
        self.method_i
    }

    pub fn num_args(&self) -> usize {
        self.desc().parameter_types.len()
    }

    pub fn line_number_table(&self) -> Option<&LineNumberTable> {
        self.method_info().code_attribute().and_then(|attr|
            attr.attributes.iter().find_map(|attr| match &attr.attribute_type {
                AttributeType::LineNumberTable(lnt) => Some(lnt),
                _ => None,
            })
        )
    }
}


pub struct MethodIterator<'l> {
    pub(crate) class_view: &'l ClassBackedView,
    pub(crate) i: usize,
}

impl<'cl> Iterator for MethodIterator<'cl> {
    type Item = MethodView<'cl>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.class_view.num_methods() {
            return None;
        }
        let res = MethodView::from(self.class_view, self.i);
        self.i += 1;
        Some(res)
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
        cv.backing_class.constant_pool[name_i as usize].extract_string_from_utf8()
    }

    pub fn variable_length(&self) -> usize {
        self.local_variable_entry.length as usize
    }

    pub fn desc_str(&self) -> String {
        let cv = self.method_view.class_view;
        let desc_i = self.local_variable_entry.descriptor_index;
        cv.backing_class.constant_pool[desc_i as usize].extract_string_from_utf8()
    }

    pub fn desc(&self) -> FieldDescriptor {
        parse_field_descriptor(self.desc_str().as_str()).unwrap()
    }

    pub fn local_var_slot(&self) -> u16 {
        self.local_variable_entry.index
    }
}