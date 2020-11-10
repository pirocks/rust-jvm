use std::sync::Arc;

use rust_jvm_common::classfile::{AttributeType, BootstrapMethod, CPIndex, SourceFile};

use crate::view::ClassView;
use crate::view::constant_info_view::{ConstantInfoView, DoubleView, FloatView, IntegerView, LongView, MethodHandleView, MethodTypeView, StringView};

#[derive(Clone)]
pub struct BootstrapMethodIterator<'cl> {
    pub(crate) view: BootstrapMethodsView<'cl>,
    pub(crate) i: usize,
}

impl<'cl> Iterator for BootstrapMethodIterator<'cl> {
    type Item = BootstrapMethodView<'cl>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.view.get_bootstrap_methods_raw().len() {
            return None;
        }
        let res = BootstrapMethodView { backing: self.view.clone(), i: self.i };
        self.i += 1;
        res.into()
    }
}

#[derive(Clone)]
pub struct BootstrapMethodsView<'cl> {
    pub(crate) backing_class: &'cl ClassView,
    pub(crate) attr_i: usize,
}

impl BootstrapMethodsView<'_> {
    fn get_bootstrap_methods_raw(&self) -> &Vec<BootstrapMethod> {
        match &self.backing_class.backing_class.attributes[self.attr_i].attribute_type {
            AttributeType::BootstrapMethods(bm) => {
                &bm.bootstrap_methods
            }
            _ => panic!()
        }
    }

    pub fn bootstrap_methods(&self) -> BootstrapMethodIterator {
        unimplemented!()
    }
}

#[derive(Clone)]
pub struct BootstrapMethodView<'cl> {
    pub(crate) backing: BootstrapMethodsView<'cl>,
    pub(crate) i: usize,
}

impl BootstrapMethodView<'_> {
    fn get_raw(&self) -> &BootstrapMethod {
        &self.backing.get_bootstrap_methods_raw()[self.i]
    }

    pub fn bootstrap_method_ref(&self) -> MethodHandleView {
        let i = self.get_raw().bootstrap_method_ref;
        dbg!(i);
        let res = self.backing.backing_class.constant_pool_view(i as usize);
        match res {
            ConstantInfoView::MethodHandle(mh) => { mh }
            _ => panic!()
        }
    }

    pub fn bootstrap_args(&self) -> BootstrapArgViewIterator {
        BootstrapArgViewIterator {
            backing_class: self.backing.backing_class,
            bootstrap_args: self.get_raw().bootstrap_arguments.clone(),
            i: 0,
        }
    }
}

pub struct BootstrapArgViewIterator<'cl> {
    backing_class: &'cl ClassView,
    bootstrap_args: Vec<CPIndex>,
    //todo get rid of clone for this
    i: usize,
}


impl<'cl> Iterator for BootstrapArgViewIterator<'cl> {
    type Item = BootstrapArgView<'cl>;

    fn next(&mut self) -> Option<Self::Item> {
        let arg = self.bootstrap_args[self.i];
        let res = match self.backing_class.constant_pool_view(arg as usize) {
            ConstantInfoView::Integer(i) => BootstrapArgView::Integer(i),
            ConstantInfoView::MethodType(mt) => BootstrapArgView::MethodType(mt),
            ConstantInfoView::MethodHandle(mh) => BootstrapArgView::MethodHandle(mh),
            ConstantInfoView::String(s) => BootstrapArgView::String(s),
            // ConstantInfoView::Class(cpelem) => BootstrapArgView::Class(cpelem),
            other => {
                dbg!(other);
                unimplemented!()
            }
        }.into();
        self.i += 1;
        res
    }
}

//CONSTANT_String_info,  CONSTANT_Class_info,CONSTANT_Integer_info, CONSTANT_Long_info,
// CONSTANT_Float_info, CONSTANT_Double_info,CONSTANT_MethodHandle_info, or CONSTANT_MethodType_info
pub enum BootstrapArgView<'cl> {
    String(StringView<'cl>),
    Class(Arc<ClassView>),
    Integer(IntegerView),
    Long(LongView),
    Float(FloatView),
    Double(DoubleView),
    MethodHandle(MethodHandleView<'cl>),
    MethodType(MethodTypeView<'cl>),
}


#[allow(dead_code)]
pub struct EnclosingMethodView {
    pub(crate) backing_class: ClassView,
    pub(crate) i: usize,
}

impl EnclosingMethodView {
    // fn get_raw(&self) -> &EnclosingMethod{
    //     match &self.backing_class.backing_class.attributes[self.i].attribute_type{
    //         AttributeType::EnclosingMethod(em) => em,
    //         _ => panic!()
    //     }
    // }
}


pub struct SourceFileView<'l> {
    pub(crate) backing_class: &'l ClassView,
    pub(crate) i: usize,
}

impl SourceFileView<'_> {
    fn source_file_attr(&self) -> &SourceFile {
        match &self.backing_class.backing_class.attributes[self.i].attribute_type {
            AttributeType::SourceFile(sf) => sf,
            _ => panic!(),
        }
    }

    pub fn file(&self) -> String {
        let si = self.source_file_attr().sourcefile_index;
        self.backing_class.backing_class.constant_pool[si as usize].extract_string_from_utf8()
    }
}