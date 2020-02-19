use std::sync::Arc;
use std::ops::Deref;
use std::slice::Iter;
use std::iter;
use crate::view::method_view::{MethodView, MethodIterator};
use crate::classfile::{ACC_FINAL, ACC_STATIC, ACC_NATIVE, ACC_PUBLIC, ACC_PRIVATE, ACC_PROTECTED, ACC_ABSTRACT, Classfile};
use crate::classnames::ClassName;
use crate::view::constant_info_view::ConstantInfoView;


trait HasAccessFlags {
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
    pub fn super_name(&self) -> ClassName {
        unimplemented!()
    }
    pub fn methods(&self) -> MethodIterator {
        MethodIterator { backing_class: self, i: 0 }
    }
    pub fn num_methods(&self) -> usize {
        self.backing_class.methods.len()
    }
    pub fn constant_pool_view(&self, i: usize) -> ConstantInfoView {
        unimplemented!()
    }
}

impl HasAccessFlags for ClassView {
    fn access_flags(&self) -> u16 {
        self.backing_class.access_flags
    }
}

pub mod constant_info_view{
    pub enum ConstantInfoView {}
}

pub mod method_view;
pub mod ptype_view;