use crate::view::ClassView;
use rust_jvm_common::classfile::{AttributeType, BootstrapMethod, CPIndex};
use crate::view::constant_info_view::{ConstantInfoView, StringView, IntegerView, LongView, FloatView, DoubleView, MethodHandleView, MethodTypeView};

#[derive(Clone)]
pub struct BootstrapMethodIterator {
    pub(crate) view: BootstrapMethodsView,
    pub(crate) i: usize,
}

impl Iterator for BootstrapMethodIterator {
    type Item = BootstrapMethodView;

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
pub struct BootstrapMethodsView {
    pub(crate) backing_class: ClassView,
    pub(crate) attr_i: usize,
}

impl BootstrapMethodsView {
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
pub struct BootstrapMethodView {
    pub(crate) backing: BootstrapMethodsView,
    pub(crate) i: usize,
}

impl BootstrapMethodView {
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
            backing_class: self.backing.backing_class.clone(),
            bootstrap_args: self.get_raw().bootstrap_arguments.clone(),
            i: 0
        }
    }
}

pub struct BootstrapArgViewIterator {
    backing_class: ClassView,
    bootstrap_args : Vec<CPIndex>,//todo get rid of clone for this
    i : usize
}


impl Iterator for BootstrapArgViewIterator{
    type Item = BootstrapArgView<'static>;

    fn next(&mut self) -> Option<Self::Item> {
        let arg = self.bootstrap_args[self.i];
        let res = match self.backing_class.constant_pool_view(arg as usize){
            ConstantInfoView::Integer(i) => BootstrapArgView::Integer(i),
            _ => unimplemented!()
        }.into();
        self.i += 1;
        res
    }
}

//CONSTANT_String_info,  CONSTANT_Class_info,CONSTANT_Integer_info, CONSTANT_Long_info,
// CONSTANT_Float_info, CONSTANT_Double_info,CONSTANT_MethodHandle_info, or CONSTANT_MethodType_info
pub enum BootstrapArgView<'l> {
    String(StringView<'l>),
    Class(ClassView),
    Integer(IntegerView),
    Long(LongView),
    Float(FloatView),
    Double(DoubleView),
    MethodHandle(MethodHandleView),
    MethodType(MethodTypeView)
}


#[allow(dead_code)]
pub struct EnclosingMethodView{
    pub(crate) backing_class : ClassView ,
    pub(crate) i : usize
}

impl EnclosingMethodView{
    // fn get_raw(&self) -> &EnclosingMethod{
    //     match &self.backing_class.backing_class.attributes[self.i].attribute_type{
    //         AttributeType::EnclosingMethod(em) => em,
    //         _ => panic!()
    //     }
    // }


}