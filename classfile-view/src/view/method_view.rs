use crate::view::{HasAccessFlags, ClassView};
use std::sync::Arc;
use rust_jvm_common::classfile::{Classfile, Code, MethodInfo};
use crate::view::ptype_view::PTypeView;
use rust_jvm_common::classnames::ClassName;
use descriptor_parser::{MethodDescriptor, parse_method_descriptor};

pub struct MethodView {
    pub(crate) backing_class: Arc<Classfile>,
    pub(crate) method_i: usize,
}

impl Clone for MethodView{
    fn clone(&self) -> Self {
        Self {
            backing_class: self.backing_class.clone(),
            method_i: self.method_i
        }
    }
}

impl HasAccessFlags for MethodView {
    fn access_flags(&self) -> u16 {
        self.backing_class.methods[self.method_i].access_flags
    }
}

impl MethodView {
    fn from(c: &ClassView, i: usize) -> MethodView {
        MethodView { backing_class: c.backing_class.clone(), method_i: i }
    }

    pub fn classview(&self) -> ClassView{
        ClassView::from(self.backing_class.clone())
    }

    //todo shouldn't be public but needs to be
    pub fn method_info(&self) -> &MethodInfo{
        &self.backing_class.methods[self.method_i]
    }

    pub fn name(&self) -> String {
        self.method_info().method_name(&self.backing_class)
    }

    pub fn desc_str(&self) -> String {
        self.method_info().descriptor_str(&self.backing_class)
    }

    pub fn desc(&self) -> MethodDescriptor {
        parse_method_descriptor( self.desc_str().as_str()).unwrap()
    }

    pub fn code_attribute(&self) -> Option<&Code>{
        self.method_info().code_attribute()//todo get a Code view
    }

    pub fn is_signature_polymorphic(&self) -> bool{
        // from the spec:
        // A method is signature polymorphic if all of the following are true:
        // •  It is declared in the java.lang.invoke.MethodHandle class.
        // •  It has a single formal parameter of type Object[].
        // •  It has a return type of Object.
        // •  It has the ACC_VARARGS and ACC_NATIVE flags set.
        ClassView::from(self.backing_class.clone()).name() == ClassName::method_handle() &&
            self.desc().parameter_types.len()  == 1 &&
            self.desc().parameter_types[0] == PTypeView::array(PTypeView::object()).to_ptype() &&
            self.desc().return_type == PTypeView::object().to_ptype() &&
            self.is_varargs() &&
            self.is_native()
    }

    //todo this shouldn't be public but needs to be atm.
    pub fn method_i(&self)-> usize {
        self.method_i
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