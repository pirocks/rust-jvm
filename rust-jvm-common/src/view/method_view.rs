use crate::view::{HasAccessFlags, ClassView};
use std::sync::Arc;
use crate::classfile::{Classfile, Code};
use crate::string_pool::StringPoolEntry;

pub struct MethodView {
    pub(crate) backing_class: Arc<Classfile>,
    pub(crate) method_i: usize,
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

    pub fn desc_str(&self) -> String {
        unimplemented!()
    }

    pub fn code_attribute(&self) -> Option<&Code>{
        unimplemented!()
    }
}



pub struct MethodIterator<'l> {
    //todo create a from and remove pub(crate)
    pub(crate) backing_class: &'l ClassView,
    pub(crate) i: usize,
}

impl Iterator for MethodIterator<'_> {
    type Item = MethodView;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.backing_class.num_methods() {
            return None;
        }
        let res = MethodView::from(self.backing_class, self.i);
        self.i += 1;
        Some(res)
    }
}