use std::sync::Arc;
use crate::view::method_view::{MethodIterator, MethodView};
use crate::classfile::{ACC_FINAL, ACC_STATIC, ACC_NATIVE, ACC_PUBLIC, ACC_PRIVATE, ACC_PROTECTED, ACC_ABSTRACT, Classfile, ACC_INTERFACE};
use crate::classnames::ClassName;
use crate::view::constant_info_view::ConstantInfoView;
use crate::view::field_view::FieldIterator;
use crate::view::interface_view::InterfaceIterator;


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

pub struct ClassView {
    backing_class: Arc<Classfile>
}

impl Clone for ClassView {
    fn clone(&self) -> Self {
        unimplemented!()
    }
}

impl ClassView {
    pub fn name(&self) -> ClassName {
        unimplemented!()
    }
    pub fn super_name(&self) -> Option<ClassName> {
        unimplemented!()
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
        unimplemented!()
    }
    pub fn fields(&self) -> FieldIterator {
        unimplemented!()
    }
    pub fn interfaces(&self) -> InterfaceIterator {
        unimplemented!()
    }
    pub fn num_fields(&self) -> usize {
        self.backing_class.fields.len()
    }
    pub fn num_interfaces(&self) -> usize {
        self.backing_class.interfaces.len()
    }
}

impl HasAccessFlags for ClassView {
    fn access_flags(&self) -> u16 {
        self.backing_class.access_flags
    }
}

pub mod interface_view;
pub mod field_view;
pub mod constant_info_view;
pub mod method_view;
pub mod ptype_view;