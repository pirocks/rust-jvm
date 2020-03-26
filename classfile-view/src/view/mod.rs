use std::sync::Arc;
use crate::view::method_view::{MethodIterator, MethodView};
use rust_jvm_common::classfile::{ACC_FINAL, ACC_STATIC, ACC_NATIVE, ACC_PUBLIC, ACC_PRIVATE, ACC_PROTECTED, ACC_ABSTRACT, Classfile, ACC_INTERFACE, ConstantKind, AttributeType, ACC_VARARGS};
use rust_jvm_common::classnames::{ClassName, class_name};
use crate::view::constant_info_view::{ConstantInfoView, ClassPoolElemView, NameAndTypeView, MethodrefView, StringView, IntegerView, FieldrefView, InterfaceMethodrefView, InvokeDynamicView, FloatView, LongView, DoubleView};
use crate::view::field_view::{FieldIterator, FieldView};
use crate::view::interface_view::InterfaceIterator;
use crate::view::attribute_view::{BootstrapMethodsView, EnclosingMethodView};


pub trait HasAccessFlags {
    fn access_flags(&self) -> u16;
    fn is_static(&self) -> bool {
        self.access_flags() & ACC_STATIC > 0
    }
    fn is_final(&self) -> bool {
        self.access_flags() & ACC_FINAL > 0
    }
    fn is_native(&self) -> bool {
        self.access_flags() & ACC_NATIVE > 0
    }
    fn is_varargs(&self) -> bool {
        self.access_flags() & ACC_VARARGS > 0
    }
    fn is_public(&self) -> bool {
        self.access_flags() & ACC_PUBLIC > 0
    }
    fn is_private(&self) -> bool {
        self.access_flags() & ACC_PRIVATE > 0
    }
    fn is_protected(&self) -> bool {
        self.access_flags() & ACC_PROTECTED > 0
    }
    fn is_abstract(&self) -> bool {
        self.access_flags() & ACC_ABSTRACT > 0
    }
    fn is_interface(&self) -> bool {
        self.access_flags() & ACC_INTERFACE > 0
    }
}

#[derive(Debug)]
pub struct ClassView {
    backing_class: Arc<Classfile>
}

impl Clone for ClassView {
    fn clone(&self) -> Self {
        Self { backing_class: self.backing_class.clone() }
    }
}

impl ClassView {
    pub fn from(c : Arc<Classfile>) -> ClassView{
        ClassView { backing_class: c.clone() }
    }
    pub fn name(&self) -> ClassName {
        class_name(&self.backing_class)
    }
    pub fn super_name(&self) -> Option<ClassName> {
        self.backing_class.super_class_name()
    }
    pub fn methods(&self) -> MethodIterator {
        MethodIterator { backing_class: self, i: 0 }
    }
    pub fn method_view_i(&self, i: usize) -> MethodView {
        MethodView { backing_class: self.backing_class.clone(), method_i: i }
    }
    pub fn num_methods(&self) -> usize {
        self.backing_class.methods.len()
    }
    pub fn constant_pool_view(&self, i: usize) -> ConstantInfoView {
        let backing_class = self.backing_class.clone();
        match &self.backing_class.constant_pool[i].kind{
            ConstantKind::Utf8(_) => unimplemented!(),
            ConstantKind::Integer(_) => ConstantInfoView::Integer(IntegerView {}),//todo
            ConstantKind::Float(_) => ConstantInfoView::Float(FloatView{}),//todo
            ConstantKind::Long(_) => ConstantInfoView::Long(LongView{}),//todo
            ConstantKind::Double(_) => ConstantInfoView::Double(DoubleView{}),//todo
            ConstantKind::Class(c) => ConstantInfoView::Class(ClassPoolElemView { backing_class, name_index: c.name_index as usize }),
            ConstantKind::String(_) => ConstantInfoView::String(StringView {}),//todo
            ConstantKind::Fieldref(_) => ConstantInfoView::Fieldref(FieldrefView { backing_class, i }),
            ConstantKind::Methodref(mr) => ConstantInfoView::Methodref(MethodrefView {
                backing_class,
                class_index: mr.class_index,
                name_and_type_index: mr.name_and_type_index
            }),
            ConstantKind::InterfaceMethodref(_) => ConstantInfoView::InterfaceMethodref(InterfaceMethodrefView { backing_class, i }),
            ConstantKind::NameAndType(_) => ConstantInfoView::NameAndType(NameAndTypeView { backing_class, i }),
            ConstantKind::MethodHandle(_) => unimplemented!(),
            ConstantKind::MethodType(_) => unimplemented!(),
            ConstantKind::Dynamic(_) => unimplemented!(),
            ConstantKind::InvokeDynamic(id) => ConstantInfoView::InvokeDynamic(InvokeDynamicView{
                backing_class:self.clone(),
                bootstrap_method_attr_index: id.bootstrap_method_attr_index,
                name_and_type_index: id.name_and_type_index
            }),
            ConstantKind::Module(_) => unimplemented!(),
            ConstantKind::Package(_) => unimplemented!(),
            ConstantKind::InvalidConstant(_) => unimplemented!(),
        }
    }
    pub fn field(&self, i: usize) -> FieldView {
        FieldView::from(self, i )
    }

    pub fn fields(&self) -> FieldIterator {
        FieldIterator { backing_class: &self, i: 0 }
    }
    pub fn interfaces(&self) -> InterfaceIterator {
        InterfaceIterator { backing_class: &self, i: 0 }
    }
    pub fn num_fields(&self) -> usize {
        self.backing_class.fields.len()
    }
    pub fn num_interfaces(&self) -> usize {
        self.backing_class.interfaces.len()
    }
    pub fn backing_class(&self) -> Arc<Classfile>{
        self.backing_class.clone()
    }
    pub fn bootstrap_methods_attr(&self) -> BootstrapMethodsView {
        unimplemented!()
    }
    pub fn enclosing_method_view(&self) -> Option<EnclosingMethodView> {
        self.backing_class.attributes.iter().enumerate().find(|(_i,attr)|{
            match attr.attribute_type {
                AttributeType::EnclosingMethod(_) => true,
                _ => false,
            }
        }).map(|(i,_)|{EnclosingMethodView { backing_class: ClassView::from(self.backing_class.clone()), i }})
    }
}

impl HasAccessFlags for ClassView {
    fn access_flags(&self) -> u16 {
        self.backing_class.access_flags
    }
}

pub mod attribute_view;
pub mod interface_view;
pub mod field_view;
pub mod constant_info_view;
pub mod method_view;
pub mod ptype_view;
pub mod descriptor_parser;