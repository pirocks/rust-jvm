use std::sync::Arc;
use std::ops::Deref;
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::classfile::{Classfile, ACC_STATIC, ACC_FINAL, ACC_NATIVE, ACC_PUBLIC, ACC_PRIVATE, ACC_PROTECTED, ACC_ABSTRACT};
use std::slice::Iter;
use rust_jvm_common::string_pool::StringPoolEntry;
use std::iter;


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

impl Clone for ClassView{
    fn clone(&self) -> Self {
        unimplemented!()
    }
}

pub struct MethodIterator<'l>{
    backing_class: &'l ClassView,
    i : usize
}

impl Iterator for MethodIterator<'_>{
    type Item = MethodView;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.backing_class.num_methods(){
            return None
        }
        let res = MethodView::from(self.backing_class,self.i);
        self.i += 1;
        Some(res)
    }
}

impl ClassView {
    pub fn name(&self) -> ClassName {
        unimplemented!()
    }
    pub fn super_name(&self) -> ClassName {
        unimplemented!()
    }
    pub fn methods<'l>(&'l self) -> MethodIterator<'l> {
        MethodIterator{ backing_class: self, i: 0 }
    }
    pub fn num_methods(&self) -> usize{
        self.backing_class.methods.len()
    }
}

impl HasAccessFlags for ClassView {
    fn access_flags(&self) -> u16 {
        self.backing_class.access_flags
    }
}


pub struct MethodView {
    backing_class: Arc<Classfile>,
    method_i: usize,
}

impl HasAccessFlags for MethodView {
    fn access_flags(&self) -> u16 {
        self.backing_class.methods[self.method_i].access_flags
    }
}

impl MethodView {
    fn from(c: &ClassView, i: usize) -> MethodView {
        unimplemented!()
    }

    pub fn name(&self) -> Arc<StringPoolEntry> {
        unimplemented!()
    }
}
